// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a reference asks of the platform it runs on: its window's frame, the
//! native view the picture is drawn in, the pointer and its cursor, the
//! displays, and the picture's opacity. Every length is in points on the
//! desktop, from the primary display's top left, or in the window's points
//! from its top left. Placing, drawing, input and gliding are the engine's,
//! the same everywhere.

use screenwide_osc::geometry::{Point, Rect, Size};
use tauri::Position;

/// The cursor over a reference.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Cursor {
  Arrow,
  /// Over the picture, which moves it.
  Grab,
  /// Moving the picture.
  Grabbing,
  /// Over a handle, or resizing by it: the edges it moves (1 left, 2 right,
  /// 4 top, 8 bottom).
  Resize(u32),
}

pub(crate) trait ReferenceHost {
  /// Pixels to the point the picture is drawn at.
  fn scale(&self) -> f64;
  fn light_appearance(&self) -> bool;
  fn window_frame(&self) -> Rect;
  fn set_window_frame(&self, frame: Rect);
  /// Places the view the picture and its chrome are drawn in at `frame`, in
  /// the window's points, its layers at the host's scale.
  fn set_view_frame(&self, frame: Rect);
  fn pointer(&self) -> Point;
  /// The whole of the display `point` is on, else the main one.
  fn screen_at(&self, point: Point) -> Rect;
  /// The room for windows on the display `point` is on, else the main one:
  /// clear of the menu bar, the Dock and the taskbar.
  fn room_at(&self, point: Point) -> Rect;
  /// Where a picture `size` in size stands centred on the main display.
  fn centred(&self, size: Size) -> Point;
  /// `position`, as Tauri gives a place on the desktop, in points.
  fn desktop_point(&self, position: Position) -> Point;
  fn set_cursor(&self, cursor: Cursor);
  /// Shows the picture at `opacity`, from 0 to 1. Its chrome stays as it is.
  fn set_picture_opacity(&self, opacity: f64);
  /// Fades the picture and its chrome out over `seconds`, as the island
  /// tucks away before the window closes.
  fn fade_out(&self, seconds: f64);
  /// Lets go of the view, as the window goes.
  fn detach(&self);
}
