// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What is made once on a device and shared, kept for each open device.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Weak};

use super::super::Gpu;
use super::lock;

trait Lend: Sync {
  /// Exchanges the hold on what was made on the device `generation` opened
  /// as for a weak one.
  fn lend(&self, generation: u64);
}

static CACHES: Mutex<Vec<&'static dyn Lend>> = Mutex::new(Vec::new());

/// Has every cache let go of its hold on what it made on the device
/// `generation` opened as.
pub(super) fn lend_all(generation: u64) {
  for cache in lock(&CACHES).iter() {
    cache.lend(generation);
  }
}

enum Hold<T> {
  Held(Arc<T>),
  Lent(Weak<T>),
}

/// Something made once on a device and shared, such as compiled pipelines.
/// It belongs to the device it was made on, so each open device has its own,
/// and a device opened again makes it again. Its hold becomes a weak one when
/// its device would close, so it never keeps the device open; while something
/// still uses it, the next asker gets it back.
pub struct DeviceCache<T> {
  /// What was made, with the generation of the device it was made on.
  made: Mutex<Vec<(u64, Hold<T>)>>,
  registered: AtomicBool,
}

impl<T> Default for DeviceCache<T> {
  fn default() -> Self {
    Self::new()
  }
}

impl<T> DeviceCache<T> {
  pub const fn new() -> Self {
    Self {
      made: Mutex::new(Vec::new()),
      registered: AtomicBool::new(false),
    }
  }
}

impl<T: Send + Sync + 'static> DeviceCache<T> {
  /// What is made on `gpu`, made by `make` if it is not.
  pub fn get_or_try_init(
    &'static self,
    gpu: &Arc<Gpu>,
    make: impl FnOnce(&Arc<Gpu>) -> Result<T, String>,
  ) -> Result<Arc<T>, String> {
    if !self.registered.swap(true, Ordering::AcqRel) {
      lock(&CACHES).push(self);
    }
    let mut made = lock(&self.made);
    // What nothing holds any more belongs to a device that has closed, or is
    // about to be made again.
    made.retain(|(_, hold)| match hold {
      Hold::Held(_) => true,
      Hold::Lent(weak) => weak.strong_count() > 0,
    });
    if let Some(index) = made
      .iter()
      .position(|(generation, _)| *generation == gpu.generation)
    {
      let found = match &made[index].1 {
        Hold::Held(found) => Some(Arc::clone(found)),
        Hold::Lent(weak) => weak.upgrade(),
      };
      if let Some(found) = found {
        made[index].1 = Hold::Held(Arc::clone(&found));
        return Ok(found);
      }
      made.remove(index);
    }
    let made_now = Arc::new(make(gpu)?);
    made.push((gpu.generation, Hold::Held(Arc::clone(&made_now))));
    Ok(made_now)
  }

  /// What was made on `gpu`, if anything still holds it, without making it.
  pub fn peek(&self, gpu: &Gpu) -> Option<Arc<T>> {
    lock(&self.made)
      .iter()
      .find(|(generation, _)| *generation == gpu.generation)
      .and_then(|(_, hold)| match hold {
        Hold::Held(made) => Some(Arc::clone(made)),
        Hold::Lent(weak) => weak.upgrade(),
      })
  }
}

impl<T: Send + Sync> Lend for DeviceCache<T> {
  fn lend(&self, generation: u64) {
    for (made_on, hold) in lock(&self.made).iter_mut() {
      if let (true, Hold::Held(made)) = (*made_on == generation, &*hold) {
        *hold = Hold::Lent(Arc::downgrade(made));
      }
    }
  }
}
