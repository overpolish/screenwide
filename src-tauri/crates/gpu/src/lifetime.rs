// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! When the shared device is open. It opens on first use and closes once
//! nothing has used it for [`IDLE_RELEASE`] and nothing holds it: the driver
//! keeps memory for a device's whole life, such as the pools video decoding
//! leaves behind, and only closing the device gives it back. On integrated
//! graphics that memory is the computer's RAM.
//!
//! Whatever draws holds a [`Gpu`] handle while it lives, so the device stays
//! open under it. Caches of what is made on the device live in a
//! [`DeviceCache`], which lets go of its hold when the device would close
//! and is filled again on the next use.

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, Once, PoisonError, TryLockError, Weak};
use std::time::{Duration, Instant};

use super::Gpu;

/// How long the device stays open after its last use, so closing and
/// reopening something in quick succession does not reopen the device.
const IDLE_RELEASE: Duration = Duration::from_secs(60);
/// How often an idle device is looked at.
const IDLE_CHECK: Duration = Duration::from_secs(15);

/// The open device, or why it could not open. A failure is kept, so every
/// renderer reports the same reason instead of retrying a device that
/// cannot open.
static CURRENT: Mutex<Option<Result<Arc<Gpu>, String>>> = Mutex::new(None);
static GENERATIONS: AtomicU64 = AtomicU64::new(0);
/// When the device was last asked for, in milliseconds since [`epoch`].
static LAST_USE: AtomicU64 = AtomicU64::new(0);
static WATCH: Once = Once::new();

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
  mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

fn epoch() -> Instant {
  static EPOCH: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
  *EPOCH.get_or_init(Instant::now)
}

fn now_ms() -> u64 {
  u64::try_from(epoch().elapsed().as_millis()).unwrap_or(u64::MAX)
}

/// The shared device, opened if it is not.
pub fn shared() -> Result<Arc<Gpu>, String> {
  LAST_USE.store(now_ms(), Ordering::Relaxed);
  let mut current = lock(&CURRENT);
  if let Some(opened) = current.as_ref() {
    return opened.clone();
  }
  let generation = GENERATIONS.fetch_add(1, Ordering::Relaxed);
  let opened = pollster::block_on(Gpu::open(generation)).map(Arc::new);
  *current = Some(opened.clone());
  drop(current);
  WATCH.call_once(|| {
    let _ = std::thread::Builder::new()
      .name("screenwide-gpu-idle".to_owned())
      .spawn(watch_idle);
  });
  opened
}

/// Starts opening the shared device on a thread of its own, for a caller
/// that will draw soon but has other work first, such as an overlay that
/// freezes the desktop before drawing over it. Whatever asks for the device
/// meanwhile waits only for what is left of the opening.
pub fn open_soon() {
  LAST_USE.store(now_ms(), Ordering::Relaxed);
  // Held while another thread opens the device: it is on its way already.
  let current = match CURRENT.try_lock() {
    Ok(current) => current,
    Err(TryLockError::Poisoned(error)) => error.into_inner(),
    Err(TryLockError::WouldBlock) => return,
  };
  if current.is_some() {
    return;
  }
  drop(current);
  let _ = std::thread::Builder::new()
    .name("screenwide-gpu-open".to_owned())
    .spawn(|| {
      let _ = shared();
    });
}

/// The device if it is open, without opening it.
pub(crate) fn current() -> Option<Arc<Gpu>> {
  lock(&CURRENT)
    .as_ref()
    .and_then(|opened| opened.clone().ok())
}

fn watch_idle() {
  loop {
    std::thread::sleep(IDLE_CHECK);
    let idle = now_ms().saturating_sub(LAST_USE.load(Ordering::Relaxed));
    if idle >= IDLE_RELEASE.as_millis() as u64 {
      // Metal autoreleases some of what the device frees, and this thread
      // lives as long as the app, so only a pool of its own ever empties.
      #[cfg(target_os = "macos")]
      objc2::rc::autoreleasepool(|_| close_if_unused());
      #[cfg(not(target_os = "macos"))]
      close_if_unused();
    }
  }
}

