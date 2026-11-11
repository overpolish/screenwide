// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! When the shared devices are open. Each [`Workload`] has one, which opens
//! on first use and closes once nothing has used it for [`IDLE_RELEASE`] and
//! nothing holds it: the driver keeps memory for a device's whole life, such
//! as the pools video decoding leaves behind, and only closing the device
//! gives it back. On integrated graphics that memory is the computer's RAM,
//! and an open device keeps a discrete graphics processor from switching
//! off. Where both workloads would open on the same graphics processor, they
//! share one device, which closes once neither has used it for that long.
//!
//! Whatever draws holds a [`Gpu`] handle while it lives, so the device stays
//! open under it. Caches of what is made on a device live in a
//! [`DeviceCache`], which lets go of its hold when the device would close
//! and is filled again on the next use.

mod cache;
pub use cache::DeviceCache;

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, Once, PoisonError, TryLockError};
use std::time::{Duration, Instant};

use super::{Gpu, Workload};

/// How long a device stays open after its last use, so closing and reopening
/// something in quick succession does not reopen the device.
const IDLE_RELEASE: Duration = Duration::from_secs(60);
/// How often an idle device is looked at.
const IDLE_CHECK: Duration = Duration::from_secs(15);

#[cfg(target_os = "windows")]
const SLOTS: usize = 2;
#[cfg(not(target_os = "windows"))]
const SLOTS: usize = 1;

type Opened = Option<Result<Arc<Gpu>, String>>;

/// Each workload's device, or why it could not open. A failure is kept, so
/// every renderer reports the same reason instead of retrying a device that
/// cannot open.
static OPENED: [Mutex<Opened>; SLOTS] = [const { Mutex::new(None) }; SLOTS];
/// When each workload's device was last asked for, in milliseconds since
/// [`epoch`].
static LAST_USE: [AtomicU64; SLOTS] = [const { AtomicU64::new(0) }; SLOTS];
static GENERATIONS: AtomicU64 = AtomicU64::new(0);
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

/// Which slot `workload` keeps its device in: one each on Windows, one for
/// both elsewhere.
const fn slot(workload: Workload) -> usize {
  match workload {
    Workload::Light => 0,
    Workload::Heavy => SLOTS - 1,
  }
}

/// `workload`'s device, opened if it is not.
pub fn shared(workload: Workload) -> Result<Arc<Gpu>, String> {
  let index = slot(workload);
  LAST_USE[index].store(now_ms(), Ordering::Relaxed);
  let mut opened = lock(&OPENED[index]);
  if let Some(opened) = opened.as_ref() {
    return opened.clone();
  }
  let result = open(workload, index);
  *opened = Some(result.clone());
  drop(opened);
  WATCH.call_once(|| {
    let _ = std::thread::Builder::new()
      .name("screenwide-gpu-idle".to_owned())
      .spawn(watch_idle);
  });
  result
}

/// Opens `workload`'s device, or joins the other workload's open device
/// where `workload` would open on the same graphics processor.
fn open(workload: Workload, index: usize) -> Result<Arc<Gpu>, String> {
  // Not waited for: a device still opening there is opened for itself.
  let other = (0..SLOTS)
    .filter(|&other| other != index)
    .find_map(|other| match OPENED[other].try_lock() {
      Ok(opened) => opened.as_ref().and_then(|opened| opened.clone().ok()),
      Err(_) => None,
    });
  let join = other.as_ref().and_then(|gpu| gpu.adapter_id);
  let generation = GENERATIONS.fetch_add(1, Ordering::Relaxed);
  match pollster::block_on(Gpu::open(workload, generation, join))? {
    Some(gpu) => Ok(Arc::new(gpu)),
    None => other.ok_or_else(|| "The device to share has closed".to_owned()),
  }
}

/// A device for a picture drawn once and read back, which any device can
/// draw: the heavy one while it is open, as the editor keeps it, so nothing
/// else wakes; otherwise the light one.
pub fn shared_for_readback() -> Result<Arc<Gpu>, String> {
  let heavy = slot(Workload::Heavy);
  if let Ok(opened) = OPENED[heavy].try_lock() {
    if let Some(Ok(gpu)) = opened.as_ref() {
      LAST_USE[heavy].store(now_ms(), Ordering::Relaxed);
      return Ok(Arc::clone(gpu));
    }
  }
  shared(Workload::Light)
}

