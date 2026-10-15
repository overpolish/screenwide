// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How a scene turns the screen in depth. The screen's pane, with everything
//! drawn on it, turns about its middle by Euler angles applied Rz * Ry * Rx,
//! in a frame with X to the right, Y up and Z toward the viewer, and is then
//! seen in perspective from in front of the canvas. In that frame a positive
//! X turn tips the top toward the viewer, a positive Y turn swings the right
//! side away, and a positive Z turn runs counter-clockwise on screen. A
//! scene's turn is blended as `recording-scene-blend.ts` blends it.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// How far in front of the canvas the eye sits, in lengths of the canvas's
/// longer side. Nearer makes a turned screen's near side loom larger.
const EYE_DISTANCE: f64 = 2.0;

/// The furthest any angle may run, in degrees: ten whole turns, which a scene
/// can spin through as it arrives.
const MAX_DEGREES: f64 = 3600.0;

/// A turn of the screen in degrees about each axis, upright where all three
/// are zero.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct SceneRotation {
  pub x: f64,
  pub y: f64,
  pub z: f64,
}

/// A rotation as columns: column `i` is where basis vector `i` lands.
pub type Rotation3 = [[f64; 3]; 3];

/// A projective map of the plane as rows, over homogeneous points.
pub type Homography = [[f64; 3]; 3];

impl SceneRotation {
  pub const UPRIGHT: Self = Self {
    x: 0.0,
    y: 0.0,
    z: 0.0,
  };

  pub fn is_upright(self) -> bool {
    self == Self::UPRIGHT
  }

  /// Whether every angle is a number a scene can hold.
  pub(crate) fn is_valid(self) -> bool {
    [self.x, self.y, self.z]
      .into_iter()
      .all(|angle| angle.is_finite() && angle.abs() <= MAX_DEGREES)
  }

  /// The turn `t` of the way from this one to `to`, each angle on its own,
  /// so a scene that spins a whole turn arrives spinning.
  pub fn toward(self, to: Self, t: f64) -> Self {
    let mix = |a: f64, b: f64| a + (b - a) * t;
    Self {
      x: mix(self.x, to.x),
      y: mix(self.y, to.y),
      z: mix(self.z, to.z),
    }
  }

  pub fn radians(self) -> [f64; 3] {
    [self.x, self.y, self.z].map(f64::to_radians)
  }

  pub fn from_radians([x, y, z]: [f64; 3]) -> Self {
    Self {
      x: x.to_degrees(),
      y: y.to_degrees(),
      z: z.to_degrees(),
    }
  }

  pub fn matrix(self) -> Rotation3 {
    euler_matrix(self.radians())
  }
}

/// The rotation Rz * Ry * Rx for Euler angles in radians.
pub fn euler_matrix([rx, ry, rz]: [f64; 3]) -> Rotation3 {
  let (sx, cx) = rx.sin_cos();
  let (sy, cy) = ry.sin_cos();
  let (sz, cz) = rz.sin_cos();
  [
    [cz * cy, sz * cy, -sy],
    [cz * sy * sx - sz * cx, sz * sy * sx + cz * cx, cy * sx],
    [cz * sy * cx + sz * sx, sz * sy * cx - cz * sx, cy * cx],
  ]
}

fn multiply(a: Homography, b: Homography) -> Homography {
  let mut out = [[0.0; 3]; 3];
  for (row, out_row) in out.iter_mut().enumerate() {
    for (column, value) in out_row.iter_mut().enumerate() {
      *value = (0..3).map(|k| a[row][k] * b[k][column]).sum();
    }
  }
  out
}

/// What carries a point of the canvas, where the upright screen would be
/// drawn, to where the screen turned by `rotation` about `centre` draws it.
/// `centre` is in pixels with Y down, and `extent` is the canvas's longer
/// side in the same pixels. Scaled so the middle keeps a weight of one; a
/// point turned behind the eye comes out with a weight of zero or less.
pub fn turned_homography(rotation: SceneRotation, centre: (f64, f64), extent: f64) -> Homography {
  let eye = EYE_DISTANCE * extent.max(1.0);
  let [x_axis, y_axis, _] = rotation.matrix();
  let (cx, cy) = centre;
  // Into the turn's own frame: about the middle, Y up.
  let into = [[1.0, 0.0, -cx], [0.0, -1.0, cy], [0.0, 0.0, 1.0]];
  // A point of the plane turned, then projected towards the eye; dividing
  // through by the eye's distance gives the middle its weight of one.
  let turn = [
    [x_axis[0], y_axis[0], 0.0],
    [x_axis[1], y_axis[1], 0.0],
    [-x_axis[2] / eye, -y_axis[2] / eye, 1.0],
  ];
  let out = [[1.0, 0.0, cx], [0.0, -1.0, cy], [0.0, 0.0, 1.0]];
  multiply(out, multiply(turn, into))
}

/// The inverse of `h`, or `None` where it flattens the plane to a line.
pub fn inverted(h: Homography) -> Option<Homography> {
  let cofactor = |row: usize, column: usize| {
    let (r0, r1) = ((row + 1) % 3, (row + 2) % 3);
    let (c0, c1) = ((column + 1) % 3, (column + 2) % 3);
    h[r0][c0] * h[r1][c1] - h[r0][c1] * h[r1][c0]
  };
  let determinant: f64 = (0..3)
    .map(|column| h[0][column] * cofactor(0, column))
    .sum();
  if !determinant.is_finite() || determinant.abs() < 1e-12 {
    return None;
  }
  let mut out = [[0.0; 3]; 3];
  for (row, out_row) in out.iter_mut().enumerate() {
    for (column, value) in out_row.iter_mut().enumerate() {
      *value = cofactor(column, row) / determinant;
    }
  }
  Some(out)
}

/// Where `h` carries `point`, or `None` where it lands behind the eye.
pub fn apply(h: Homography, (x, y): (f64, f64)) -> Option<(f64, f64)> {
  let w = h[2][0] * x + h[2][1] * y + h[2][2];
  (w > 1e-9).then(|| {
    (
      (h[0][0] * x + h[0][1] * y + h[0][2]) / w,
      (h[1][0] * x + h[1][1] * y + h[1][2]) / w,
    )
  })
}

#[cfg(test)]
mod tests;
