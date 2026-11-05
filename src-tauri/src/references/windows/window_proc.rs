// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The reference view's window procedure on Windows. The view reaches into
//! the shadow's room and over the island's top; only the picture and its
//! handles' reach are its own, and everywhere else it answers
//! `HTTRANSPARENT`, so the press falls to the webview under it, as the region
//! overlay's does. It never takes activation or the keyboard, which stay
//! with the page for the island and the arrow keys' nudges.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::OnceLock;

use screenwide_osc::geometry::Point;
use windows::core::{w, PCWSTR};
use windows::Win32::{
  Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM},
  Graphics::Gdi::ScreenToClient,
  UI::{
    Input::KeyboardAndMouse::{
      GetCapture, ReleaseCapture, SetCapture, TrackMouseEvent, TME_LEAVE, TRACKMOUSEEVENT,
    },
    WindowsAndMessaging::{
      DefWindowProcW, GetAncestor, GetForegroundWindow, PostMessageW, SetForegroundWindow, GA_ROOT,
      HTCLIENT, HTTRANSPARENT, MA_NOACTIVATE, WM_APP, WM_CANCELMODE, WM_CAPTURECHANGED,
      WM_DPICHANGED_AFTERPARENT, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEACTIVATE, WM_MOUSEMOVE,
      WM_NCHITTEST, WM_SETCURSOR, WNDCLASS_STYLES,
    },
  },
};

use super::super::engine;
use super::host::show_cursor;
use super::Shared;
use crate::app_windows::overlay_surface;

pub(super) const CLASS: PCWSTR = w!("ScreenwideReference");
/// `WM_MOUSELEAVE`, which the `windows` crate keeps with the common controls.
const WM_MOUSELEAVE: u32 = 0x02A3;
/// Takes up a display of another scale once nothing else is under way: the
/// change can come while the engine is placing the window.
const RESCALE: u32 = WM_APP + 0x351;

static CLASS_ATOM: OnceLock<u16> = OnceLock::new();

thread_local! {
  static VIEWS: RefCell<HashMap<isize, Rc<Shared>>> = RefCell::new(HashMap::new());
}

pub(super) fn register() {
  CLASS_ATOM
    .get_or_init(|| overlay_surface::register_class(CLASS, Some(window_proc), WNDCLASS_STYLES(0)));
}

pub(super) fn remember(view: HWND, shared: Rc<Shared>) {
  VIEWS.with(|views| views.borrow_mut().insert(view.0 as isize, shared));
}

pub(super) fn forget(view: HWND) {
  VIEWS.with(|views| views.borrow_mut().remove(&(view.0 as isize)));
}

fn shared(view: HWND) -> Option<Rc<Shared>> {
  VIEWS.with(|views| views.borrow().get(&(view.0 as isize)).cloned())
}

/// `lparam`'s signed pixel pair.
fn pixels(lparam: LPARAM) -> POINT {
  POINT {
    x: i32::from(lparam.0 as u16 as i16),
    y: i32::from((lparam.0 >> 16) as u16 as i16),
  }
}

/// A point in the view's pixels, in its points.
fn in_view(shared: &Shared, at: POINT) -> Point {
  let scale = shared.scale.get();
  Point {
    x: f64::from(at.x) / scale,
    y: f64::from(at.y) / scale,
  }
}

/// Ends the press under way, its pointer at `point` in the view, or taken
/// away for `None`, and takes up a display of another scale it came onto.
fn release(view: HWND, shared: &Shared, point: Option<Point>) {
  if !shared.pressed.replace(false) {
    return;
  }
  engine::up(&shared.label, point);
  if shared.rescale_pending.replace(false) {
    let _ = unsafe { PostMessageW(Some(view), RESCALE, WPARAM(0), LPARAM(0)) };
  }
}

/// `window_proc` is an `extern "system"` callback: a panic that reaches it
/// cannot unwind and aborts the whole process. Contain it to a dropped
/// message instead.
fn guard(work: impl FnOnce()) {
  if std::panic::catch_unwind(std::panic::AssertUnwindSafe(work)).is_err() {
    eprintln!("A reference dropped a message after a panic");
  }
}

unsafe extern "system" fn window_proc(
  view: HWND,
  message: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  let Some(shared) = shared(view) else {
    return unsafe { DefWindowProcW(view, message, wparam, lparam) };
  };
  match message {
    WM_NCHITTEST => {
      let mut at = pixels(lparam);
      let _ = unsafe { ScreenToClient(view, &mut at) };
      let mut takes = shared.pressed.get();
      guard(|| takes = takes || engine::takes(&shared.label, in_view(&shared, at)));
      LRESULT(if takes {
        HTCLIENT as isize
      } else {
        HTTRANSPARENT as isize
      })
    }
    WM_MOUSEACTIVATE => {
      // A press on the picture raises the reference as a press anywhere
      // else in it would, and leaves the keyboard with the page.
      let root = unsafe { GetAncestor(view, GA_ROOT) };
      if !root.is_invalid() && unsafe { GetForegroundWindow() } != root {
        let _ = unsafe { SetForegroundWindow(root) };
      }
      LRESULT(MA_NOACTIVATE as isize)
    }
    WM_MOUSEMOVE => {
      if shared.pressed.get() {
        guard(|| engine::dragged(&shared.label));
        return LRESULT(0);
      }
      if !shared.tracking.replace(true) {
        let mut track = TRACKMOUSEEVENT {
          cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
          dwFlags: TME_LEAVE,
          hwndTrack: view,
          dwHoverTime: 0,
        };
        let _ = unsafe { TrackMouseEvent(&mut track) };
      }
      let point = in_view(&shared, pixels(lparam));
      guard(|| engine::hover(&shared.label, Some(point)));
      LRESULT(0)
    }
    WM_MOUSELEAVE => {
      shared.tracking.set(false);
      if !shared.pressed.get() {
        guard(|| engine::hover(&shared.label, None));
      }
      LRESULT(0)
    }
    WM_LBUTTONDOWN => {
      unsafe { SetCapture(view) };
      shared.pressed.set(true);
      let point = in_view(&shared, pixels(lparam));
      guard(|| engine::down(&shared.label, point));
      LRESULT(0)
    }
    WM_LBUTTONUP => {
      let point = in_view(&shared, pixels(lparam));
      guard(|| release(view, &shared, Some(point)));
      if unsafe { GetCapture() } == view {
        let _ = unsafe { ReleaseCapture() };
      }
      LRESULT(0)
    }
    WM_CANCELMODE | WM_CAPTURECHANGED => {
      guard(|| release(view, &shared, None));
      LRESULT(0)
    }
    WM_SETCURSOR if (lparam.0 & 0xFFFF) as u32 == HTCLIENT => {
      show_cursor(shared.cursor.get());
      LRESULT(1)
    }
    WM_DPICHANGED_AFTERPARENT => {
      let _ = unsafe { PostMessageW(Some(view), RESCALE, WPARAM(0), LPARAM(0)) };
      LRESULT(0)
    }
    RESCALE => {
      if shared.pressed.get() {
        shared.rescale_pending.set(true);
      } else {
        guard(|| engine::rescale(&shared.label));
      }
      LRESULT(0)
    }
    _ => unsafe { DefWindowProcW(view, message, wparam, lparam) },
  }
}