/// Asked to let go of what they hold on the device once it has been idle,
/// such as views kept for windows that are hidden; whatever they still hold
/// keeps the device open until the next check.
static IDLE_HOOKS: Mutex<Vec<fn()>> = Mutex::new(Vec::new());

/// Has `hook` called each time the device has been idle for long enough to
/// close, before it is closed.
pub fn on_idle(hook: fn()) {
  lock(&IDLE_HOOKS).push(hook);
}

/// How many drawing surfaces are alive. Each is something on screen, or
/// held for a window that will be, so the device stays open and its caches
/// stay filled while any exists: a window left untouched calls nothing, and
/// would otherwise look idle.
static LIVE_SURFACES: AtomicUsize = AtomicUsize::new(0);

/// Counts a surface for as long as it lives.
pub(crate) struct LiveSurface;

impl LiveSurface {
  pub(crate) fn new() -> Self {
    LIVE_SURFACES.fetch_add(1, Ordering::AcqRel);
    Self
  }
}

impl Drop for LiveSurface {
  fn drop(&mut self) {
    LIVE_SURFACES.fetch_sub(1, Ordering::AcqRel);
  }
}

/// Closes the device if nothing but this module and the caches hold it.
fn close_if_unused() {
  if current().is_none() {
    return;
  }
  let hooks = lock(&IDLE_HOOKS).clone();
  for hook in hooks {
    hook();
  }
  // A hook's surfaces may go a moment later, on the thread that owns them;
  // the next check finds them gone.
  if LIVE_SURFACES.load(Ordering::Acquire) > 0 {
    return;
  }
  // The caches let go first; whatever still uses what they made keeps it,
  // and with it the device.
  for cache in lock(&CACHES).iter() {
    cache.lend();
  }
  let closed = {
    let mut current = lock(&CURRENT);
    match current.as_ref() {
      Some(Ok(gpu)) if Arc::strong_count(gpu) == 1 => current.take(),
      _ => None,
    }
  };
  if let Some(Ok(gpu)) = closed {
    // Work still in flight finishes before the device goes.
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    drop(gpu);
  }
}

trait Lend: Sync {
  /// Exchanges the cache's hold on what it made for a weak one.
  fn lend(&self);
}

static CACHES: Mutex<Vec<&'static dyn Lend>> = Mutex::new(Vec::new());

enum Slot<T> {
  Empty,
  Held(u64, Arc<T>),
  Lent(u64, Weak<T>),
}

/// Something made once on the device and shared, such as compiled pipelines.
/// It belongs to one device: the device opened again makes it again. Its hold
/// becomes a weak one when the device would close, so it never keeps the
/// device open; while something still uses it, the next asker gets it back.
pub struct DeviceCache<T> {
  slot: Mutex<Slot<T>>,
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
      slot: Mutex::new(Slot::Empty),
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
    let mut slot = lock(&self.slot);
    match &*slot {
      Slot::Held(generation, made) if *generation == gpu.generation => {
        return Ok(Arc::clone(made));
      }
      Slot::Lent(generation, made) if *generation == gpu.generation => {
        if let Some(made) = made.upgrade() {
          *slot = Slot::Held(*generation, Arc::clone(&made));
          return Ok(made);
        }
      }
      _ => {}
    }
    let made = Arc::new(make(gpu)?);
    *slot = Slot::Held(gpu.generation, Arc::clone(&made));
    Ok(made)
  }

  /// What was made, if anything still holds it, without making it.
  pub fn peek(&self) -> Option<Arc<T>> {
    match &*lock(&self.slot) {
      Slot::Held(_, made) => Some(Arc::clone(made)),
      Slot::Lent(_, made) => made.upgrade(),
      Slot::Empty => None,
    }
  }
}

impl<T: Send + Sync> Lend for DeviceCache<T> {
  fn lend(&self) {
    let mut slot = lock(&self.slot);
    if let Slot::Held(generation, made) = &*slot {
      *slot = Slot::Lent(*generation, Arc::downgrade(made));
    }
  }
}
