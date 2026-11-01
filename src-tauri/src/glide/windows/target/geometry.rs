// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use windows::Win32::{
  Foundation::LPARAM,
  UI::WindowsAndMessaging::{SendMessageW, MINMAXINFO, WM_GETMINMAXINFO},
};

/// How far the window's invisible resize border extends past its visible
/// frame on each side. `GetWindowRect` and `SetWindowPos` speak in the outer
/// rectangle; Glide places the visible one, so two windows placed edge to
/// edge really touch and a window at the work area's edge really reaches it.
#[derive(Clone, Copy, Default)]
pub(super) struct FrameInsets {
  left: f64,
  top: f64,
  right: f64,
  bottom: f64,
}

pub(super) fn frame_insets(hwnd: HWND, outer: RECT) -> FrameInsets {
  let mut visible = RECT::default();
  let read = unsafe {
    DwmGetWindowAttribute(
      hwnd,
      DWMWA_EXTENDED_FRAME_BOUNDS,
      std::ptr::from_mut(&mut visible).cast(),
      std::mem::size_of::<RECT>() as u32,
    )
  };
  if read.is_err() {
    return FrameInsets::default();
  }
  FrameInsets {
    left: f64::from(visible.left - outer.left).max(0.0),
    top: f64::from(visible.top - outer.top).max(0.0),
    right: f64::from(outer.right - visible.right).max(0.0),
    bottom: f64::from(outer.bottom - visible.bottom).max(0.0),
  }
}

pub(super) fn visible_frame(outer: GlideFrame, insets: FrameInsets) -> GlideFrame {
  GlideFrame {
    x: outer.x + insets.left,
    y: outer.y + insets.top,
    width: (outer.width - insets.left - insets.right).max(0.0),
    height: (outer.height - insets.top - insets.bottom).max(0.0),
  }
}

pub(super) fn outer_frame(visible: GlideFrame, insets: FrameInsets) -> GlideFrame {
  GlideFrame {
    x: visible.x - insets.left,
    y: visible.y - insets.top,
    width: visible.width + insets.left + insets.right,
    height: visible.height + insets.top + insets.bottom,
  }
}

pub(super) fn frame(rect: RECT) -> GlideFrame {
  GlideFrame {
    x: f64::from(rect.left),
    y: f64::from(rect.top),
    width: f64::from(rect.right - rect.left),
    height: f64::from(rect.bottom - rect.top),
  }
}

/// The smallest visible size one of our windows accepts, from the minimum
/// track size it answers `WM_GETMINMAXINFO` with. Tao's subclass handles
/// `WM_WINDOWPOSCHANGING` without the default procedure, which is where
/// Windows would otherwise clamp a `SetWindowPos` to that minimum, so our own
/// placements are not held to it unless they ask. A window with no minimum
/// leaves the track size untouched and answers `None`.
pub(super) fn min_visible_size(hwnd: HWND, insets: FrameInsets) -> Option<(f64, f64)> {
  let mut info = MINMAXINFO::default();
  unsafe {
    SendMessageW(
      hwnd,
      WM_GETMINMAXINFO,
      None,
      Some(LPARAM(std::ptr::from_mut(&mut info) as isize)),
    )
  };
  let min = info.ptMinTrackSize;
  (min.x > 0 || min.y > 0).then(|| {
    (
      (f64::from(min.x) - insets.left - insets.right).max(0.0),
      (f64::from(min.y) - insets.top - insets.bottom).max(0.0),
    )
  })
}
