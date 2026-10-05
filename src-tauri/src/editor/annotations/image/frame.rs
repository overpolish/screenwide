// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! An image's own selection frame, in display points: the box drawn along
//! the turned picture's four sides, its eight grips and its radius dot, and
//! the resize cursor each grip shows. Both platforms' chrome reads it - the
//! macOS one through `geometry.h` - so they place, draw and pick one frame.
//!
//! A grip reports the picture's own sides it moves, whatever the turn, so
//! the gesture reads it in the picture's own frame. Mirroring leaves the
//! frame alone: every side and corner of the picture is the same either way.

use super::model::{axes, from_local};
use crate::editor::annotations::box_gesture::{
  EDGE_BOTTOM, EDGE_LEFT, EDGE_RIGHT, EDGE_TOP, RADIUS_INSET_POINTS, RADIUS_TRAVEL,
};
use crate::editor::annotations::gesture::{BOX_HANDLES, RADIUS_HANDLE};
use crate::editor::annotations::AnnotationPoint;

/// The eight grips, clockwise from the picture's own top-left corner as the
/// layer selection orders its own, each with the sides it moves.
const GRIP_SIDES: [u32; 8] = [
  EDGE_LEFT | EDGE_TOP,
  EDGE_TOP,
  EDGE_RIGHT | EDGE_TOP,
  EDGE_RIGHT,
  EDGE_RIGHT | EDGE_BOTTOM,
  EDGE_BOTTOM,
  EDGE_LEFT | EDGE_BOTTOM,
  EDGE_LEFT,
];

/// The picture as its chrome sees it: its middle, its turn and its half
/// width and half height, all in display points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ImageFrame {
  pub(crate) center: AnnotationPoint,
  pub(crate) angle: f64,
  pub(crate) half_width: f64,
  pub(crate) half_height: f64,
}

impl ImageFrame {
  /// The point `x` along the picture's own right and `y` down it, as shares
  /// of its half width and half height from its middle.
  fn at(&self, x: f64, y: f64) -> AnnotationPoint {
    from_local(
      self.center,
      self.angle,
      AnnotationPoint {
        x: x * self.half_width,
        y: y * self.half_height,
      },
    )
  }

  /// Its four corners, clockwise from its own top-left.
  pub(crate) fn corners(&self) -> [AnnotationPoint; 4] {
    [
      self.at(-1.0, -1.0),
      self.at(1.0, -1.0),
      self.at(1.0, 1.0),
      self.at(-1.0, 1.0),
    ]
  }

  /// Where the radius dot sits: in from the picture's own top-left corner
  /// along its diagonal, further in the rounder its corners are, as a box's
  /// dot sits in from its corner. The twin of the gesture's reading, which
  /// turns a dragged dot back into a radius.
  pub(crate) fn radius_dot(&self, radius_percent: f64) -> AnnotationPoint {
    let shortest = (self.half_width.min(self.half_height) * 2.0).max(0.0);
    let inset =
      shortest * radius_percent.clamp(0.0, 50.0) / 100.0 * RADIUS_TRAVEL + RADIUS_INSET_POINTS;
    from_local(
      self.center,
      self.angle,
      AnnotationPoint {
        x: inset - self.half_width,
        y: inset - self.half_height,
      },
    )
  }

  /// Every grip with the handle it reports: the eight of the frame, then
  /// the radius dot. The frame's come first, so on a picture too small to
  /// keep them apart a corner wins over the dot.
  pub(crate) fn grips(&self, radius_percent: f64) -> [(AnnotationPoint, u32); 9] {
    let mut grips = [(self.radius_dot(radius_percent), RADIUS_HANDLE); 9];
    for (slot, sides) in GRIP_SIDES.into_iter().enumerate() {
      grips[slot] = (self.grip(sides), BOX_HANDLES + sides);
    }
    grips
  }

  /// The grip that moves `sides`: a corner, or the middle of a side.
  fn grip(&self, sides: u32) -> AnnotationPoint {
    let (x, y) = local_direction(sides);
    self.at(x, y)
  }
}

/// Which way a grip moving `sides` points in the picture's own frame: left
/// and up are negative.
fn local_direction(sides: u32) -> (f64, f64) {
  let along = |low: u32, high: u32| {
    if sides & low != 0 {
      -1.0
    } else if sides & high != 0 {
      1.0
    } else {
      0.0
    }
  };
  (along(EDGE_LEFT, EDGE_RIGHT), along(EDGE_TOP, EDGE_BOTTOM))
}

/// The upright sides whose resize cursor runs the way a grip moving `sides`
/// of a picture turned `angle` does on screen, so the platform cursors,
/// which are chosen by sides, follow the turn. A turn lands on whichever of
/// the four resize cursors is nearest, as the eighth of a turn either side
/// of each.
pub(crate) fn cursor_sides(sides: u32, angle: f64) -> u32 {
  let (x, y) = local_direction(sides);
  let (across, down) = axes(angle);
  let screen = (across[0] * x + down[0] * y, across[1] * x + down[1] * y);
  // A resize cursor points both ways, so only the line matters: fold the
  // direction into a half turn and take the nearest eighth.
  let half_turn = screen.1.atan2(screen.0).rem_euclid(std::f64::consts::PI);
  match (half_turn / std::f64::consts::FRAC_PI_4).round() as u32 % 4 {
    0 => EDGE_RIGHT,
    // Down and right on screen, where y grows downwards.
    1 => EDGE_LEFT | EDGE_TOP,
    2 => EDGE_BOTTOM,
    _ => EDGE_RIGHT | EDGE_TOP,
  }
}

