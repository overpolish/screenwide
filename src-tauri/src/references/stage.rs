// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

//! A reference's state on screen, the same on every platform: the size its
//! picture is shown at, the island's size and parts, whether the pointer is
//! over it, and a resize under way. Resizing is the region selection's own
//! gesture, held to the picture's shape. Desktop lengths are in points,
//! top-left origin.

use screenwide_osc::geometry::{Handle, Monitor, Point, Rect, Size};
use screenwide_osc::gesture::{hit_test, Gesture, GestureKind};

use super::drawing::view_round;

/// How near a handle the pointer takes it, in points.
const HANDLE_REACH: f64 = 6.0;
/// The smallest a reference is resized to along either side, in points.
const SMALLEST_SIDE: f64 = 16.0;

pub(crate) struct Stage {
  /// The picture's size as shown.
  pub size: Size,
  /// The picture's size as taken, which Back to origin returns it to.
  pub true_size: Size,
  /// The island's size, as its page measures it.
  pub island: Size,
  /// Whether the pointer is over the picture or its handles.
  pub hovered: bool,
  /// Where the picture stands in the view it is drawn in, in points.
  pub in_view: Rect,
  resize: Option<Resize>,
  /// A move under way: where the pointer and the picture's corner were as
  /// it began, on the desktop. The picture is moved here rather than by the
  /// system's window drag, so the window is fitted round it at every step:
  /// the island goes over the picture as it nears the screen's foot.
  moving: Option<(Point, Point)>,
}

struct Resize {
  gesture: Gesture,
  /// What bounds the resize: the screen it started on, and as much of the
  /// picture as already stood past it, so a picture partly off the screen
  /// keeps its shape rather than being pressed into it.
  bounds: Rect,
}

impl Stage {
  pub(crate) fn new(size: Size) -> Self {
    let picture = Rect {
      origin: Point::default(),
      size,
    };
    let view = view_round(picture);
    Self {
      size,
      true_size: size,
      island: Size::default(),
      hovered: false,
      in_view: Rect {
        origin: Point {
          x: -view.origin.x,
          y: -view.origin.y,
        },
        size,
      },
      resize: None,
      moving: None,
    }
  }

  /// What is under `point` in the view: a handle, the picture (`Body`), or
  /// nothing.
  pub(crate) fn handle_at(&self, point: Point) -> Option<Handle> {
    hit_test(self.in_view, point, HANDLE_REACH)
  }

  /// What bounds a resize under way, on the desktop.
  pub(crate) fn resize_bounds(&self) -> Option<Rect> {
    self.resize.as_ref().map(|resize| resize.bounds)
  }

  pub(crate) fn resizing(&self) -> bool {
    self.resize.is_some()
  }

  /// Whether the reference takes the pointer at `point` in the view: on the
  /// picture, or near enough a handle to take it.
  pub(crate) fn takes(&self, point: Point) -> bool {
    self.handle_at(point).is_some()
  }

  /// Starts moving the picture from `pointer`, with its corner at `picture`,
  /// both on the desktop.
  pub(crate) fn begin_move(&mut self, pointer: Point, picture: Point) {
    self.moving = Some((pointer, picture));
  }

  /// Where the picture's corner goes with the pointer at `pointer`. None
  /// where no move is under way.
  pub(crate) fn move_to(&self, pointer: Point) -> Option<Point> {
    let (from, picture) = self.moving?;
    Some(Point {
      x: (picture.x + pointer.x - from.x).round(),
      y: (picture.y + pointer.y - from.y).round(),
    })
  }

  pub(crate) fn end_move(&mut self) {
    self.moving = None;
  }

  /// Starts resizing by `handle` from `pointer`, with the picture at
  /// `picture` on `screen`.
  pub(crate) fn begin_resize(
    &mut self,
    handle: Handle,
    pointer: Point,
    picture: Rect,
    screen: Rect,
  ) {
    let aspect = picture.size.width / picture.size.height;
    let bounds = screen.union(picture);
    self.resize = Some(Resize {
      gesture: Gesture::begin(
        GestureKind::Resizing(handle),
        local(pointer, bounds),
        local_rect(picture, bounds),
        (aspect.is_finite() && aspect > 0.0).then_some(aspect),
      ),
      bounds,
    });
  }

