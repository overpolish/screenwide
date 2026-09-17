// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The overlay child's window procedure: hit testing, the pointer, the cursor
//! and the keys.

use super::*;

/// The child's own state: which display it draws, and whether it is letting
/// presses through to whatever is underneath.
///
/// It lives on the window rather than in the overlay's registry because
/// `SetWindowPos` and `DestroyWindow` reach this procedure from inside that
/// registry's lock. One word carries both, the display index above the flag.
pub(super) fn set_state(child: HWND, display: u32, click_through: bool) {
  let word = ((display as isize) << 1) | isize::from(click_through);
  unsafe { SetWindowLongPtrW(child, GWLP_USERDATA, word) };
}

fn state(child: HWND) -> (u32, bool) {
  let word = unsafe { GetWindowLongPtrW(child, GWLP_USERDATA) };
  ((word >> 1) as u32, word & 1 != 0)
}

/// True while this window holds the pointer, which is the only time a move or
/// a release belongs to a stroke.
fn drawing(child: HWND) -> bool {
  let captured = unsafe { GetCapture() };
  captured == child
}

/// One pointer step, in the global logical desktop points the cursor sidecar
/// reports in: the display's origin plus the client offset in its own points.
/// The child covers the host's client area, and the host covers the display,
/// so a client offset is already that display's physical pixels.
fn pointer(child: HWND, phase: u32, lparam: LPARAM) {
  let Some(display) = display(state(child).0) else {
    return;
  };
  let scale = if display.scale > 0.0 {
    display.scale
  } else {
    1.0
  };
  // Signed: a captured drag reports points outside the window it started in.
  let x = f64::from((lparam.0 & 0xffff) as u16 as i16);
  let y = f64::from(((lparam.0 >> 16) & 0xffff) as u16 as i16);
  input::pointer(
    phase,
    display.origin.0 + x / scale,
    display.origin.1 + y / scale,
  );
  redraw();
}

/// A key the overlay may have acted on. A redraw follows only when it did,
/// which is what the macOS side's return value decides too.
fn key(code: u16, modifiers: u32) -> bool {
  let Some(app) = APP.get() else {
    return false;
  };
  let handled = input::key(app, code, modifiers);
  if handled {
    redraw();
  }
  handled
}

/// The modifiers behind a key that arrived at this window directly.
fn pressed_modifiers() -> u32 {
  let down = |key: VIRTUAL_KEY| unsafe { GetKeyState(i32::from(key.0)) } < 0;
  (if down(VK_CONTROL) {
    input::MODIFIER_COMMAND
  } else {
    0
  }) | (if down(VK_SHIFT) {
    input::MODIFIER_SHIFT
  } else {
    0
  })
}

/// The modifiers the keyboard monitor posted with a key it consumed.
fn posted_modifiers(flags: isize) -> u32 {
  (if flags & FLAG_CONTROL_DOWN != 0 {
    input::MODIFIER_COMMAND
  } else {
    0
  }) | (if flags & FLAG_SHIFT != 0 {
    input::MODIFIER_SHIFT
  } else {
    0
  })
}

pub(super) extern "system" fn window_proc(
  hwnd: HWND,
  message: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  let click_through = state(hwnd).1;
  match message {
    // The overlay is pixels only while it shows annotations it no longer owns:
    // a press then belongs to whatever is underneath.
    WM_NCHITTEST if click_through => LRESULT(HTTRANSPARENT as isize),
    WM_NCHITTEST => LRESULT(HTCLIENT as isize),
    // Pressing the overlay never takes the foreground off the application the
    // user was working in, which is the one the cursor lease puts back.
    WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
    // DirectComposition owns every pixel of this window, so there is no
    // background for the shell to erase.
    WM_ERASEBKGND => LRESULT(1),
    // Nothing is taken, and no cursor is set, while the annotations are only
    // being shown.
    _ if click_through => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    WM_SETCURSOR => {
      unsafe { SetCursor(LoadCursorW(None, IDC_CROSS).ok()) };
      LRESULT(1)
    }
    WM_LBUTTONDOWN => {
      // Captured so a drag that leaves the display keeps reporting, and so the
      // release arrives even over another window.
      let _ = unsafe { SetCapture(hwnd) };
      pointer(hwnd, input::PHASE_DOWN, lparam);
      LRESULT(0)
    }
    WM_MOUSEMOVE => {
      if drawing(hwnd) {
        pointer(hwnd, input::PHASE_DRAG, lparam);
      }
      LRESULT(0)
    }
    WM_LBUTTONUP => {
      if drawing(hwnd) {
        pointer(hwnd, input::PHASE_UP, lparam);
        let _ = unsafe { ReleaseCapture() };
      }
      LRESULT(0)
    }
    // The capture going elsewhere, or the shell cancelling its modes, drops
    // the stroke in hand rather than leaving half a drag for the next press.
    WM_CAPTURECHANGED | WM_CANCELMODE => {
      input::cancel();
      redraw();
      LRESULT(0)
    }
    // The overlay is a drawing surface: every other thing the pointer can do
    // is swallowed rather than passed to the window underneath.
    WM_RBUTTONDOWN | WM_RBUTTONUP | WM_MBUTTONDOWN | WM_MBUTTONUP | WM_XBUTTONDOWN
    | WM_XBUTTONUP | WM_MOUSEWHEEL | WM_MOUSEHWHEEL => LRESULT(0),
    // Holding a modifier turns ordinary key messages into system ones, so both
    // families feed the same handler.
    WM_KEYDOWN | WM_SYSKEYDOWN => {
      if key(wparam.0 as u16, pressed_modifiers()) {
        LRESULT(0)
      } else {
        unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
      }
    }
    // The monitor's fallback for the keys that arrive while something else
    // holds focus. Releases and bare modifier transitions are not commands.
    OVERLAY_KEY_EVENT => {
      let flags = lparam.0;
      if flags & (FLAG_MODIFIER | FLAG_RELEASE) != 0 {
        return LRESULT(0);
      }
      LRESULT(isize::from(key(wparam.0 as u16, posted_modifiers(flags))))
    }
    _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
  }
}
