// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(in crate::windows::screenshot_region::native_osc_windows) unsafe extern "system" fn window_proc(
  hwnd: HWND,
  message: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  match message {
    WM_NCHITTEST => {
      let transparent = guard(|| hit_test(hwnd, lparam)).unwrap_or(true);
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
      // A middle-button pan owns the pointer until the button comes back up.
      if guard(|| ruler_pan_drag(hwnd, x, y)).unwrap_or(false) {
        return LRESULT(0);
      }
      let dragging = guard(|| pointer_drag_active(hwnd)).unwrap_or(false);
      dispatch(
        hwnd,
        if dragging { PHASE_DRAG } else { PHASE_MOVE },
        x,
        y,
        modifier_bits(wparam.0, false, alt_pressed()),
      );
      LRESULT(0)
    }
    WM_LBUTTONDOWN | WM_LBUTTONDBLCLK => {
      SetCapture(hwnd);
      let (x, y) = client_point(lparam);
      // A double click resets this display's viewport before the region
      // gesture ever sees it (`+input.m:645-649`).
      if message == WM_LBUTTONDBLCLK
        && guard(|| ruler_viewport(hwnd, 3, x, y, Point::default())).unwrap_or(false)
      {
        return LRESULT(0);
      }
      let alt = alt_pressed();
      dispatch(
        hwnd,
        PHASE_DOWN,
        x,
        y,
        modifier_bits(wparam.0, message == WM_LBUTTONDBLCLK, alt),
      );
      LRESULT(0)
    }
    // Right-click hides the label under the pointer, or toggles the labels of
    // whatever artifact is there (`processRulerLabelRightClick`).
    WM_RBUTTONDOWN => {
      let (x, y) = client_point(lparam);
      if guard(|| ruler_right_click(hwnd, x, y)).unwrap_or(false) {
        LRESULT(0)
      } else {
        DefWindowProcW(hwnd, message, wparam, lparam)
      }
    }
    WM_MBUTTONDOWN => {
      let (x, y) = client_point(lparam);
      if guard(|| ruler_pan_begin(hwnd, x, y)).unwrap_or(false) {
        SetCapture(hwnd);
        LRESULT(0)
      } else {
        DefWindowProcW(hwnd, message, wparam, lparam)
      }
    }
    WM_MBUTTONUP => {
      if guard(|| ruler_pan_end(hwnd)).unwrap_or(false) {
        let _ = ReleaseCapture();
        LRESULT(0)
      } else {
        DefWindowProcW(hwnd, message, wparam, lparam)
      }
    }
    // A notched mouse wheel zooms around the pointer without a modifier.
    // Precision deltas and horizontal wheel input remain two-axis panning;
    // Ctrl is retained as an explicit zoom override.
    WM_MOUSEWHEEL | WM_MOUSEHWHEEL => {
      let wheel_delta = (wparam.0 >> 16) as u16 as i16;
      let notches = f64::from(wheel_delta) / f64::from(WHEEL_DELTA);
      let zoom =
        message == WM_MOUSEWHEEL && vertical_wheel_zooms(wheel_delta, wparam.0 & MK_CONTROL != 0);
      let delta = if zoom {
        Point {
          x: (notches * WHEEL_ZOOM_EXPONENT).exp(),
          y: 0.0,
        }
      } else if message == WM_MOUSEHWHEEL {
        Point {
          // Win32's horizontal wheel sign describes content scrolling. AppKit
          // reports the trackpad gesture itself, so invert it to keep Ruler's
          // pan direction identical on both platforms.
          x: -notches * WHEEL_PAN_POINTS,
          y: 0.0,
        }
      } else {
        Point {
          x: 0.0,
          y: notches * WHEEL_PAN_POINTS,
        }
      };
      let (screen_x, screen_y) = screen_point(lparam);
      if guard(|| ruler_viewport_screen(hwnd, if zoom { 1 } else { 2 }, screen_x, screen_y, delta))
        .unwrap_or(false)
      {
        LRESULT(0)
      } else {
        DefWindowProcW(hwnd, message, wparam, lparam)
      }
    }
    WM_LBUTTONUP => {
      let (x, y) = client_point(lparam);
      dispatch(
        hwnd,
        PHASE_UP,
        x,
        y,
        modifier_bits(wparam.0, false, alt_pressed()),
      );
      let _ = ReleaseCapture();
      LRESULT(0)
    }
    WM_CANCELMODE | WM_CAPTURECHANGED => {
      dispatch(hwnd, PHASE_CANCEL, 0.0, 0.0, 0);
      LRESULT(0)
    }
    WM_SETCURSOR => {
      if guard(|| apply_cursor(hwnd)).unwrap_or(false) {
        LRESULT(1)
      } else {
        DefWindowProcW(hwnd, message, wparam, lparam)
      }
    }
    WM_TIMER => {
      match wparam.0 {
        ANIMATION_TIMER => guard(|| animation_frame(hwnd)),
        CONFIRM_TIMER => guard(|| confirm_expired(hwnd)),
        RULER_SETTLE_TIMER => guard(|| ruler_timer(hwnd, RULER_SETTLE_TIMER)),
        RULER_COPIED_TIMER => guard(|| ruler_timer(hwnd, RULER_COPIED_TIMER)),
        RULER_TOLERANCE_TIMER => guard(|| ruler_timer(hwnd, RULER_TOLERANCE_TIMER)),
        _ => Some(()),
      };
      LRESULT(0)
    }
    OVERLAY_KEY_EVENT => {
      let flags = lparam.0;
      let handled = if flags & 16 != 0 {
        let alt = flags & 32 != 0;
        guard(|| modifier_changed(hwnd, alt)).unwrap_or(false)
      } else if flags & 8 != 0 {
        guard(|| keyboard_release(hwnd, wparam)).unwrap_or(false)
      } else {
        guard(|| {
          overlay_keyboard_command(
            hwnd,
            wparam.0 as u16,
            (if flags & 64 != 0 { 2 } else { 0 })
              | (if flags & 128 != 0 { 1 } else { 0 })
              | (if flags & 32 != 0 { 4 } else { 0 })
              | (if flags & 2 != 0 { 8 } else { 0 }),
            flags & 4 != 0,
          )
        })
        .unwrap_or(false)
      };
      LRESULT(isize::from(handled))
    }
    RULER_CURSOR_EVENT => {
      let _ = guard(|| apply_cursor(hwnd));
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
      let handled = guard(|| keyboard_command(hwnd, wparam, lparam)).unwrap_or(false);
      if handled {
        LRESULT(0)
      } else {
        DefWindowProcW(hwnd, message, wparam, lparam)
      }
    }
    WM_KEYUP | WM_SYSKEYUP => {
      if guard(|| keyboard_release(hwnd, wparam)).unwrap_or(false) {
        LRESULT(0)
      } else {
        DefWindowProcW(hwnd, message, wparam, lparam)
      }
    }
    // Peers are top-level, so they are the surfaces Windows tells about a
    // topology change. The notification is coalesced: it arrives once per peer.
    WM_DISPLAYCHANGE | WM_DPICHANGED => {
      guard(|| state::notify_layout_changed_for_surface(hwnd));
      LRESULT(0)
    }
    _ => DefWindowProcW(hwnd, message, wparam, lparam),
  }
}

