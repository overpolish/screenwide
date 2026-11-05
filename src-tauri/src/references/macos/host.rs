// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the reference engine asks of macOS, in AppKit's terms.

use objc2::rc::Retained;
use objc2_app_kit::NSCursor;
use objc2_foundation::{NSPoint, NSRect, NSSize};
use objc2_quartz_core::{CALayer, CATransaction};
use screenwide_osc::geometry::{Point, Rect, Size};
use tauri::Position;

use super::super::engine::{Cursor, ReferenceHost};
use super::desktop::{appkit_rect, centred, desktop_rect, pointer, room_at, screen_at};
use super::MacHost;

unsafe extern "C" {
  /// The frame-resize cursor for `edges` (1 left, 2 right, 4 top, 8 bottom),
  /// shared with the region overlay in `osc/src/gpu/macos/region_cursor_macos.m`.
  fn screenwide_region_resize_cursor(edges: u32) -> *mut NSCursor;
}

/// Runs `change` with implicit animations off, so a layer lands at once.
fn transaction(change: impl FnOnce()) {
  CATransaction::begin();
  CATransaction::setDisableActions(true);
  change();
  CATransaction::commit();
}

impl MacHost {
  /// `rect`, in the window's points from its top left, in its content
  /// view's own, which are not flipped: their origin is the window's foot.
  fn in_content(&self, rect: Rect) -> NSRect {
    let y = if self.content.isFlipped() {
      rect.origin.y
    } else {
      self.content.bounds().size.height - rect.origin.y - rect.size.height
    };
    NSRect::new(
      NSPoint::new(rect.origin.x, y),
      NSSize::new(rect.size.width, rect.size.height),
    )
  }
}

impl ReferenceHost for MacHost {
  const VIEW_SPANS_WINDOW: bool = false;

  fn scale(&self) -> f64 {
    self.window.backingScaleFactor()
  }

  fn light_appearance(&self) -> bool {
    crate::app_windows::platform::light_appearance(self.mtm)
  }

  fn window_frame(&self) -> Rect {
    desktop_rect(self.mtm, self.window.frame())
  }

  fn set_window_frame(&self, frame: Rect) {
    self
      .window
      .setFrame_display(appkit_rect(self.mtm, frame), true);
  }

  fn set_view_frame(&self, frame: Rect, _picture: Rect) {
    let frame = self.in_content(frame);
    self.view.setFrame(frame);
    let bounds = NSRect::new(NSPoint::new(0.0, 0.0), frame.size);
    let scale = self.scale();
    transaction(|| {
      for layer in [&self.picture_layer, &self.chrome_layer] {
        let layer: &CALayer = layer;
        layer.setFrame(bounds);
        layer.setContentsScale(scale);
      }
    });
  }

  fn pointer(&self) -> Point {
    pointer(self.mtm)
  }

  fn screen_at(&self, point: Point) -> Rect {
    screen_at(self.mtm, point)
  }

  fn room_at(&self, point: Point) -> Rect {
    room_at(self.mtm, point)
  }

  fn centred(&self, size: Size) -> Point {
    centred(self.mtm, size)
  }

  /// macOS lays its displays out in points, as a window's logical position
  /// is given, so either kind of position is already in them.
  fn desktop_point(&self, position: Position) -> Point {
    match position {
      Position::Logical(at) => Point { x: at.x, y: at.y },
      Position::Physical(at) => Point {
        x: f64::from(at.x),
        y: f64::from(at.y),
      },
    }
  }

  fn set_cursor(&self, cursor: Cursor) {
    let cursor: Retained<NSCursor> = match cursor {
      Cursor::Arrow => NSCursor::arrowCursor(),
      Cursor::Grab => NSCursor::openHandCursor(),
      Cursor::Grabbing => NSCursor::closedHandCursor(),
      // Safety: the helper returns an autoreleased cursor or null.
      Cursor::Resize(edges) => unsafe { Retained::retain(screenwide_region_resize_cursor(edges)) }
        .unwrap_or_else(NSCursor::openHandCursor),
    };
    cursor.set();
  }

  fn set_picture_opacity(&self, opacity: f64) {
    self.picture_layer.setOpacity(opacity as f32);
  }

  fn fade_out(&self, seconds: f64) {
    CATransaction::begin();
    CATransaction::setAnimationDuration(seconds);
    self.picture_layer.setOpacity(0.0);
    self.chrome_layer.setOpacity(0.0);
    CATransaction::commit();
  }

  fn detach(&self) {
    self.view.removeFromSuperview();
    // Safety: the observer was returned by `addObserverForName`.
    unsafe {
      objc2_foundation::NSNotificationCenter::defaultCenter()
        .removeObserver(self.on_screen.as_ref());
    }
  }
}
