// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Passive native-overlay keyboard monitor for Windows.
//!
//! The full-desktop OSC windows deliberately use `WS_EX_NOACTIVATE`; WebView2
//! therefore owns keyboard focus even while the native compositor owns pointer
//! input. macOS solves the same split with a local `NSEvent` monitor. Windows
//! uses one short-lived low-level hook and posts only keys recognised by the
//! active overlay back to the OSC window's owning UI thread.

#[path = "keyboard_windows/hooks.rs"]
mod hooks;
use hooks::active_overlay;
pub(crate) use hooks::alt_pressed;
use hooks::cancel_alt_menu_activation;
use hooks::hook_proc;
use hooks::post_alt_transition;
#[cfg(test)]
use hooks::routes_to_overlay;

use std::cell::RefCell;
use std::mem::size_of;
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicU32, AtomicU8, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Input::KeyboardAndMouse::{
  GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
  VIRTUAL_KEY, VK_MENU,
};
use windows::Win32::UI::WindowsAndMessaging::{
  CallNextHookEx, DispatchMessageW, GetMessageW, KillTimer, PostMessageW, PostThreadMessageW,
  SetTimer, SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx, HC_ACTION, KBDLLHOOKSTRUCT,
  MSG, WH_KEYBOARD_LL, WM_APP, WM_KEYDOWN, WM_KEYUP, WM_QUIT, WM_SYSKEYDOWN, WM_SYSKEYUP, WM_TIMER,
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
static MONITOR: Mutex<Option<Monitor>> = Mutex::new(None);

const START_TIMEOUT: Duration = Duration::from_secs(2);
const ALT_POLL_INTERVAL_MS: u32 = 8;
const HOOK_STATE_GRACE: Duration = Duration::from_millis(24);

struct Monitor {
  thread_id: Arc<AtomicU32>,
  worker: JoinHandle<()>,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Overlay {
  Ruler = 1,
  TextRecognition = 2,
  Annotate = 3,
}

thread_local! {
  static PRESSED: RefCell<[bool; 256]> = const { RefCell::new([false; 256]) };
  static CONSUMED: RefCell<[bool; 256]> = const { RefCell::new([false; 256]) };
  static LAST_ALT_HOOK: RefCell<Option<Instant>> = const { RefCell::new(None) };
}

fn run_monitor(
  thread_id: Arc<AtomicU32>,
  overlay: Overlay,
  ready: mpsc::Sender<Result<(), String>>,
) {
  // Low-level hook callbacks are delivered by posting to the installing
  // thread. This thread exists solely to pump those messages, so compositor
  // rendering and pointer work can never delay held-modifier transitions.
  thread_id.store(unsafe { GetCurrentThreadId() }, Ordering::Release);
  let hook = match unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), None, 0) } {
    Ok(hook) => hook,
    Err(error) => {
      let _ = ready.send(Err(format!(
        "Could not listen for overlay shortcuts: {error}"
      )));
      return;
    }
  };
  let timer = if overlay == Overlay::Ruler {
    unsafe { SetTimer(None, 0, ALT_POLL_INTERVAL_MS, None) }
  } else {
    0
  };
  if overlay == Overlay::Ruler && timer == 0 {
    let _ = unsafe { UnhookWindowsHookEx(hook) };
    let _ = ready.send(Err("Could not start the Ruler Alt monitor".to_owned()));
    return;
  }
  let _ = ready.send(Ok(()));

  let mut message = MSG::default();
  while unsafe { GetMessageW(&mut message, None, 0, 0) }.0 > 0 {
    if timer != 0 && message.message == WM_TIMER && message.wParam.0 == timer {
      let recent_hook = LAST_ALT_HOOK.with(|at| {
        at.borrow()
          .is_some_and(|at| at.elapsed() < HOOK_STATE_GRACE)
      });
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

  if timer != 0 {
    let _ = unsafe { KillTimer(None, timer) };
  }
  let _ = unsafe { UnhookWindowsHookEx(hook) };
  PRESSED.with(|pressed| pressed.borrow_mut().fill(false));
  CONSUMED.with(|consumed| consumed.borrow_mut().fill(false));
  LAST_ALT_HOOK.with(|at| *at.borrow_mut() = None);
  ALT_DOWN.store(false, Ordering::Release);
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
  let thread_id = Arc::new(AtomicU32::new(0));
  let hook_thread_id = Arc::clone(&thread_id);
  let (ready, did_start) = mpsc::channel();
  let worker = match std::thread::Builder::new()
    .name("screenwide-overlay-keyboard".to_owned())
    .spawn(move || run_monitor(hook_thread_id, overlay, ready))
  {
    Ok(worker) => worker,
    Err(error) => {
      TARGET.store(0, Ordering::Release);
      OVERLAY.store(0, Ordering::Release);
      return Err(format!(
        "Could not start the overlay keyboard monitor: {error}"
      ));
    }
  };
  let result = match did_start.recv_timeout(START_TIMEOUT) {
    Ok(result) => result,
    Err(_) => Err("The overlay keyboard monitor did not start in time".to_owned()),
  };
  if let Err(error) = result {
    TARGET.store(0, Ordering::Release);
    OVERLAY.store(0, Ordering::Release);
    quit(&thread_id);
    let _ = worker.join();
    return Err(error);
  }
  let Ok(mut monitor) = MONITOR.lock() else {
    TARGET.store(0, Ordering::Release);
    OVERLAY.store(0, Ordering::Release);
    quit(&thread_id);
    let _ = worker.join();
    return Err("The overlay keyboard monitor lock was poisoned".to_owned());
  };
  *monitor = Some(Monitor { thread_id, worker });
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
      quit(&monitor.thread_id);
      let _ = monitor.worker.join();
    }
  }
  ALT_DOWN.store(false, Ordering::Release);
}

#[cfg(test)]
#[path = "keyboard_windows/tests.rs"]
mod tests;