/// `window_proc` is an `extern "system"` callback: a panic that reaches it
/// cannot unwind and aborts the process.
pub(super) fn guard<T>(work: impl FnOnce() -> T) -> Option<T> {
  std::panic::catch_unwind(std::panic::AssertUnwindSafe(work))
    .inspect_err(|_| eprintln!("The Windows region OSC dropped an input after a panic"))
    .ok()
}

pub(super) fn hit_test(hwnd: HWND, lparam: LPARAM) -> bool {
  let Some(context) = state::context_for_surface(hwnd) else {
    return true;
  };
  let Ok(mut set) = context.surfaces.lock() else {
    return true;
  };
  let Some(surface) = set.find_mut(hwnd) else {
    return true;
  };
  let (screen_x, screen_y) = screen_point(lparam);
  let (x, y) = surface.screen_to_client(screen_x, screen_y);
  hit_transparent(
    surface.input_enabled,
    surface.visible,
    surface.exclusion_rect,
    surface.logical_point(x, y),
  )
}

pub(super) fn pointer_drag_active(hwnd: HWND) -> bool {
  state::context_for_surface(hwnd).is_some_and(|context| {
    let region_drag = context
      .surfaces
      .lock()
      .map(|mut set| {
        set
          .find_mut(hwnd)
          .is_some_and(|surface| surface.gesture_active)
      })
      .unwrap_or(false);
    let label_drag = context
      .ruler
      .lock()
      .map(|session| session.label_drag_active)
      .unwrap_or(false);
    region_drag || label_drag
  })
}
