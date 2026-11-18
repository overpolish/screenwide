// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Passive native-overlay keyboard monitor for Windows.
//!
//! The full-desktop OSC windows deliberately use `WS_EX_NOACTIVATE`; WebView2
//! therefore owns keyboard focus even while the native compositor owns pointer
//! input. macOS solves the same split with a local `NSEvent` monitor. Windows
//! takes the overlay's place first on the process's keyboard hook while an
//! overlay is up, and posts only keys recognised by the active overlay back to
//! the OSC window's owning UI thread.

mod hooks;
use hooks::active_overlay;
pub(crate) use hooks::alt_pressed;
use hooks::cancel_alt_menu_activation;
use hooks::post_alt_transition;
use hooks::route;
#[cfg(test)]
use hooks::routes_to_overlay;

use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicU32, AtomicU8, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use screenwide_keyboard_hook::{Priority, Registration};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_MENU};
use windows::Win32::UI::WindowsAndMessaging::{
  DispatchMessageW, GetMessageW, KillTimer, PostMessageW, PostThreadMessageW, SetTimer,
  TranslateMessage, MSG, WM_APP, WM_QUIT, WM_TIMER,
};

/// One key the monitor recognised, posted to the overlay window that owns it.
/// `wparam` is the virtual key; `lparam` carries the flags below. It lives
/// here, beside the hook that posts it, so every consumer decodes the same
/// protocol from the same place.
pub(crate) const OVERLAY_KEY_EVENT: u32 = WM_APP + 0x341;

pub(crate) const FLAG_COMMAND: isize = 1;
pub(crate) const FLAG_SHIFT: isize = 2;
pub(crate) const FLAG_REPEAT: isize = 4;
pub(crate) const FLAG_RELEASE: isize = 8;
pub(crate) const FLAG_MODIFIER: isize = 16;
pub(crate) const FLAG_ALT_DOWN: isize = 32;
pub(crate) const FLAG_CONTROL_DOWN: isize = 64;
pub(crate) const FLAG_SUPER_DOWN: isize = 128;

static TARGET: AtomicIsize = AtomicIsize::new(0);
static OVERLAY: AtomicU8 = AtomicU8::new(0);
static ALT_DOWN: AtomicBool = AtomicBool::new(false);
/// Bumped as each overlay opens, so a key latched by the last one is not
/// mistaken for this one's.
static GENERATION: AtomicU32 = AtomicU32::new(0);
/// When the hook last heard Alt, which the Ruler's Alt poll defers to.
static LAST_ALT_HOOK: Mutex<Option<Instant>> = Mutex::new(None);
static MONITOR: Mutex<Option<Monitor>> = Mutex::new(None);

const START_TIMEOUT: Duration = Duration::from_secs(2);
const ALT_POLL_INTERVAL_MS: u32 = 8;
/// How long the Alt poll leaves Alt to the hook after the hook last heard it.
/// The Ruler takes Alt from the system, so `GetAsyncKeyState` never sees it
/// held, and Alt held is only heard again as it repeats: after the keyboard's
/// repeat delay, at most a second, then at the repeat rate. Longer than that
/// delay, so the poll does not read a held Alt as let go between repeats; the
/// hook's own release still ends it at once.
const HOOK_STATE_GRACE: Duration = Duration::from_millis(1_200);

struct Monitor {
  _registration: Registration,
  alt_poll: Option<AltPoll>,
}

