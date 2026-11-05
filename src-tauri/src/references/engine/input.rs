// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The pointer over a reference's picture, as its view reports it in the
//! view's points: over it the handles show, a press on the picture moves it,
//! and a press on a handle resizes the picture in its shape, the window
//! following it.

use screenwide_osc::geometry::{Handle, Point, Rect};

use super::super::stage::resize_edges;
use super::place::{draw, picture_on_desktop, place};
use super::{with, Cursor, ReferenceHost};

fn cursor_for(handle: Option<Handle>) -> Cursor {
  handle
    .and_then(resize_edges)
    .map_or(Cursor::Grab, Cursor::Resize)
}

/// Whether the view takes the pointer at `point` in it: on the picture or
/// near enough a handle to take it.
pub(crate) fn takes(label: &str, point: Point) -> bool {
  with(label, |native| native.stage.takes(point)).unwrap_or(false)
}

/// The pointer is at `point` in the view, or has left it for `None`. The
/// view reaches into the shadow's room and over the island's top, so only
/// the picture and its handles count as on it.
pub(crate) fn hover(label: &str, point: Option<Point>) {
  with(label, |native| {
    if native.stage.resizing() {
      return;
    }
    let handle = point.and_then(|point| native.stage.handle_at(point));
    let hovered = handle.is_some();
    if hovered {
      native.host.set_cursor(cursor_for(handle));
    } else if native.stage.hovered {
      native.host.set_cursor(Cursor::Arrow);
    }
    if native.stage.hovered != hovered {
      native.stage.hovered = hovered;
      draw(native);
    }
  });
}

pub(crate) fn down(label: &str, point: Point) {
  with(label, |native| {
    let pointer = native.host.pointer();
    match native.stage.handle_at(point) {
      Some(Handle::Body) | None => {
        let picture = picture_on_desktop(native).origin;
        native.stage.begin_move(pointer, picture);
        native.host.set_cursor(Cursor::Grabbing);
      }
      Some(handle) => {
        let picture = picture_on_desktop(native);
        let middle = Point {
          x: picture.origin.x + picture.size.width / 2.0,
          y: picture.origin.y + picture.size.height / 2.0,
        };
        let screen = native.host.screen_at(middle);
        native.stage.begin_resize(handle, pointer, picture, screen);
      }
    }
  });
}

pub(crate) fn dragged(label: &str) {
  with(label, |native| {
    let pointer = native.host.pointer();
    if let Some(origin) = native.stage.move_to(pointer) {
      let size = native.stage.size;
      place(native, Rect { origin, size });
    } else if let Some(picture) = native.stage.resize_to(pointer) {
      place(native, picture);
    }
  });
}

/// The press is let go with the pointer at `point` in the view, or taken
/// away for `None`.
pub(crate) fn up(label: &str, point: Option<Point>) {
  with(label, |native| {
    native.stage.end_resize();
    native.stage.end_move();
    let handle = point.and_then(|point| native.stage.handle_at(point));
    native.host.set_cursor(match handle {
      Some(_) => cursor_for(handle),
      None => Cursor::Arrow,
    });
    draw(native);
  });
}
