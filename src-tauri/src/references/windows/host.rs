// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

//! What the reference engine asks of Windows, in Win32's terms.

use screenwide_osc::geometry::{Point, Rect, Size};
use tauri::Position;
use windows::Win32::{
  Foundation::{POINT, RECT},
  UI::WindowsAndMessaging::{
    DestroyWindow, GetWindowRect, LoadCursorW, SetCursor, SetWindowPos, HWND_TOP, IDC_ARROW,
    IDC_SIZEALL, IDC_SIZENESW, IDC_SIZENS, IDC_SIZENWSE, IDC_SIZEWE, SWP_NOACTIVATE,
    SWP_NOOWNERZORDER, SWP_NOZORDER, SWP_SHOWWINDOW,
  },
};

use super::super::engine::{Cursor, ReferenceHost};
use super::super::fit::HANDLE_MARGIN;
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

impl WindowsHost {
  /// The window's frame on the desktop, in pixels.
  fn window_pixels(&self) -> RECT {
    let mut frame = RECT::default();
    let _ = unsafe { GetWindowRect(self.top, &mut frame) };
    frame
  }
}

impl ReferenceHost for WindowsHost {
  const VIEW_SPANS_WINDOW: bool = true;

  fn scale(&self) -> f64 {
    self.shared.scale.get()
  }

  fn light_appearance(&self) -> bool {
    overlay_surface::light_mode()
  }

  fn window_frame(&self) -> Rect {
    to_points(self.window_pixels(), self.scale())
  }

  fn set_window_frame(&self, frame: Rect) {
    let mut placed = self.placed.get();
    placed.origin = frame.origin;
    placed.window = to_pixels(frame, self.scale());
    self.placed.set(placed);
  }

  /// The view and the input pad over the picture and its handles' reach are
  /// placed from where they stand on the desktop, rounded there, as the
  /// engine rounds the picture's place in the view.
  fn set_view_frame(&self, frame: Rect, picture: Rect) {
    let scale = self.scale();
    let mut placed = self.placed.get();
    let (origin, window) = (placed.origin, placed.window);
    let in_window = |rect: Rect| {
      let on_desktop = to_pixels(
        Rect {
          origin: Point {
            x: origin.x + rect.origin.x,
            y: origin.y + rect.origin.y,
          },
          size: rect.size,
        },
        scale,
      );
      RECT {
        left: on_desktop.left - window.left,
        top: on_desktop.top - window.top,
        right: on_desktop.right - window.left,
        bottom: on_desktop.bottom - window.top,
      }
    };
    let view = in_window(frame);
    placed.pad = in_window(Rect::from_xywh(
      picture.origin.x - HANDLE_MARGIN,
      picture.origin.y - HANDLE_MARGIN,
      picture.size.width + HANDLE_MARGIN * 2.0,
      picture.size.height + HANDLE_MARGIN * 2.0,
    ));
    placed.view = POINT {
      x: view.left,
      y: view.top,
    };
    self.placed.set(placed);
  }

  /// The view's visuals move first and the window straight after, so a
  /// window moving with its picture shows it out of place for as short a
  /// time as can be. A window already where it is placed is left as it is:
  /// moving or resizing it, even to where it stands, has the webview lay
  /// itself out again.
  fn commit_placement(&self) {
    let placed = self.placed.get();
    self
      .composition
      .set_origin(placed.view.x as f32, placed.view.y as f32);
    let window = placed.window;
    if window != self.window_pixels() {
      let _ = unsafe {
        SetWindowPos(
          self.top,
          None,
          window.left,
          window.top,
          window.right - window.left,
          window.bottom - window.top,
          SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOOWNERZORDER,
        )
      };
    }
    let pad = placed.pad;
    self.shared.pad.set(POINT {
      x: pad.left - placed.view.x,
      y: pad.top - placed.view.y,
    });
    // Over the webview each time, which keeps it above it.
    let _ = unsafe {
      SetWindowPos(
        self.pad,
        Some(HWND_TOP),
        pad.left,
        pad.top,
        (pad.right - pad.left).max(1),
        (pad.bottom - pad.top).max(1),
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
    window_proc::forget(self.pad);
    let _ = unsafe { DestroyWindow(self.pad) };
  }
}
