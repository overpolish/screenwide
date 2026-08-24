// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Win32 message translation for the native preview editor window.

use windows::Win32::{
  Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM},
  Graphics::Gdi::ScreenToClient,
  UI::{
    Input::KeyboardAndMouse::{GetKeyState, ReleaseCapture, SetCapture, VK_MENU},
    WindowsAndMessaging::{
      DefWindowProcW, GetAncestor, GetForegroundWindow, SetForegroundWindow, GA_ROOT, HTCLIENT,
      MA_NOACTIVATE, WM_CANCELMODE, WM_CAPTURECHANGED, WM_DESTROY, WM_LBUTTONDBLCLK,
      WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEACTIVATE, WM_MOUSEMOVE,
      WM_MOUSEWHEEL, WM_NCHITTEST, WM_SETCURSOR, WM_TIMER,
    },
  },
};

use super::{EditorWindow, Input};

const MK_LBUTTON_MASK: usize = 0x0001;
const MK_CONTROL_MASK: usize = 0x0008;
const MK_MBUTTON_MASK: usize = 0x0010;

fn option_pressed() -> bool {
  (unsafe { GetKeyState(VK_MENU.0 as i32) }) < 0
}

fn point(lparam: LPARAM) -> (f64, f64) {
  let x = (lparam.0 as u16 as i16) as f64;
  let y = ((lparam.0 >> 16) as u16 as i16) as f64;
  (x, y)
}

pub(super) unsafe extern "system" fn window_proc(
  hwnd: HWND,
  message: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  match message {
    WM_DESTROY => LRESULT(0),
    WM_TIMER if wparam.0 == EditorWindow::ACTION_TIMER => {
      dispatch(hwnd, Input::AnimateAction);
      LRESULT(0)
    }
    WM_NCHITTEST => LRESULT(HTCLIENT as isize),
    WM_MOUSEACTIVATE => {
      // The editor itself never takes activation or focus (keyboard input
      // stays with the webview), but a click on the workspace still has to
      // raise the export window like a click anywhere else in it would.
      let root = GetAncestor(hwnd, GA_ROOT);
      if !root.is_invalid() && GetForegroundWindow() != root {
        let _ = SetForegroundWindow(root);
      }
      LRESULT(MA_NOACTIVATE as isize)
    }
    WM_LBUTTONDOWN => {
      SetCapture(hwnd);
      let (x, y) = point(lparam);
      dispatch(
        hwnd,
        Input::Down {
          centered: option_pressed(),
          x,
          y,
          snapping: wparam.0 & MK_CONTROL_MASK != 0,
        },
      );
      LRESULT(0)
    }
    WM_LBUTTONDBLCLK => {
      let (x, y) = point(lparam);
      dispatch(hwnd, Input::DoubleClick { x, y });
      LRESULT(0)
    }
    WM_MOUSEMOVE => {
      let (x, y) = point(lparam);
      dispatch(
        hwnd,
        Input::Move {
          centered: option_pressed(),
          x,
          y,
          pressed: wparam.0 & (MK_LBUTTON_MASK | MK_MBUTTON_MASK) != 0,
          snapping: wparam.0 & MK_CONTROL_MASK != 0,
        },
      );
      LRESULT(0)
    }
    WM_LBUTTONUP => {
      let (x, y) = point(lparam);
      dispatch(hwnd, Input::Up { x, y });
      let _ = ReleaseCapture();
      LRESULT(0)
    }
    WM_MBUTTONDOWN => {
      SetCapture(hwnd);
      let (x, y) = point(lparam);
      dispatch(hwnd, Input::PanDown { x, y });
      LRESULT(0)
    }
    WM_MBUTTONUP => {
      let (x, y) = point(lparam);
      dispatch(hwnd, Input::PanUp { x, y });
      let _ = ReleaseCapture();
      LRESULT(0)
    }
    WM_CANCELMODE | WM_CAPTURECHANGED => {
      dispatch(hwnd, Input::Cancel);
      LRESULT(0)
    }
    WM_MOUSEWHEEL => {
      let (screen_x, screen_y) = point(lparam);
      let mut local = POINT {
        x: screen_x as i32,
        y: screen_y as i32,
      };
      let _ = ScreenToClient(hwnd, &mut local);
      let delta = ((wparam.0 >> 16) as u16 as i16) as f64 / 120.0;
      dispatch(
        hwnd,
        Input::Wheel {
          x: f64::from(local.x),
          y: f64::from(local.y),
          delta,
        },
      );
      LRESULT(0)
    }
    WM_SETCURSOR => {
      guard(|| super::super::refresh_editor_cursor(hwnd));
      LRESULT(1)
    }
    _ => DefWindowProcW(hwnd, message, wparam, lparam),
  }
}

/// `window_proc` is an `extern "system"` callback: a panic that reaches it
/// cannot unwind and aborts the whole process. Contain gesture bugs to a
/// logged, dropped input instead.
fn guard(work: impl FnOnce()) {
  if std::panic::catch_unwind(std::panic::AssertUnwindSafe(work)).is_err() {
    eprintln!("The Windows preview editor dropped an input after a panic");
  }
}

fn dispatch(hwnd: HWND, input: Input) {
  guard(|| super::super::handle_editor_input(hwnd, input));
}
