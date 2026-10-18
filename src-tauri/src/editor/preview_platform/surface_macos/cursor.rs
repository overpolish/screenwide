// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The engine's cursors as AppKit draws them.

use objc2::rc::Retained;
use objc2::MainThreadMarker;
use objc2_app_kit::NSCursor;

use screenwide_workspace::editing::CursorKind;

unsafe extern "C" {
  /// The frame-resize cursor for `edges` (1 left, 2 right, 4 top, 8 bottom),
  /// shared with the region overlay in `osc/src/gpu/macos/region_cursor_macos.m`.
  fn screenwide_region_resize_cursor(edges: u32) -> *mut NSCursor;
}

fn resize(edges: u32) -> Option<Retained<NSCursor>> {
  // Safety: the helper returns an autoreleased cursor or null.
  unsafe { Retained::retain(screenwide_region_resize_cursor(edges)) }
}

pub(super) fn cursor(kind: CursorKind) -> Retained<NSCursor> {
  match kind {
    CursorKind::Arrow => NSCursor::arrowCursor(),
    CursorKind::Crosshair => NSCursor::crosshairCursor(),
    CursorKind::Move => NSCursor::closedHandCursor(),
    CursorKind::ResizeHorizontal => resize(1).unwrap_or_else(NSCursor::arrowCursor),
    CursorKind::ResizeVertical => resize(4).unwrap_or_else(NSCursor::arrowCursor),
    CursorKind::ResizeNesw => resize(2 | 4).unwrap_or_else(NSCursor::arrowCursor),
    CursorKind::ResizeNwse => resize(1 | 4).unwrap_or_else(NSCursor::arrowCursor),
    CursorKind::IBeam => NSCursor::IBeamCursor(),
  }
}

/// Shows `kind` now.
pub(super) fn set(kind: CursorKind, _mtm: MainThreadMarker) {
  cursor(kind).set();
}
