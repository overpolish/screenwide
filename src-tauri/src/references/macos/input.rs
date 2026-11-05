// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The pointer over a reference's picture: over it the handles show, a press
//! on the picture moves the window with it, and a press on a handle resizes
//! the picture in its shape, the window following it.

use objc2::rc::Retained;
use objc2::MainThreadMarker;
use objc2_app_kit::NSCursor;
use screenwide_osc::geometry::{Handle, Point, Rect};

use super::super::stage::resize_edges;
use super::{draw, picture_on_desktop, place, pointer, screen_at, with};

unsafe extern "C" {
  /// The frame-resize cursor for `edges` (1 left, 2 right, 4 top, 8 bottom),
  /// shared with the region overlay in `osc/src/gpu/macos/region_cursor_macos.m`.
  fn screenwide_region_resize_cursor(edges: u32) -> *mut NSCursor;
}

fn cursor_for(handle: Option<Handle>) -> Retained<NSCursor> {
  let resize = handle.and_then(resize_edges).and_then(|edges| {
    // Safety: the helper returns an autoreleased cursor or null.
    unsafe { Retained::retain(screenwide_region_resize_cursor(edges)) }
  });
  resize.unwrap_or_else(NSCursor::openHandCursor)
}

/// Whether the view takes the pointer at `point` in it: on the picture or
/// near enough a handle to take it.
pub(super) fn takes(label: &str, point: Point) -> bool {
  with(label, |native| native.stage.takes(point)).unwrap_or(false)
}

/// The pointer is at `point` in the view, or has left it for `None`. The
/// view reaches into the shadow's room and over the island's top, so only
/// the picture and its handles count as on it.
pub(super) fn hover(label: &str, point: Option<Point>) {
  let Some(mtm) = MainThreadMarker::new() else {
    return;
  };
  with(label, |native| {
    if native.stage.resizing() {
      return;
    }
    let handle = point.and_then(|point| native.stage.handle_at(point));
    let hovered = handle.is_some();
    if hovered {
      cursor_for(handle).set();
    } else if native.stage.hovered {
      NSCursor::arrowCursor().set();
    }
    if native.stage.hovered != hovered {
      native.stage.hovered = hovered;
      draw(mtm, native);
    }
  });
}

pub(super) fn down(label: &str, point: Point) {
  let Some(mtm) = MainThreadMarker::new() else {
    return;
  };
  with(label, |native| match native.stage.handle_at(point) {
    Some(Handle::Body) | None => {
      let picture = picture_on_desktop(mtm, native).origin;
      native.stage.begin_move(pointer(mtm), picture);
      NSCursor::closedHandCursor().set();
    }
    Some(handle) => {
      let picture = picture_on_desktop(mtm, native);
      let middle = Point {
        x: picture.origin.x + picture.size.width / 2.0,
        y: picture.origin.y + picture.size.height / 2.0,
      };
      native
        .stage
        .begin_resize(handle, pointer(mtm), picture, screen_at(mtm, middle));
    }
  });
}

pub(super) fn dragged(label: &str) {
  let Some(mtm) = MainThreadMarker::new() else {
    return;
  };
  with(label, |native| {
    let pointer = pointer(mtm);
    if let Some(origin) = native.stage.move_to(pointer) {
      let size = native.stage.size;
      place(mtm, native, Rect { origin, size });
    } else if let Some(picture) = native.stage.resize_to(pointer) {
      place(mtm, native, picture);
    }
  });
}

pub(super) fn up(label: &str, point: Point) {
  let Some(mtm) = MainThreadMarker::new() else {
    return;
  };
  with(label, |native| {
    native.stage.end_resize();
    native.stage.end_move();
    cursor_for(native.stage.handle_at(point)).set();
    draw(mtm, native);
  });
}
