// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The overlay windows' procedure: Win32 messages decoded into the region
//! engine's events. macOS installed a global `NSEvent` monitor and returned
//! `nil` to swallow; Win32 gives the overlay the events it hit-tests into, so
//! pass-through is expressed as `HTTRANSPARENT` instead.

use windows::Win32::{
  Foundation::{HWND, LPARAM, LRESULT, WPARAM},
  UI::{
    Input::KeyboardAndMouse::{ReleaseCapture, SetCapture},
    WindowsAndMessaging::{
      DefWindowProcW, HTCLIENT, HTTRANSPARENT, MA_NOACTIVATE, WHEEL_DELTA, WM_CANCELMODE,
      WM_CAPTURECHANGED, WM_DISPLAYCHANGE, WM_DPICHANGED, WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDBLCLK,
      WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEACTIVATE, WM_MOUSEHWHEEL,
      WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_NCHITTEST, WM_RBUTTONDOWN, WM_SETCURSOR, WM_SYSKEYDOWN,
      WM_SYSKEYUP, WM_TIMER,
    },
  },
};

use super::super::region_osc::host::ScreenPoint;
use super::super::region_osc::input::{self as events, modifier_bits};
use super::{window_id, WindowsRegionHost, CURSOR_REFRESH_EVENT};
use crate::osc::keyboard_windows::OVERLAY_KEY_EVENT;

const MK_SHIFT: usize = 0x0004;
const MK_CONTROL: usize = 0x0008;

fn client_point(lparam: LPARAM) -> (f64, f64) {
  let x = f64::from(lparam.0 as u16 as i16);
  let y = f64::from((lparam.0 >> 16) as u16 as i16);
  (x, y)
}

fn screen_point(lparam: LPARAM) -> ScreenPoint {
  ScreenPoint {
    x: i32::from(lparam.0 as u16 as i16),
    y: i32::from((lparam.0 >> 16) as u16 as i16),
  }
}

/// The pointer message's held keys, as the engine packs them.
fn pointer_modifiers(wparam: WPARAM, double_click: bool) -> u8 {
  modifier_bits(
    wparam.0 & MK_SHIFT != 0,
    wparam.0 & MK_CONTROL != 0,
    double_click,
    WindowsRegionHost.alt_pressed(),
  )
}