  /// Where the picture stands with the pointer at `pointer`, kept to its
  /// shape. None where no resize is under way or it would come out smaller
  /// than a reference is shown.
  pub(crate) fn resize_to(&mut self, pointer: Point) -> Option<Rect> {
    let resize = self.resize.as_mut()?;
    let bounds = resize.bounds;
    let rect = resize
      .gesture
      .update(local(pointer, bounds), Monitor { size: bounds.size }, false);
    if rect.size.width < SMALLEST_SIDE || rect.size.height < SMALLEST_SIDE {
      return None;
    }
    self.size = rect.size;
    Some(Rect {
      origin: Point {
        x: rect.origin.x + bounds.origin.x,
        y: rect.origin.y + bounds.origin.y,
      },
      size: rect.size,
    })
  }

  pub(crate) fn end_resize(&mut self) {
    self.resize = None;
  }
}

/// The frame-resize cursor's edges for `handle`: 1 start, 2 end, 4 top,
/// 8 bottom. None for the picture itself.
pub(crate) fn resize_edges(handle: Handle) -> Option<u32> {
  match handle {
    Handle::Body => None,
    Handle::West => Some(1),
    Handle::East => Some(2),
    Handle::North => Some(4),
    Handle::South => Some(8),
    Handle::NorthWest => Some(1 | 4),
    Handle::NorthEast => Some(2 | 4),
    Handle::SouthWest => Some(1 | 8),
    Handle::SouthEast => Some(2 | 8),
  }
}

fn local(point: Point, screen: Rect) -> Point {
  Point {
    x: point.x - screen.origin.x,
    y: point.y - screen.origin.y,
  }
}

fn local_rect(rect: Rect, screen: Rect) -> Rect {
  Rect {
    origin: local(rect.origin, screen),
    size: rect.size,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const SCREEN: Rect = Rect {
    origin: Point { x: 1000.0, y: 0.0 },
    size: Size {
      width: 1000.0,
      height: 800.0,
    },
  };

  fn stage() -> (Stage, Rect) {
    let picture = Rect::from_xywh(1200.0, 100.0, 200.0, 100.0);
    (Stage::new(picture.size), picture)
  }

  #[test]
  fn a_corner_resizes_the_picture_in_its_shape_from_the_opposite_corner() {
    let (mut stage, picture) = stage();
    stage.begin_resize(
      Handle::SouthEast,
      Point {
        x: 1400.0,
        y: 200.0,
      },
      picture,
      SCREEN,
    );
    let rect = stage
      .resize_to(Point {
        x: 1600.0,
        y: 220.0,
      })
      .unwrap();
    assert_eq!(rect.origin, picture.origin);
    assert_eq!(rect.size.width / rect.size.height, 2.0);
    assert!(rect.size.width > 200.0);
    assert_eq!(stage.size, rect.size);
    assert_eq!(stage.true_size, picture.size);
  }

  #[test]
  fn a_picture_partly_off_the_screen_keeps_its_shape_as_it_is_resized() {
    let mut stage = Stage::new(Size {
      width: 400.0,
      height: 200.0,
    });
    // Its right half stands past the screen's right edge, at 2000.
    let picture = Rect::from_xywh(1800.0, 100.0, 400.0, 200.0);
    stage.begin_resize(
      Handle::SouthWest,
      Point {
        x: 1800.0,
        y: 300.0,
      },
      picture,
      SCREEN,
    );
    let rect = stage
      .resize_to(Point {
        x: 1840.0,
        y: 280.0,
      })
      .unwrap();
    assert_eq!(rect.size.width / rect.size.height, 2.0);
    assert_eq!(rect.right(), picture.right());
  }

  #[test]
  fn a_resize_stops_short_of_too_small_a_picture() {
    let (mut stage, picture) = stage();
    stage.begin_resize(
      Handle::SouthEast,
      Point {
        x: 1400.0,
        y: 200.0,
      },
      picture,
      SCREEN,
    );
    assert!(stage
      .resize_to(Point {
        x: 1205.0,
        y: 104.0
      })
      .is_none());
    assert_eq!(stage.size, picture.size);
  }

  #[test]
  fn handles_are_found_round_the_picture_in_its_view() {
    let (stage, _) = stage();
    let frame = stage.in_view;
    assert_eq!(stage.handle_at(frame.origin), Some(Handle::NorthWest));
    let middle = Point {
      x: frame.origin.x + 100.0,
      y: frame.origin.y + 50.0,
    };
    assert_eq!(stage.handle_at(middle), Some(Handle::Body));
  }
}
