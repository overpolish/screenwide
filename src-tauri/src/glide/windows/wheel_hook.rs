// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Observes the high-resolution wheel messages Windows synthesizes for
//! Precision Touchpads, including Apple's Windows PTP driver.

use std::sync::{
  atomic::{AtomicIsize, Ordering},
  Mutex,
};

use windows::Win32::{
  Foundation::{HWND, LPARAM, LRESULT, WPARAM},
  UI::WindowsAndMessaging::{
    CallNextHookEx, PostMessageW, SetWindowsHookExW, UnhookWindowsHookEx, HC_ACTION, HHOOK,
    LLMHF_INJECTED, MSLLHOOKSTRUCT, WH_MOUSE_LL, WM_APP, WM_MBUTTONDOWN, WM_MBUTTONUP,
    WM_MOUSEHWHEEL, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_XBUTTONDOWN, WM_XBUTTONUP,
  },
};

pub(super) const WM_GLIDE_WHEEL_X: u32 = WM_APP + 1;
pub(super) const WM_GLIDE_WHEEL_Y: u32 = WM_APP + 2;
pub(super) const WM_GLIDE_MOUSE_MOVE: u32 = WM_APP + 4;

static TARGET: AtomicIsize = AtomicIsize::new(0);
static LAST_MOUSE_POINT: Mutex<Option<(i32, i32)>> = Mutex::new(None);

pub(super) struct WheelHook(HHOOK);

impl Drop for WheelHook {
  fn drop(&mut self) {
    let _ = unsafe { UnhookWindowsHookEx(self.0) };
    TARGET.store(0, Ordering::Release);
  }
}

pub(super) fn install(target: HWND) -> Result<WheelHook, String> {
  TARGET.store(target.0 as isize, Ordering::Release);
  match unsafe { SetWindowsHookExW(WH_MOUSE_LL, Some(callback), None, 0) } {
    Ok(hook) => Ok(WheelHook(hook)),
    Err(error) => {
      TARGET.store(0, Ordering::Release);
      Err(error.to_string())
    }
  }
}

unsafe extern "system" fn callback(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
  if code == HC_ACTION as i32 {
    let message = wparam.0 as u32;
    let packet = unsafe { &*(lparam.0 as *const MSLLHOOKSTRUCT) };
    if message == WM_MOUSEMOVE {
      forward_mouse_move(packet);
    }
    if let Some((button, pressed)) = mouse_button(message, packet.mouseData) {
      super::native_settings::observe(button, pressed);
      let configured = super::key_hook::configured(button);
      let mouse_control = super::native_settings::snapshot()
        .mouse_modifier
        .matches(button);
      if mouse_control {
        set_mouse_point(pressed.then_some((packet.pt.x, packet.pt.y)));
      }
      if configured {
        super::key_hook::forward(button, pressed);
      }
      let suppressed = super::key_hook::suppresses(button);
      if suppressed {
        return LRESULT(1);
      }
    }
    let forwarded = match message {
      WM_MOUSEHWHEEL => Some(WM_GLIDE_WHEEL_X),
      WM_MOUSEWHEEL => Some(WM_GLIDE_WHEEL_Y),
      _ => None,
    };
    if let Some(forwarded) = forwarded {
      let target = TARGET.load(Ordering::Acquire);
      if target != 0 {
        let delta = ((packet.mouseData >> 16) as u16) as i16;
        let _ = unsafe {
          PostMessageW(
            Some(HWND(target as *mut std::ffi::c_void)),
            forwarded,
            WPARAM(0),
            LPARAM(isize::from(delta)),
          )
        };
      }
    }
  }
  unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

fn forward_mouse_move(packet: &MSLLHOOKSTRUCT) {
  let settings = super::native_settings::snapshot();
  if !settings.mouse_modifier.is_mouse_button()
    || !super::native_settings::is_down(settings.mouse_modifier)
  {
    return;
  }
  let point = (packet.pt.x, packet.pt.y);
  let previous = LAST_MOUSE_POINT
    .lock()
    .ok()
    .and_then(|mut last| last.replace(point));
  if packet.flags & LLMHF_INJECTED != 0 {
    return;
  }
  let Some(previous) = previous else {
    return;
  };
  let target = TARGET.load(Ordering::Acquire);
  if target == 0 || point == previous {
    return;
  }
  let _ = unsafe {
    PostMessageW(
      Some(HWND(target as *mut std::ffi::c_void)),
      WM_GLIDE_MOUSE_MOVE,
      WPARAM((point.0 - previous.0) as isize as usize),
      LPARAM((point.1 - previous.1) as isize),
    )
  };
}

fn set_mouse_point(point: Option<(i32, i32)>) {
  if let Ok(mut last) = LAST_MOUSE_POINT.lock() {
    *last = point;
  }
}

fn mouse_button(message: u32, data: u32) -> Option<(u32, bool)> {
  use super::control::{MOUSE_BACK, MOUSE_FORWARD, MOUSE_MIDDLE};

  match message {
    WM_MBUTTONDOWN => Some((MOUSE_MIDDLE, true)),
    WM_MBUTTONUP => Some((MOUSE_MIDDLE, false)),
    WM_XBUTTONDOWN | WM_XBUTTONUP => {
      let button = if data >> 16 == 1 {
        MOUSE_BACK
      } else {
        MOUSE_FORWARD
      };
      Some((button, message == WM_XBUTTONDOWN))
    }
    _ => None,
  }
}

#[cfg(test)]
mod tests {
  use super::super::control::{MOUSE_BACK, MOUSE_FORWARD, MOUSE_MIDDLE};
  use super::*;

  #[test]
  fn maps_middle_button_transitions() {
    assert_eq!(mouse_button(WM_MBUTTONDOWN, 0), Some((MOUSE_MIDDLE, true)));
    assert_eq!(mouse_button(WM_MBUTTONUP, 0), Some((MOUSE_MIDDLE, false)));
  }

  #[test]
  fn maps_xbutton_transitions() {
    assert_eq!(
      mouse_button(WM_XBUTTONDOWN, 1 << 16),
      Some((MOUSE_BACK, true))
    );
    assert_eq!(
      mouse_button(WM_XBUTTONUP, 2 << 16),
      Some((MOUSE_FORWARD, false))
    );
  }
}
