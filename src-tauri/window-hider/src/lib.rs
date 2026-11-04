// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Loaded into an app Screenwide hides while it records. Windows keeps a
//! window out of every capture only when its own process asks
//! (`SetWindowDisplayAffinity`), so Screenwide starts threads in the app on
//! the two exports here. [`screenwide_hide`] keeps each window's own setting
//! and asks for its exclusion, and goes on doing so for windows the app opens
//! later; [`screenwide_restore`] stops that and gives every window back the
//! setting it had. Nothing runs as the library loads, as the loader's lock is
//! held then.

#[cfg(windows)]
mod hider {
  use std::sync::atomic::{AtomicBool, Ordering};
  use std::sync::{Mutex, PoisonError};
  use std::thread::JoinHandle;
  use std::time::Duration;

  use windows::core::BOOL;
  use windows::Win32::Foundation::{HWND, LPARAM};
  use windows::Win32::System::Threading::GetCurrentProcessId;
  use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowDisplayAffinity, GetWindowThreadProcessId, IsWindow,
    SetWindowDisplayAffinity, WDA_EXCLUDEFROMCAPTURE, WDA_NONE, WINDOW_DISPLAY_AFFINITY,
  };

  /// How often windows opened since are looked for: a new window may show in
  /// a capture for up to this long.
  const WATCH: Duration = Duration::from_millis(100);

  /// Each window hidden, by handle, with the setting it had.
  static KEPT: Mutex<Vec<(isize, u32)>> = Mutex::new(Vec::new());
  static WATCHING: AtomicBool = AtomicBool::new(false);
  static WATCHER: Mutex<Option<JoinHandle<()>>> = Mutex::new(None);

  fn own_windows() -> Vec<HWND> {
    unsafe extern "system" fn collect(window: HWND, found: LPARAM) -> BOOL {
      let mut process = 0;
      unsafe { GetWindowThreadProcessId(window, Some(&raw mut process)) };
      if process == unsafe { GetCurrentProcessId() } {
        unsafe { (*(found.0 as *mut Vec<HWND>)).push(window) };
      }
      true.into()
    }
    let mut found: Vec<HWND> = Vec::new();
    let _ = unsafe { EnumWindows(Some(collect), LPARAM(&raw mut found as isize)) };
    found
  }

  /// Hides the windows not hidden yet, keeping each one's own setting. One
  /// already kept from every capture, or shown black in them, is left as it
  /// is, as that keeps it out already.
  fn hide_new() -> u32 {
    let mut kept = KEPT.lock().unwrap_or_else(PoisonError::into_inner);
    let mut hidden = 0;
    for window in own_windows() {
      let handle = window.0 as isize;
      if kept.iter().any(|(kept, _)| *kept == handle) {
        continue;
      }
      let mut affinity = 0;
      if unsafe { GetWindowDisplayAffinity(window, &raw mut affinity) }.is_err() {
        continue;
      }
      if affinity == WDA_NONE.0
        && unsafe { SetWindowDisplayAffinity(window, WDA_EXCLUDEFROMCAPTURE) }.is_ok()
      {
        kept.push((handle, affinity));
        hidden += 1;
      }
    }
    hidden
  }

  pub fn hide() -> u32 {
    let hidden = hide_new();
    if !WATCHING.swap(true, Ordering::AcqRel) {
      // Built rather than spawned, so a thread the app cannot start is
      // given up rather than a panic that would take the app down.
      let watcher = std::thread::Builder::new().spawn(|| {
        while WATCHING.load(Ordering::Acquire) {
          std::thread::sleep(WATCH);
          hide_new();
        }
      });
      match watcher {
        Ok(watcher) => *WATCHER.lock().unwrap_or_else(PoisonError::into_inner) = Some(watcher),
        Err(_) => WATCHING.store(false, Ordering::Release),
      }
    }
    hidden
  }

  pub fn restore() -> u32 {
    WATCHING.store(false, Ordering::Release);
    if let Some(watcher) = WATCHER.lock().unwrap_or_else(PoisonError::into_inner).take() {
      let _ = watcher.join();
    }
    let mut kept = KEPT.lock().unwrap_or_else(PoisonError::into_inner);
    let mut restored = 0;
    for (handle, affinity) in kept.drain(..) {
      let window = HWND(handle as *mut _);
      if unsafe { IsWindow(Some(window)) }.as_bool()
        && unsafe { SetWindowDisplayAffinity(window, WINDOW_DISPLAY_AFFINITY(affinity)) }.is_ok()
      {
        restored += 1;
      }
    }
    restored
  }
}

/// A thread's start: hides the app's windows, and those it opens after,
/// until [`screenwide_restore`]. Returns how many it hid now.
///
/// # Safety
/// Started by Screenwide as a thread in the app; the argument is unused.
#[cfg(windows)]
#[no_mangle]
pub unsafe extern "system" fn screenwide_hide(_: *mut core::ffi::c_void) -> u32 {
  hider::hide()
}

/// A thread's start: stops hiding and gives each window back its own
/// setting. Returns how many it restored. The library may be unloaded once
/// it returns.
///
/// # Safety
/// Started by Screenwide as a thread in the app; the argument is unused.
#[cfg(windows)]
#[no_mangle]
pub unsafe extern "system" fn screenwide_restore(_: *mut core::ffi::c_void) -> u32 {
  hider::restore()
}
