// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::osc::geometry::{Point, Size};

pub const MINIMUM_ZOOM: f64 = 1.0;
pub const MAXIMUM_ZOOM: f64 = 16.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
  pub zoom: f64,
  /// Monitor-local source coordinate shown at the viewport's top-left.
  pub origin: Point,
}

impl Default for Viewport {
  fn default() -> Self {
    Self {
      zoom: MINIMUM_ZOOM,
      origin: Point::default(),
    }
  }
}

impl Viewport {
  pub fn screen_to_source(self, point: Point) -> Point {
    Point {
      x: self.origin.x + point.x / self.zoom,
      y: self.origin.y + point.y / self.zoom,
    }
  }

  pub fn zoom_at(&mut self, size: Size, anchor: Point, factor: f64) -> bool {
    if !factor.is_finite() || factor <= 0.0 || !size.valid() {
      return false;
    }
    let previous = *self;
    let source_anchor = self.screen_to_source(anchor);
    self.zoom = (self.zoom * factor).clamp(MINIMUM_ZOOM, MAXIMUM_ZOOM);
    self.origin = Point {
      x: source_anchor.x - anchor.x / self.zoom,
      y: source_anchor.y - anchor.y / self.zoom,
    };
    self.clamp(size);
    *self != previous
  }

  /// Moves the rendered content by a screen-space delta.
  pub fn pan_content(&mut self, size: Size, delta: Point) -> bool {
    if !delta.finite() || !size.valid() {
      return false;
    }
    let previous = *self;
    self.origin.x -= delta.x / self.zoom;
    self.origin.y -= delta.y / self.zoom;
    self.clamp(size);
    *self != previous
  }

  pub fn reset(&mut self) -> bool {
    let changed = *self != Self::default();
    *self = Self::default();
    changed
  }

  fn clamp(&mut self, size: Size) {
    self.zoom = self.zoom.clamp(MINIMUM_ZOOM, MAXIMUM_ZOOM);
    let visible_width = size.width / self.zoom;
    let visible_height = size.height / self.zoom;
    self.origin.x = self
      .origin
      .x
      .clamp(0.0, (size.width - visible_width).max(0.0));
    self.origin.y = self
      .origin
      .y
      .clamp(0.0, (size.height - visible_height).max(0.0));
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn size() -> Size {
    Size {
      width: 100.0,
      height: 80.0,
    }
  }

  #[test]
  fn zoom_keeps_the_source_under_the_pointer_fixed() {
    let mut viewport = Viewport::default();
    let anchor = Point { x: 75.0, y: 20.0 };
    let before = viewport.screen_to_source(anchor);
    assert!(viewport.zoom_at(size(), anchor, 2.0));
    assert_eq!(viewport.screen_to_source(anchor), before);
  }

  #[test]
  fn pan_and_zoom_never_expose_space_outside_the_monitor() {
    let mut viewport = Viewport::default();
    viewport.zoom_at(size(), Point { x: 50.0, y: 40.0 }, 4.0);
    viewport.pan_content(
      size(),
      Point {
        x: 10_000.0,
        y: 10_000.0,
      },
    );
    assert_eq!(viewport.origin, Point::default());
    viewport.pan_content(
      size(),
      Point {
        x: -10_000.0,
        y: -10_000.0,
      },
    );
    assert_eq!(viewport.origin, Point { x: 75.0, y: 60.0 });
  }

  #[test]
  fn reset_returns_only_this_viewport_to_identity() {
    let mut first = Viewport::default();
    let second = Viewport::default();
    first.zoom_at(size(), Point { x: 50.0, y: 40.0 }, 2.0);
    assert!(first.reset());
    assert_eq!(first, second);
  }
}
