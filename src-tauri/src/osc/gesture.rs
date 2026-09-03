// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::geometry::{drawn_region, Handle, Monitor, Point, Rect};
use super::resize::resized_region;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GestureKind {
  Drawing,
  Moving,
  Resizing(Handle),
}
#[derive(Clone, Copy, Debug)]
pub struct Gesture {
  kind: GestureKind,
  start_pointer: Point,
  start_rect: Rect,
  aspect: Option<f64>,
  free: bool,
  last_pointer: Point,
  last_rect: Rect,
}
impl Gesture {
  pub fn begin(kind: GestureKind, pointer: Point, rect: Rect, aspect: Option<f64>) -> Self {
    Self {
      kind,
      start_pointer: pointer,
      start_rect: rect,
      aspect,
      free: false,
      last_pointer: pointer,
      last_rect: rect,
    }
  }
  pub fn kind(&self) -> GestureKind {
    self.kind
  }
  pub fn update(&mut self, pointer: Point, monitor: Monitor, free_aspect: bool) -> Rect {
    let resizing = matches!(self.kind, GestureKind::Resizing(_));
    if resizing && free_aspect != self.free {
      // The modifier transition creates a new baseline at the last presented
      // rectangle. In particular, releasing Shift makes the freeform shape
      // the immutable rectangle that uniform resize scales from here on.
      self.start_pointer = self.last_pointer;
      self.start_rect = self.last_rect;
      if !free_aspect {
        let ratio = self.last_rect.size.width / self.last_rect.size.height;
        self.aspect = (ratio.is_finite() && ratio > 0.0).then_some(ratio);
      }
    }
    let dx = pointer.x - self.start_pointer.x;
    let dy = pointer.y - self.start_pointer.y;
    let (result, resizing) = match self.kind {
      GestureKind::Drawing => (
        drawn_region(
          monitor,
          self.start_pointer,
          pointer,
          if free_aspect { None } else { self.aspect },
        ),
        None,
      ),
      GestureKind::Moving => (
        Rect {
          origin: super::geometry::Point {
            x: self.start_rect.origin.x + dx,
            y: self.start_rect.origin.y + dy,
          },
          size: self.start_rect.size,
        },
        None,
      ),
      GestureKind::Resizing(handle) => (
        resized_region(
          self.start_rect,
          handle,
          dx,
          dy,
          monitor,
          if free_aspect { None } else { self.aspect },
        ),
        Some(handle),
      ),
    };
    if resizing.is_some() {
      self.free = free_aspect;
      self.last_pointer = pointer;
      self.last_rect = result;
      result
    } else {
      result.clamp(monitor).snap()
    }
  }
  pub fn finish(&self, pointer: Point, monitor: Monitor, free_aspect: bool) -> Rect {
    let mut copy = *self;
    copy.update(pointer, monitor, free_aspect)
  }
  #[cfg(test)]
  pub fn cancel(&self) -> Rect {
    self.start_rect
  }
}