pub(super) unsafe extern "system" fn window_proc(
  hwnd: HWND,
  message: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  let window = window_id(hwnd);
  let pass = || unsafe { DefWindowProcW(hwnd, message, wparam, lparam) };
  match message {
    WM_NCHITTEST => {
      let transparent = guard(|| events::hit_test(window, screen_point(lparam))).unwrap_or(true);
      LRESULT(if transparent {
        HTTRANSPARENT as isize
      } else {
        HTCLIENT as isize
      })
    }
    // The overlay never takes activation or focus; the webview keeps both.
    WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
    WM_MOUSEMOVE => {
      let (x, y) = client_point(lparam);
      guard(|| events::pointer_moved(window, x, y, pointer_modifiers(wparam, false)));
      LRESULT(0)
    }
    WM_LBUTTONDOWN | WM_LBUTTONDBLCLK => {
      if guard(|| events::take_press(window)).unwrap_or(false) {
        return LRESULT(0);
      }
      unsafe { SetCapture(hwnd) };
      let (x, y) = client_point(lparam);
      let double = message == WM_LBUTTONDBLCLK;
      guard(|| events::pointer_down(window, x, y, pointer_modifiers(wparam, double), double));
      LRESULT(0)
    }
    WM_RBUTTONDOWN => {
      let (x, y) = client_point(lparam);
      if guard(|| events::right_click(window, x, y)).unwrap_or(false) {
        LRESULT(0)
      } else {
        pass()
      }
    }
    WM_MBUTTONDOWN => {
      let (x, y) = client_point(lparam);
      if guard(|| events::pan_begin(window, x, y)).unwrap_or(false) {
        unsafe { SetCapture(hwnd) };
        LRESULT(0)
      } else {
        pass()
      }
    }
    WM_MBUTTONUP => {
      if guard(|| events::pan_end(window)).unwrap_or(false) {
        let _ = unsafe { ReleaseCapture() };
        LRESULT(0)
      } else {
        pass()
      }
    }
    WM_MOUSEWHEEL | WM_MOUSEHWHEEL => {
      let wheel_delta = (wparam.0 >> 16) as u16 as i16;
      let notches = f64::from(wheel_delta) / f64::from(WHEEL_DELTA);
      // Legacy mouse wheels report integral 120-unit notches; precision
      // touchpads report smaller deltas.
      let notched = wheel_delta != 0 && i32::from(wheel_delta) % WHEEL_DELTA as i32 == 0;
      let horizontal = message == WM_MOUSEHWHEEL;
      let control = wparam.0 & MK_CONTROL != 0;
      let screen = screen_point(lparam);
      if guard(|| events::wheel(window, screen, notches, horizontal, notched, control))
        .unwrap_or(false)
      {
        LRESULT(0)
      } else {
        pass()
      }
    }
    WM_LBUTTONUP => {
      let (x, y) = client_point(lparam);
      guard(|| events::pointer_up(window, x, y, pointer_modifiers(wparam, false)));
      let _ = unsafe { ReleaseCapture() };
      LRESULT(0)
    }
    WM_CANCELMODE | WM_CAPTURECHANGED => {
      guard(|| events::pointer_cancelled(window));
      LRESULT(0)
    }
    WM_SETCURSOR => {
      if guard(|| events::cursor_request(window)).unwrap_or(false) {
        LRESULT(1)
      } else {
        pass()
      }
    }
    WM_TIMER => {
      guard(|| events::timer(window, wparam.0));
      LRESULT(0)
    }
    OVERLAY_KEY_EVENT => {
      let flags = lparam.0;
      let handled = if flags & 16 != 0 {
        guard(|| events::alt_changed(window, flags & 32 != 0)).unwrap_or(false)
      } else if flags & 8 != 0 {
        guard(|| events::key_up(window, wparam.0 as u16)).unwrap_or(false)
      } else {
        let modifiers = (if flags & 64 != 0 { 2 } else { 0 })
          | (if flags & 128 != 0 { 1 } else { 0 })
          | (if flags & 32 != 0 { 4 } else { 0 })
          | (if flags & 2 != 0 { 8 } else { 0 });
        guard(|| events::key_down(window, wparam.0 as u16, modifiers, flags & 4 != 0))
          .unwrap_or(false)
      };
      LRESULT(isize::from(handled))
    }
    CURSOR_REFRESH_EVENT => {
      let _ = guard(|| events::cursor_request(window));
      LRESULT(0)
    }
    // The overlay remains `WS_EX_NOACTIVATE` for pointer input, but the Ruler
    // explicitly focuses this child while active so keyboard commands arrive
    // directly. The low-level monitor remains the fallback for focus changes.
    // Holding Alt changes ordinary key messages into system-key messages.
    // Feed both families into the same Ruler command path; bare Alt itself is
    // consumed here so DefWindowProc cannot enter menu-activation mode.
    WM_SYSKEYDOWN | WM_SYSKEYUP if matches!(wparam.0 as u32, 0x12 | 0xa4 | 0xa5) => LRESULT(0),
    WM_KEYDOWN | WM_SYSKEYDOWN => {
      // Bit 30 of `lparam` is set when this key-down is an auto-repeat.
      let repeat = lparam.0 & (1 << 30) != 0;
      let modifiers = WindowsRegionHost.keyboard_modifiers();
      if guard(|| events::key_down(window, wparam.0 as u16, modifiers, repeat)).unwrap_or(false) {
        LRESULT(0)
      } else {
        pass()
      }
    }
    WM_KEYUP | WM_SYSKEYUP => {
      if guard(|| events::key_up(window, wparam.0 as u16)).unwrap_or(false) {
        LRESULT(0)
      } else {
        pass()
      }
    }
    // Peers are top-level, so they are the surfaces Windows tells about a
    // topology change. The notification is coalesced: it arrives once per peer.
    WM_DISPLAYCHANGE | WM_DPICHANGED => {
      guard(|| events::display_changed(window));
      LRESULT(0)
    }
    _ => pass(),
  }
}

/// `window_proc` is an `extern "system"` callback: a panic that reaches it
/// cannot unwind and aborts the process.
fn guard<T>(work: impl FnOnce() -> T) -> Option<T> {
  std::panic::catch_unwind(std::panic::AssertUnwindSafe(work))
    .inspect_err(|_| eprintln!("The Windows region OSC dropped an input after a panic"))
    .ok()
}
