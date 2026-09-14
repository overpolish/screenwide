// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Windows wheel and auxiliary mouse hook integration.

use std::sync::{
  atomic::{AtomicBool, AtomicIsize, Ordering},
  Mutex,
};
use std::time::{Duration, Instant};

use windows::Win32::UI::Input::KeyboardAndMouse::GetDoubleClickTime;
use windows::Win32::{
  Foundation::{HWND, LPARAM, LRESULT, WPARAM},
  UI::WindowsAndMessaging::{
    CallNextHookEx, GetSystemMetrics, PostMessageW, SetWindowsHookExW, UnhookWindowsHookEx,
    HC_ACTION, HHOOK, LLMHF_INJECTED, MSLLHOOKSTRUCT, SM_CXDOUBLECLK, SM_CYDOUBLECLK, WH_MOUSE_LL,
    WM_APP, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEHWHEEL, WM_MOUSEMOVE, WM_MOUSEWHEEL,
  },
};

#[path = "wheel_hook/mouse.rs"]
mod mouse;
pub(super) const WM_GLIDE_WHEEL_X: u32 = WM_APP + 1;
pub(super) const WM_GLIDE_WHEEL_Y: u32 = WM_APP + 2;
pub(super) const WM_GLIDE_MOUSE_MOVE: u32 = WM_APP + 4;
pub(super) const WM_GLIDE_CANCEL: u32 = WM_APP + 6;

static TARGET: AtomicIsize = AtomicIsize::new(0);
static LAST_MOUSE_POINT: Mutex<Option<(i32, i32)>> = Mutex::new(None);
static LAST_LEFT_CLICK: Mutex<super::center::Matcher> = Mutex::new(super::center::Matcher::new());
static SWALLOW_LEFT_UP: AtomicBool = AtomicBool::new(false);
type CenterRequest = (isize, (i32, i32), super::InputKind);
static CENTER_REQUEST: Mutex<Option<CenterRequest>> = Mutex::new(None);
pub(super) const WM_GLIDE_CENTER: u32 = WM_APP + 5;

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

pub(super) fn hook_owns_mouse_motion() -> bool {
  let settings = super::native_settings::snapshot();
  [
    settings.mouse_modifier,
    settings.monitors_modifier,
    settings.spaces_modifier,
  ]
  .into_iter()
  .any(|control| control.is_mouse_button() && super::native_settings::is_down(control))
}

pub(super) fn take_center_request() -> Option<CenterRequest> {
  CENTER_REQUEST
    .lock()
    .ok()
    .and_then(|mut request| request.take())
}

pub(super) fn queue_center(hwnd: isize, point: (i32, i32), input: super::InputKind) -> bool {
  if let Ok(mut pending) = CENTER_REQUEST.lock() {
    *pending = Some((hwnd, point, input));
  } else {
    return false;
  }
  let target = TARGET.load(Ordering::Acquire);
  if target != 0 {
    let posted = unsafe {
      windows::Win32::UI::WindowsAndMessaging::PostMessageW(
        Some(HWND(target as *mut std::ffi::c_void)),
        WM_GLIDE_CENTER,
        WPARAM(0),
        LPARAM(0),
      )
      .is_ok()
    };
    if !posted {
      let _ = CENTER_REQUEST.lock().map(|mut pending| pending.take());
    }
    return posted;
  }
  false
}