/// Starts opening `workload`'s device on a thread of its own, for a caller
/// that will draw soon but has other work first, such as an overlay that
/// freezes the desktop before drawing over it. Whatever asks for the device
/// meanwhile waits only for what is left of the opening.
pub fn open_soon(workload: Workload) {
  let index = slot(workload);
  LAST_USE[index].store(now_ms(), Ordering::Relaxed);
  // Held while another thread opens the device: it is on its way already.
  let opened = match OPENED[index].try_lock() {
    Ok(opened) => opened,
    Err(TryLockError::Poisoned(error)) => error.into_inner(),
    Err(TryLockError::WouldBlock) => return,
  };
  if opened.is_some() {
    return;
  }
  drop(opened);
  let _ = std::thread::Builder::new()
    .name("screenwide-gpu-open".to_owned())
    .spawn(move || {
      let _ = shared(workload);
    });
}

/// Every open device, each once, without opening any.
pub(crate) fn open_devices() -> Vec<Arc<Gpu>> {
  let mut devices: Vec<Arc<Gpu>> = Vec::new();
  for slot in &OPENED {
    if let Some(Ok(gpu)) = lock(slot).as_ref() {
      if !devices.iter().any(|open| Arc::ptr_eq(open, gpu)) {
        devices.push(Arc::clone(gpu));
      }
    }
  }
  devices
}

fn watch_idle() {
  loop {
    std::thread::sleep(IDLE_CHECK);
    // Metal autoreleases some of what the device frees, and this thread
    // lives as long as the app, so only a pool of its own ever empties.
    #[cfg(target_os = "macos")]
    objc2::rc::autoreleasepool(|_| close_unused());
    #[cfg(not(target_os = "macos"))]
    close_unused();
  }
}

/// Asked to let go of what they hold on a workload's device once it has been
/// idle, such as views kept for windows that are hidden; whatever they still
/// hold keeps the device open until the next check.
type IdleHook = (Workload, fn());
static IDLE_HOOKS: Mutex<Vec<IdleHook>> = Mutex::new(Vec::new());

/// Has `hook` called each time `workload`'s device has been idle for long
/// enough to close, before it is closed.
pub fn on_idle(workload: Workload, hook: fn()) {
  lock(&IDLE_HOOKS).push((workload, hook));
}

/// Counts a drawing surface on a device for as long as it lives. Each is
/// something on screen, or held for a window that will be, so its device
/// stays open and its caches stay filled while any exists: a window left
/// untouched calls nothing, and would otherwise look idle.
pub(crate) struct LiveSurface(Arc<AtomicUsize>);

impl LiveSurface {
  pub(crate) fn new(gpu: &Gpu) -> Self {
    gpu.live_surfaces.fetch_add(1, Ordering::AcqRel);
    Self(Arc::clone(&gpu.live_surfaces))
  }
}

impl Drop for LiveSurface {
  fn drop(&mut self) {
    self.0.fetch_sub(1, Ordering::AcqRel);
  }
}

/// Closes each device that every workload on it has left idle, if nothing
/// but this module and the caches holds it.
fn close_unused() {
  let now = now_ms();
  for gpu in open_devices() {
    let holders: Vec<usize> = (0..SLOTS).filter(|&index| holds(index, &gpu)).collect();
    let idle = holders.iter().all(|&index| {
      now.saturating_sub(LAST_USE[index].load(Ordering::Relaxed)) >= IDLE_RELEASE.as_millis() as u64
    });
    if !idle {
      continue;
    }
    let hooks = lock(&IDLE_HOOKS).clone();
    for (workload, hook) in hooks {
      if holders.contains(&slot(workload)) {
        hook();
      }
    }
    // A hook's surfaces may go a moment later, on the thread that owns them;
    // the next check finds them gone.
    if gpu.live_surfaces.load(Ordering::Acquire) > 0 {
      continue;
    }
    // The caches let go first; whatever still uses what they made keeps it,
    // and with it the device.
    cache::lend_all(gpu.generation);
    close(gpu, &holders);
  }
}

fn holds(index: usize, gpu: &Arc<Gpu>) -> bool {
  matches!(lock(&OPENED[index]).as_ref(), Some(Ok(held)) if Arc::ptr_eq(held, gpu))
}

/// Takes `gpu` out of the slots in `holders` and closes it, if they and the
/// caller are all that hold it.
fn close(gpu: Arc<Gpu>, holders: &[usize]) {
  // Locked together, so nothing takes a hold between the count and the take.
  let mut opened: Vec<MutexGuard<'_, Opened>> =
    holders.iter().map(|&index| lock(&OPENED[index])).collect();
  let held_here = opened
    .iter()
    .filter(|slot| matches!(slot.as_ref(), Some(Ok(held)) if Arc::ptr_eq(held, &gpu)))
    .count();
  if held_here != holders.len() || Arc::strong_count(&gpu) != holders.len() + 1 {
    return;
  }
  for slot in &mut opened {
    **slot = None;
  }
  drop(opened);
  // Work still in flight finishes before the device goes.
  let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
}