/// An image's frame as the macOS chrome reads it: the four corners clockwise
/// from the picture's own top-left, then every grip and the handle it
/// reports. The twin of `AnnotationImageFrame` in `geometry.h`.
#[cfg(target_os = "macos")]
#[repr(C)]
pub struct ImageFrameRecord {
  corners: [[f32; 2]; 4],
  grips: [[f32; 2]; 9],
  handles: [u32; 9],
}

#[cfg(target_os = "macos")]
const _: () = assert!(std::mem::size_of::<ImageFrameRecord>() == 140);

/// An image's own frame and grips, in the display points its middle and
/// half extents are given in, turned `angle` radians clockwise and rounded
/// `radius` percent.
///
/// # Safety
/// `out` must point at one writable [`ImageFrameRecord`].
#[cfg(target_os = "macos")]
#[no_mangle]
pub unsafe extern "C" fn screenwide_image_frame(
  center_x: f32,
  center_y: f32,
  half_width: f32,
  half_height: f32,
  angle: f32,
  radius: f32,
  out: *mut ImageFrameRecord,
) {
  let Some(out) = (unsafe { out.as_mut() }) else {
    return;
  };
  let frame = ImageFrame {
    center: AnnotationPoint {
      x: f64::from(center_x),
      y: f64::from(center_y),
    },
    angle: f64::from(angle),
    half_width: f64::from(half_width),
    half_height: f64::from(half_height),
  };
  let pair = |point: AnnotationPoint| [point.x as f32, point.y as f32];
  out.corners = frame.corners().map(pair);
  for (slot, (grip, handle)) in frame.grips(f64::from(radius)).into_iter().enumerate() {
    out.grips[slot] = pair(grip);
    out.handles[slot] = handle;
  }
}

/// [`cursor_sides`], for the macOS chrome.
#[cfg(target_os = "macos")]
#[no_mangle]
pub extern "C" fn screenwide_image_cursor_sides(sides: u32, angle: f32) -> u32 {
  cursor_sides(sides, f64::from(angle))
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::f64::consts::{FRAC_PI_2, FRAC_PI_4};

  fn frame(angle: f64) -> ImageFrame {
    ImageFrame {
      center: AnnotationPoint { x: 100.0, y: 50.0 },
      angle,
      half_width: 40.0,
      half_height: 20.0,
    }
  }

  fn close(a: AnnotationPoint, b: (f64, f64)) -> bool {
    (a.x - b.0).abs() < 1e-9 && (a.y - b.1).abs() < 1e-9
  }

  #[test]
  fn an_upright_frame_is_the_picture_s_own_box() {
    let corners = frame(0.0).corners();
    assert!(close(corners[0], (60.0, 30.0)));
    assert!(close(corners[2], (140.0, 70.0)));
    let grips = frame(0.0).grips(0.0);
    assert_eq!(grips[1].1, BOX_HANDLES + EDGE_TOP);
    assert!(close(grips[1].0, (100.0, 30.0)));
    assert!(close(grips[3].0, (140.0, 50.0)));
  }

  #[test]
  fn a_turned_frame_turns_its_grips_and_keeps_their_sides() {
    // A quarter turn clockwise: the picture's own top now faces right.
    let grips = frame(FRAC_PI_2).grips(0.0);
    assert_eq!(grips[1].1, BOX_HANDLES + EDGE_TOP);
    assert!(close(grips[1].0, (120.0, 50.0)));
    // Its own top-left corner is now at the top right.
    assert!(close(grips[0].0, (120.0, 10.0)));
  }

  #[test]
  fn the_radius_dot_walks_in_along_the_picture_s_own_diagonal() {
    let square = |angle| ImageFrame {
      half_height: 40.0,
      ..frame(angle)
    };
    // No radius: the inset alone, in from the top-left corner.
    assert!(close(square(0.0).radius_dot(0.0), (70.0, 20.0)));
    // The whole radius: half the shorter side's travel further in.
    assert!(close(square(0.0).radius_dot(50.0), (92.0, 42.0)));
    assert!(close(square(0.0).radius_dot(80.0), (92.0, 42.0)));
    // Turned, it stays in its own corner.
    let turned = square(FRAC_PI_2).radius_dot(0.0);
    assert!(close(turned, (130.0, 20.0)));
  }

  #[test]
  fn a_grip_s_cursor_follows_the_turn() {
    let top = EDGE_TOP;
    assert_eq!(cursor_sides(top, 0.0), EDGE_BOTTOM);
    assert_eq!(cursor_sides(top, FRAC_PI_2), EDGE_RIGHT);
    assert_eq!(cursor_sides(top, FRAC_PI_4), EDGE_RIGHT | EDGE_TOP);
    let corner = EDGE_LEFT | EDGE_TOP;
    assert_eq!(cursor_sides(corner, 0.0), EDGE_LEFT | EDGE_TOP);
    assert_eq!(cursor_sides(corner, FRAC_PI_4), EDGE_BOTTOM);
    assert_eq!(cursor_sides(corner, FRAC_PI_2), EDGE_RIGHT | EDGE_TOP);
  }
}