struct AltPoll {
  thread_id: Arc<AtomicU32>,
  worker: JoinHandle<()>,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Overlay {
  Ruler = 1,
  TextRecognition = 2,
}

/// The Ruler's Alt poll: follows Alt through `GetAsyncKeyState` while the hook
/// has not heard it lately, on a thread of its own so compositor rendering
/// and pointer work can never delay it.
fn run_alt_poll(thread_id: Arc<AtomicU32>, ready: mpsc::Sender<Result<(), String>>) {
  thread_id.store(unsafe { GetCurrentThreadId() }, Ordering::Release);
  let timer = unsafe { SetTimer(None, 0, ALT_POLL_INTERVAL_MS, None) };
  if timer == 0 {
    let _ = ready.send(Err("Could not start the Ruler Alt monitor".to_owned()));
    return;
  }
  let _ = ready.send(Ok(()));

  let mut message = MSG::default();
  while unsafe { GetMessageW(&mut message, None, 0, 0) }.0 > 0 {
    if message.message == WM_TIMER && message.wParam.0 == timer {
      let recent_hook = LAST_ALT_HOOK
        .lock()
        .is_ok_and(|at| at.is_some_and(|at| at.elapsed() < HOOK_STATE_GRACE));
      if !recent_hook {
        let async_down = unsafe { GetAsyncKeyState(VK_MENU.0 as i32) } < 0;
        if async_down != ALT_DOWN.load(Ordering::Acquire) {
          let changed = post_alt_transition(async_down);
          if changed && async_down {
            cancel_alt_menu_activation();
          }
        }
      }
      continue;
    }
    unsafe {
      let _ = TranslateMessage(&message);
      DispatchMessageW(&message);
    }
  }
  let _ = unsafe { KillTimer(None, timer) };
}

fn start_alt_poll() -> Result<AltPoll, String> {
  let thread_id = Arc::new(AtomicU32::new(0));
  let poll_thread_id = Arc::clone(&thread_id);
  let (ready, did_start) = mpsc::channel();
  let worker = std::thread::Builder::new()
    .name("screenwide-ruler-alt".to_owned())
    .spawn(move || run_alt_poll(poll_thread_id, ready))
    .map_err(|error| format!("Could not start the Ruler Alt monitor: {error}"))?;
  let result = did_start
    .recv_timeout(START_TIMEOUT)
    .unwrap_or_else(|_| Err("The Ruler Alt monitor did not start in time".to_owned()));
  if let Err(error) = result {
    quit(&thread_id);
    let _ = worker.join();
    return Err(error);
  }
  Ok(AltPoll { thread_id, worker })
}

fn quit(thread_id: &AtomicU32) {
  let thread_id = thread_id.load(Ordering::Acquire);
  if thread_id != 0 {
    let _ = unsafe { PostThreadMessageW(thread_id, WM_QUIT, WPARAM(0), LPARAM(0)) };
  }
}

pub(crate) fn start(target: isize, overlay: Overlay) -> Result<(), String> {
  stop_current();
  TARGET.store(target, Ordering::Release);
  OVERLAY.store(overlay as u8, Ordering::Release);
  GENERATION.fetch_add(1, Ordering::AcqRel);
  let started = screenwide_keyboard_hook::take(Priority::Overlay, route)
    .map_err(|error| format!("Could not listen for overlay shortcuts: {error}"))
    .and_then(|registration| {
      let alt_poll = (overlay == Overlay::Ruler)
        .then(start_alt_poll)
        .transpose()?;
      Ok(Monitor {
        _registration: registration,
        alt_poll,
      })
    });
  let monitor = match started {
    Ok(monitor) => monitor,
    Err(error) => {
      TARGET.store(0, Ordering::Release);
      OVERLAY.store(0, Ordering::Release);
      return Err(error);
    }
  };
  let Ok(mut slot) = MONITOR.lock() else {
    TARGET.store(0, Ordering::Release);
    OVERLAY.store(0, Ordering::Release);
    return Err("The overlay keyboard monitor lock was poisoned".to_owned());
  };
  *slot = Some(monitor);
  Ok(())
}

pub(crate) fn stop(overlay: Overlay) {
  if active_overlay() == Some(overlay) {
    stop_current();
  }
}

fn stop_current() {
  TARGET.store(0, Ordering::Release);
  OVERLAY.store(0, Ordering::Release);
  if let Ok(mut slot) = MONITOR.lock() {
    if let Some(monitor) = slot.take() {
      if let Some(poll) = monitor.alt_poll {
        quit(&poll.thread_id);
        let _ = poll.worker.join();
      }
    }
  }
  if let Ok(mut at) = LAST_ALT_HOOK.lock() {
    *at = None;
  }
  ALT_DOWN.store(false, Ordering::Release);
}

/// Posts `vk` with `flags` to the overlay window, when there is one.
fn post(vk: u32, flags: isize) -> bool {
  let target = TARGET.load(Ordering::Acquire);
  target != 0
    && unsafe {
      PostMessageW(
        Some(HWND(target as *mut _)),
        OVERLAY_KEY_EVENT,
        WPARAM(vk as usize),
        LPARAM(flags),
      )
    }
    .is_ok()
}

#[cfg(test)]
mod tests;