pub fn hit_test(rect: Rect, point: Point, radius: f64) -> Option<Handle> {
  let d = radius.max(0.0);
  let near = |a: f64, b: f64| (a - b).abs() <= d;
  let inside = point.x >= rect.origin.x
    && point.x <= rect.right()
    && point.y >= rect.origin.y
    && point.y <= rect.bottom();
  let within_x = point.x >= rect.origin.x - d && point.x <= rect.right() + d;
  let within_y = point.y >= rect.origin.y - d && point.y <= rect.bottom() + d;
  if near(point.x, rect.origin.x) && near(point.y, rect.origin.y) {
    Some(Handle::NorthWest)
  } else if near(point.x, rect.right()) && near(point.y, rect.origin.y) {
    Some(Handle::NorthEast)
  } else if near(point.x, rect.origin.x) && near(point.y, rect.bottom()) {
    Some(Handle::SouthWest)
  } else if near(point.x, rect.right()) && near(point.y, rect.bottom()) {
    Some(Handle::SouthEast)
  } else if near(point.y, rect.origin.y) && within_x {
    Some(Handle::North)
  } else if near(point.y, rect.bottom()) && within_x {
    Some(Handle::South)
  } else if near(point.x, rect.origin.x) && within_y {
    Some(Handle::West)
  } else if near(point.x, rect.right()) && within_y {
    Some(Handle::East)
  } else if inside {
    Some(Handle::Body)
  } else {
    None
  }
}
#[cfg(test)]
mod tests {
  use super::*;
  fn r() -> Rect {
    Rect {
      origin: Point { x: 10., y: 10. },
      size: super::super::geometry::Size {
        width: 30.,
        height: 20.,
      },
    }
  }
  fn m() -> Monitor {
    Monitor {
      size: super::super::geometry::Size {
        width: 100.,
        height: 80.,
      },
    }
  }
  #[test]
  fn move_clamps() {
    let mut g = Gesture::begin(GestureKind::Moving, Point { x: 10., y: 10. }, r(), None);
    assert_eq!(
      g.update(Point { x: -20., y: -20. }, m(), false).origin,
      Point { x: 0., y: 0. }
    );
  }
  #[test]
  fn cancel() {
    let g = Gesture::begin(GestureKind::Moving, Point { x: 0., y: 0. }, r(), None);
    assert_eq!(g.cancel(), r());
  }
  #[test]
  fn hit_priority() {
    assert_eq!(
      hit_test(r(), Point { x: 10., y: 10. }, 5.),
      Some(Handle::NorthWest)
    );
    assert_eq!(
      hit_test(r(), Point { x: 20., y: 20. }, 2.),
      Some(Handle::Body)
    );
  }
  #[test]
  fn shift_temporarily_frees_resize_aspect() {
    let mut g = Gesture::begin(
      GestureKind::Resizing(Handle::SouthEast),
      Point { x: 40., y: 20. },
      r(),
      Some(1.),
    );
    let release_pointer = Point { x: 60., y: 40. };
    let unlocked = g.update(release_pointer, m(), true);
    let relocked = g.update(release_pointer, m(), false);
    assert_eq!(unlocked, relocked);
    let continued = g.update(Point { x: 65., y: 40. }, m(), false);
    let unlocked_ratio = unlocked.size.width / unlocked.size.height;
    let continued_ratio = continued.size.width / continued.size.height;
    // Pixel snapping may move the ratio by less than one pixel per axis.
    assert!((unlocked_ratio - continued_ratio).abs() < 0.03);
    assert!((continued_ratio - 1.0).abs() > 0.1);
  }
  #[test]
  fn resize_boundaries_keep_opposite_edges() {
    let mut west = Gesture::begin(
      GestureKind::Resizing(Handle::West),
      Point { x: 10., y: 20. },
      r(),
      None,
    );
    assert_eq!(
      west.update(Point { x: -20., y: 20. }, m(), false).right(),
      40.
    );
    let mut east = Gesture::begin(
      GestureKind::Resizing(Handle::East),
      Point { x: 40., y: 20. },
      r(),
      None,
    );
    let east_rect = east.update(Point { x: 120., y: 20. }, m(), false);
    assert_eq!(east_rect.origin.x, 10.);
    assert_eq!(east_rect.right(), 100.);
    let mut north = Gesture::begin(
      GestureKind::Resizing(Handle::North),
      Point { x: 20., y: 10. },
      r(),
      None,
    );
    assert_eq!(
      north.update(Point { x: 20., y: -20. }, m(), false).bottom(),
      30.
    );
    let mut south = Gesture::begin(
      GestureKind::Resizing(Handle::South),
      Point { x: 20., y: 30. },
      r(),
      None,
    );
    let south_rect = south.update(Point { x: 20., y: 100. }, m(), false);
    assert_eq!(south_rect.origin.y, 10.);
    assert_eq!(south_rect.bottom(), 80.);
  }

  #[test]
  fn locked_side_resize_scales_both_dimensions() {
    let mut east = Gesture::begin(
      GestureKind::Resizing(Handle::East),
      Point { x: 40., y: 20. },
      r(),
      Some(1.5),
    );
    let resized = east.update(Point { x: 25., y: 20. }, m(), false);
    assert_eq!(resized.origin, Point { x: 10., y: 15. });
    assert_eq!(resized.size.width, 15.);
    assert_eq!(resized.size.height, 10.);
  }

  #[test]
  fn locked_resize_stops_at_monitor_edge_without_distorting() {
    let mut south_east = Gesture::begin(
      GestureKind::Resizing(Handle::SouthEast),
      Point { x: 40., y: 30. },
      r(),
      Some(1.5),
    );
    let resized = south_east.update(Point { x: 140., y: 130. }, m(), false);
    assert_eq!(resized.origin, Point { x: 10., y: 10. });
    assert_eq!(resized.size.width, 90.);
    assert_eq!(resized.size.height, 60.);
  }
}