unsafe extern "system" fn callback(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
  if code == HC_ACTION as i32 {
    let message = wparam.0 as u32;
    let packet = unsafe { &*(lparam.0 as *const MSLLHOOKSTRUCT) };
    if packet.flags & LLMHF_INJECTED != 0 && message != WM_MOUSEMOVE {
      return CallNextHookEx(None, code, wparam, lparam);
    }
    if message == WM_LBUTTONUP && SWALLOW_LEFT_UP.swap(false, Ordering::AcqRel) {
      return LRESULT(1);
    }
    if message == WM_LBUTTONDOWN {
      if super::session::monitor_mode() || super::spaces::active_input().is_some() {
        let target = TARGET.load(Ordering::Acquire);
        if target != 0
          && unsafe {
            PostMessageW(
              Some(HWND(target as *mut std::ffi::c_void)),
              WM_GLIDE_CANCEL,
              WPARAM(0),
              LPARAM(0),
            )
            .is_ok()
          }
        {
          super::navigation::cancel_until_release();
          SWALLOW_LEFT_UP.store(true, Ordering::Release);
          return LRESULT(1);
        }
      }
      let now = Instant::now();
      let candidate = (super::session::active_input() == Some(super::InputKind::Mouse))
        .then(super::session::target_hwnd)
        .flatten()
        .or_else(|| super::titlebar::cached_window_at(packet.pt).map(|hwnd| hwnd.0 as isize));
      let double_click = LAST_LEFT_CLICK.lock().ok().is_some_and(|mut last| {
        if candidate.is_none() {
          last.clear();
          return false;
        }
        let current = candidate.map(|hwnd| super::center::Click {
          at: now,
          point: (packet.pt.x, packet.pt.y),
          hwnd,
        });
        current.is_some_and(|current| {
          last.observe(
            current,
            Duration::from_millis(unsafe { GetDoubleClickTime() as u64 }),
            (
              (unsafe { GetSystemMetrics(SM_CXDOUBLECLK) }.max(1) / 2).max(1),
              (unsafe { GetSystemMetrics(SM_CYDOUBLECLK) }.max(1) / 2).max(1),
            ),
          )
        })
      });
      let app = super::APP.get();
      if double_click
        && candidate.is_some()
        && app.is_some_and(|app| super::center::allowed(app, super::InputKind::Mouse))
      {
        if let Some(hwnd) = candidate {
          super::center::trace(&format!(
            "queue hwnd={hwnd} point=({}, {})",
            packet.pt.x, packet.pt.y
          ));
          if queue_center(hwnd, (packet.pt.x, packet.pt.y), super::InputKind::Mouse) {
            SWALLOW_LEFT_UP.store(true, Ordering::Release);
            return LRESULT(1);
          }
        }
      }
    }
    if message == WM_MOUSEMOVE {
      forward_mouse_move(packet);
    }
    if let Some((button, pressed)) = mouse_button(message, packet.mouseData) {
      super::native_settings::observe(button, pressed);
      super::release_tracker::hook(button, pressed, packet.time);
      let configured = super::key_hook::configured(button);
      if configured && pressed {
        set_mouse_point(pressed.then_some((packet.pt.x, packet.pt.y)));
      }
      let posted = configured && super::key_hook::forward(button, pressed);
      let eligible = super::key_hook::suppresses(button);
      if super::key_hook::reserve(button, pressed, eligible, posted) {
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
        let posted = unsafe {
          PostMessageW(
            Some(HWND(target as *mut std::ffi::c_void)),
            forwarded,
            WPARAM(0),
            LPARAM(isize::from(delta)),
          )
        }
        .is_ok();
        if posted {
          let settings = super::native_settings::snapshot();
          let over_titlebar = super::titlebar::cached_window_at(packet.pt).is_some();
          let app = super::APP.get();
          let owned = super::session::monitor_mode()
            || super::spaces::active_input().is_some()
            || (over_titlebar
              && app.is_some_and(|app| !crate::capture_overlays::blocks_glide(app))
              && (super::native_settings::is_down(settings.monitors_modifier)
                || super::native_settings::is_down(settings.spaces_modifier))
              && settings.enabled
              && !crate::shortcuts::is_capturing()
              && (super::native_settings::is_down(settings.monitors_modifier)
                || super::spaces::supported()))
            || super::navigation::cancelled();
          if owned {
            return LRESULT(1);
          }
        }
      }
    }
  }
  unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

fn forward_mouse_move(packet: &MSLLHOOKSTRUCT) {
  if !hook_owns_mouse_motion() {
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

use mouse::{mouse_button, set_mouse_point};
