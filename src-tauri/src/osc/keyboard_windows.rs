// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Passive native-overlay keyboard monitor for Windows.
//!
//! The full-desktop OSC windows deliberately use `WS_EX_NOACTIVATE`; WebView2
//! therefore owns keyboard focus even while the native compositor owns pointer
//! input. macOS solves the same split with a local `NSEvent` monitor. Windows
//! uses one short-lived low-level hook and posts only keys recognised by the
//! active overlay back to the OSC window's owning UI thread.

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
  MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_QUIT, WM_SYSKEYDOWN, WM_SYSKEYUP, WM_TIMER,
};

use crate::windows::screenshot_region::native_osc_windows::OVERLAY_KEY_EVENT;

const FLAG_COMMAND: isize = 1;
const FLAG_SHIFT: isize = 2;
const FLAG_REPEAT: isize = 4;
const FLAG_RELEASE: isize = 8;
const FLAG_MODIFIER: isize = 16;
const FLAG_ALT_DOWN: isize = 32;

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
}

fn active_overlay() -> Option<Overlay> {
  match OVERLAY.load(Ordering::Acquire) {
    1 => Some(Overlay::Ruler),
    2 => Some(Overlay::TextRecognition),
    _ => None,
  }
}

thread_local! {
  static PRESSED: RefCell<[bool; 256]> = const { RefCell::new([false; 256]) };
  static LAST_ALT_HOOK: RefCell<Option<Instant>> = const { RefCell::new(None) };
}

fn post_alt_transition(down: bool) -> bool {
  let previous = ALT_DOWN.swap(down, Ordering::AcqRel);
  if previous == down {
    return false;
  }
  let target = TARGET.load(Ordering::Acquire);
  let mut flags = FLAG_MODIFIER;
  if down {
    flags |= FLAG_ALT_DOWN;
  } else {
    flags |= FLAG_RELEASE;
  }
  if target != 0 {
    let _ = unsafe {
      PostMessageW(
        Some(HWND(target as *mut _)),
        OVERLAY_KEY_EVENT,
        WPARAM(VK_MENU.0 as usize),
        LPARAM(flags),
      )
    };
  }
  true
}

/// A bare Alt press enters Win32's menu-activation mode even when the app has
/// no visible menu. An unassigned key while Alt remains held cancels that mode
/// without releasing Alt or stealing its Ruler meaning. This is only needed
/// for virtual-input paths that update async state without reaching our hook.
fn cancel_alt_menu_activation() {
  let key = KEYBDINPUT {
    wVk: VIRTUAL_KEY(0xe8),
    ..Default::default()
  };
  let inputs = [
    INPUT {
      r#type: INPUT_KEYBOARD,
      Anonymous: INPUT_0 { ki: key },
    },
    INPUT {
      r#type: INPUT_KEYBOARD,
      Anonymous: INPUT_0 {
        ki: KEYBDINPUT {
          dwFlags: KEYEVENTF_KEYUP,
          ..key
        },
      },
    },
  ];
  let _ = unsafe { SendInput(&inputs, size_of::<INPUT>() as i32) };
}

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
  if code == HC_ACTION as i32 {
    let data = unsafe { *(lparam.0 as *const KBDLLHOOKSTRUCT) };
    let down = matches!(wparam.0 as u32, WM_KEYDOWN | WM_SYSKEYDOWN);
    let up = matches!(wparam.0 as u32, WM_KEYUP | WM_SYSKEYUP);
    if down || up {
      let (command, shift, repeat) = update_pressed(data.vkCode, down);
      let overlay = active_overlay();
      let alt = overlay == Some(Overlay::Ruler) && matches!(data.vkCode, 0x12 | 0xa4 | 0xa5);
      if alt {
        LAST_ALT_HOOK.with(|at| *at.borrow_mut() = Some(Instant::now()));
        let _ = post_alt_transition(down);
        let target = TARGET.load(Ordering::Acquire);
        if target != 0 {
          return LRESULT(1);
        }
      }
      if overlay.is_some_and(|overlay| routes_to_overlay(overlay, data.vkCode, command, down, up)) {
        let target = TARGET.load(Ordering::Acquire);
        if target != 0 {
          let mut flags = 0;
          if command {
            flags |= FLAG_COMMAND;
          }
          if shift {
            flags |= FLAG_SHIFT;
          }
          if repeat {
            flags |= FLAG_REPEAT;
          }
          if up {
            flags |= FLAG_RELEASE;
          }
          let posted = unsafe {
            PostMessageW(
              Some(HWND(target as *mut _)),
              OVERLAY_KEY_EVENT,
              WPARAM(data.vkCode as usize),
              LPARAM(flags),
            )
          }
          .is_ok();
          if posted {
            // Match the macOS event monitor: an overlay command is consumed
            // and does not type into whichever app was behind it.
            return LRESULT(1);
          }
        }
      }
    }
  }
  unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

pub(crate) fn alt_pressed() -> bool {
  ALT_DOWN.load(Ordering::Acquire)
}

fn update_pressed(vk: u32, down: bool) -> (bool, bool, bool) {
  PRESSED.with(|pressed| {
    let mut pressed = pressed.borrow_mut();
    let index = (vk as usize).min(pressed.len() - 1);
    let repeat = down && pressed[index];
    pressed[index] = down;
    let command = pressed[0x11] || pressed[0xa2] || pressed[0xa3];
    let shift = pressed[0x10] || pressed[0xa0] || pressed[0xa1];
    (command, shift, repeat)
  })
}

fn routes_to_overlay(overlay: Overlay, vk: u32, command: bool, down: bool, up: bool) -> bool {
  if overlay == Overlay::TextRecognition {
    return down && command && matches!(vk, 0x41 | 0x43);
  }
  if matches!(vk, 0x12 | 0xa4 | 0xa5) {
    return down || up;
  }
  if up {
    return matches!(vk, 0x31 | 0x32 | 0x56 | 0x48 | 0x52);
  }
  if !down {
    return false;
  }
  if command {
    matches!(vk, 0x43 | 0x5a | 0x59)
  } else {
    matches!(
      vk,
      0x58 | 0x09 | 0x08 | 0x2e | 0x54 | 0x4d | 0x31 | 0x32 | 0x56 | 0x48 | 0x52
    )
  }
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
mod tests {
  use super::{routes_to_overlay, Overlay};

  #[test]
  fn routes_plain_latched_and_command_shortcuts() {
    assert!(routes_to_overlay(Overlay::Ruler, 0x31, false, true, false));
    assert!(routes_to_overlay(Overlay::Ruler, 0x31, false, false, true));
    assert!(routes_to_overlay(Overlay::Ruler, 0x43, true, true, false));
    assert!(!routes_to_overlay(Overlay::Ruler, 0x43, false, true, false));
    assert!(!routes_to_overlay(Overlay::Ruler, 0x41, true, true, false));
    assert!(routes_to_overlay(Overlay::Ruler, 0x12, false, true, false));
    assert!(routes_to_overlay(Overlay::Ruler, 0x12, false, false, true));
  }

  #[test]
  fn text_recognition_routes_only_control_a_and_control_c_down() {
    assert!(routes_to_overlay(
      Overlay::TextRecognition,
      0x41,
      true,
      true,
      false
    ));
    assert!(routes_to_overlay(
      Overlay::TextRecognition,
      0x43,
      true,
      true,
      false
    ));
    assert!(!routes_to_overlay(
      Overlay::TextRecognition,
      0x41,
      false,
      true,
      false
    ));
    assert!(!routes_to_overlay(
      Overlay::TextRecognition,
      0x41,
      true,
      false,
      true
    ));
  }
}
