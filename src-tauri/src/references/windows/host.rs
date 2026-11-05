// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the reference engine asks of Windows, in Win32's terms.

use screenwide_osc::geometry::{Point, Rect, Size};
use tauri::Position;
use windows::Win32::{
  Foundation::RECT,
  UI::WindowsAndMessaging::{
    DestroyWindow, GetWindowRect, LoadCursorW, SetCursor, SetWindowPos, HWND_TOP, IDC_ARROW,
    IDC_SIZEALL, IDC_SIZENESW, IDC_SIZENS, IDC_SIZENWSE, IDC_SIZEWE, SWP_NOACTIVATE,
    SWP_NOOWNERZORDER, SWP_NOZORDER, SWP_SHOWWINDOW,
  },
};

use super::super::engine::{Cursor, ReferenceHost};
use super::desktop::{display_at, pointer, primary, to_pixels, to_points};
use super::{window_proc, WindowsHost};
use crate::app_windows::overlay_surface;

/// Shows `cursor`. Windows has no open or closed hand: moving takes the move
/// cursor, as the region overlay's does.
pub(super) fn show_cursor(cursor: Cursor) {
  let name = match cursor {
    Cursor::Arrow => IDC_ARROW,
    Cursor::Grab | Cursor::Grabbing => IDC_SIZEALL,
    // Left and top, or right and bottom, are one diagonal; the others the
    // other one.
    Cursor::Resize(5 | 10) => IDC_SIZENWSE,
    Cursor::Resize(6 | 9) => IDC_SIZENESW,
    Cursor::Resize(1 | 2) => IDC_SIZEWE,
    Cursor::Resize(_) => IDC_SIZENS,
  };
  if let Ok(cursor) = unsafe { LoadCursorW(None, name) } {
    unsafe { SetCursor(Some(cursor)) };
  }
}

impl ReferenceHost for WindowsHost {
  fn scale(&self) -> f64 {
    self.shared.scale.get()
  }

  fn light_appearance(&self) -> bool {
    overlay_surface::light_mode()
  }

  fn window_frame(&self) -> Rect {
    let mut frame = RECT::default();
    let _ = unsafe { GetWindowRect(self.top, &mut frame) };
    to_points(frame, self.scale())
  }

  fn set_window_frame(&self, frame: Rect) {
    let frame = to_pixels(frame, self.scale());
    let _ = unsafe {
      SetWindowPos(
        self.top,
        None,
        frame.left,
        frame.top,
        frame.right - frame.left,
        frame.bottom - frame.top,
        SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOOWNERZORDER,
      )
    };
  }

  /// The view goes over the webview each time, which keeps it above it.
  fn set_view_frame(&self, frame: Rect) {
    let scale = self.scale();
    let left = (frame.origin.x * scale).round() as i32;
    let top = (frame.origin.y * scale).round() as i32;
    // The view is as many pixels as the frames drawn in it (`engine::place`).
    let width = (frame.size.width * scale).ceil().max(1.0) as i32;
    let height = (frame.size.height * scale).ceil().max(1.0) as i32;
    let _ = unsafe {
      SetWindowPos(
        self.child,
        Some(HWND_TOP),
        left,
        top,
        width,
        height,
        SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_SHOWWINDOW,
      )
    };
  }

  fn pointer(&self) -> Point {
    pointer(self.scale())
  }

  fn screen_at(&self, point: Point) -> Rect {
    display_at(point, self.scale()).map_or_else(Rect::default, |display| {
      to_points(display.rcMonitor, self.scale())
    })
  }

  fn room_at(&self, point: Point) -> Rect {
    display_at(point, self.scale()).map_or_else(Rect::default, |display| {
      to_points(display.rcWork, self.scale())
    })
  }

  fn centred(&self, size: Size) -> Point {
    let room = primary().map_or_else(Rect::default, |display| {
      to_points(display.rcWork, self.scale())
    });
    Point {
      x: (room.origin.x + (room.size.width - size.width) / 2.0).round(),
      y: (room.origin.y + (room.size.height - size.height) / 2.0).round(),
    }
  }

  fn desktop_point(&self, position: Position) -> Point {
    match position {
      Position::Physical(at) => Point {
        x: f64::from(at.x) / self.scale(),
        y: f64::from(at.y) / self.scale(),
      },
      Position::Logical(at) => Point { x: at.x, y: at.y },
    }
  }

  fn set_cursor(&self, cursor: Cursor) {
    self.shared.cursor.set(cursor);
    show_cursor(cursor);
  }

  fn set_picture_opacity(&self, opacity: f64) {
    self.composition.set_picture_opacity(opacity as f32);
  }

  fn fade_out(&self, seconds: f64) {
    self.composition.fade_out(seconds);
  }

  fn detach(&self) {
    window_proc::forget(self.child);
    let _ = unsafe { DestroyWindow(self.child) };
  }
}
