// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A caption piece turned in depth or flat: turned about its own middle, then
//! with its whole line about the line's, each seen in perspective from in
//! front of its middle, as a card is when it flips. The piece is drawn flat
//! as ever; the shader takes each pixel back onto it through the inverse of
//! the turn, which is a homography since the piece stays a flat card.

use super::arrows::PreviewGeometry;

type Matrix = [[f64; 3]; 3];

/// How far in front of what turns the eye is, in times its larger side: far
/// enough that a flip reads as a card turning, near enough to read as depth.
const PERSPECTIVE: f64 = 2.5;

fn product(a: Matrix, b: Matrix) -> Matrix {
  let mut out = [[0.0; 3]; 3];
  for (row, out_row) in out.iter_mut().enumerate() {
    for (column, cell) in out_row.iter_mut().enumerate() {
      *cell = (0..3).map(|at| a[row][at] * b[at][column]).sum();
    }
  }
  out
}

fn inverse(m: Matrix) -> Option<Matrix> {
  let cofactor = |row: usize, column: usize| {
    let (r0, r1) = ((row + 1) % 3, (row + 2) % 3);
    let (c0, c1) = ((column + 1) % 3, (column + 2) % 3);
    m[r0][c0] * m[r1][c1] - m[r0][c1] * m[r1][c0]
  };
  let determinant: f64 = (0..3)
    .map(|column| m[0][column] * cofactor(0, column))
    .sum();
  if determinant.abs() < 1e-9 {
    return None;
  }
  let mut out = [[0.0; 3]; 3];
  for (row, out_row) in out.iter_mut().enumerate() {
    for (column, cell) in out_row.iter_mut().enumerate() {
      *cell = cofactor(column, row) / determinant;
    }
  }
  Some(out)
}

/// `v` turned `degrees` about the horizontal axis, its top going away, then
/// the vertical, its right going away, then flat, clockwise; depth runs
/// away from the eye and down runs down the picture.
fn turned(v: [f64; 3], degrees: [f64; 3]) -> [f64; 3] {
  let [x, y, z] = degrees.map(f64::to_radians);
  let [v0, v1, v2] = v;
  let (v1, v2) = (v1 * x.cos() + v2 * x.sin(), -v1 * x.sin() + v2 * x.cos());
  let (v0, v2) = (v0 * y.cos() - v2 * y.sin(), v0 * y.sin() + v2 * y.cos());
  let (v0, v1) = (v0 * z.cos() - v1 * z.sin(), v0 * z.sin() + v1 * z.cos());
  [v0, v1, v2]
}

/// The homography taking a point of the picture's plane to where it lands
/// once the plane is turned `degrees` about `pivot` and seen from
/// `distance` in front of it.
fn turning(degrees: [f64; 3], pivot: [f64; 2], distance: f64) -> Matrix {
  let across = turned([1.0, 0.0, 0.0], degrees);
  let down = turned([0.0, 1.0, 0.0], degrees);
  let d = distance.max(1.0);
  let about = [
    [d * across[0], d * down[0], 0.0],
    [d * across[1], d * down[1], 0.0],
    [across[2], down[2], d],
  ];
  let to = [[1.0, 0.0, pivot[0]], [0.0, 1.0, pivot[1]], [0.0, 0.0, 1.0]];
  let from = [
    [1.0, 0.0, -pivot[0]],
    [0.0, 1.0, -pivot[1]],
    [0.0, 0.0, 1.0],
  ];
  product(to, product(about, from))
}

/// Where `point` lands through `m`, or none where it falls behind the eye.
fn mapped(m: Matrix, point: [f64; 2]) -> Option<[f64; 2]> {
  let w = m[2][0] * point[0] + m[2][1] * point[1] + m[2][2];
  (w > 1e-6).then(|| {
    [
      (m[0][0] * point[0] + m[0][1] * point[1] + m[0][2]) / w,
      (m[1][0] * point[0] + m[1][1] * point[1] + m[1][2]) / w,
    ]
  })
}

/// How a piece turns at one moment: about its own middle by `own`, then
/// with its line about `line`'s pivot, whose larger half across is its
/// reach, by its degrees.
pub(super) struct Turn {
  pub(super) own: [f64; 3],
  pub(super) line: Option<([f64; 3], [f64; 2], f64)>,
}

/// Where the canvas bounds of a geometry that draws nothing sit: low past
/// high, so they hold nothing and grow to whatever else they are joined to.
const EMPTY: [f32; 4] = [
  f32::INFINITY,
  f32::INFINITY,
  f32::NEG_INFINITY,
  f32::NEG_INFINITY,
];

/// The low and high corners of `geometry`'s box.
fn box_of(geometry: &PreviewGeometry) -> [[f64; 2]; 2] {
  [
    [f64::from(geometry.ax), f64::from(geometry.ay)],
    [f64::from(geometry.bx), f64::from(geometry.by)],
  ]
}

impl Turn {
  /// The turn of a piece whose box is `[low, high]`.
  fn matrix(&self, [low, high]: [[f64; 2]; 2]) -> Matrix {
    let centre = [(low[0] + high[0]) * 0.5, (low[1] + high[1]) * 0.5];
    let extent = (high[0] - low[0]).max(high[1] - low[1]);
    let own = turning(self.own, centre, PERSPECTIVE * extent);
    match self.line {
      Some((degrees, pivot, reach)) => {
        product(turning(degrees, pivot, PERSPECTIVE * 2.0 * reach), own)
      }
      None => own,
    }
  }
}

/// `geometry` turned as `turn` says: its way back from the canvas and the
/// bounds its box, `margin` larger all round, reaches written into its
/// `turn`. False, with no bounds, where the piece is edge on or any of it
/// is behind the eye, which draws nothing.
pub(super) fn turn_geometry(geometry: &mut PreviewGeometry, turn: &Turn, margin: f64) -> bool {
  geometry.turn[9..13].copy_from_slice(&EMPTY);
  let [low, high] = box_of(geometry);
  let m = turn.matrix([low, high]);
  let Some(back) = inverse(m) else {
    return false;
  };
  let [low, high] = [
    low.map(|value| value - margin),
    high.map(|value| value + margin),
  ];
  let corners = [low, [high[0], low[1]], [low[0], high[1]], high];
  let Some(landed) = corners
    .iter()
    .map(|corner| mapped(m, *corner))
    .collect::<Option<Vec<_>>>()
  else {
    return false;
  };
  // Scaled by a positive amount only, so the third coordinate keeps its
  // sign, and to about one, so single precision holds the ratios.
  let largest = back
    .iter()
    .flatten()
    .fold(0.0_f64, |most, entry| most.max(entry.abs()));
  for (slot, entry) in geometry.turn.iter_mut().zip(back.iter().flatten()) {
    *slot = (entry / largest) as f32;
  }
  let mut bounds = EMPTY.map(f64::from);
  for corner in landed {
    for axis in 0..2 {
      bounds[axis] = bounds[axis].min(corner[axis]);
      bounds[2 + axis] = bounds[2 + axis].max(corner[axis]);
    }
  }
  geometry.turn[9..13].copy_from_slice(&bounds.map(|value| value as f32));
  true
}

/// `geometry`'s bounds grown to hold `other`'s too.
pub(super) fn hold_bounds(geometry: &mut PreviewGeometry, other: &PreviewGeometry) {
  for axis in 0..2 {
    geometry.turn[9 + axis] = geometry.turn[9 + axis].min(other.turn[9 + axis]);
    geometry.turn[11 + axis] = geometry.turn[11 + axis].max(other.turn[11 + axis]);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// A card 100 across and down about [100, 50].
  const CARD: [[f64; 2]; 2] = [[50.0, 0.0], [150.0, 100.0]];

  fn turned(own: [f64; 3]) -> Matrix {
    Turn { own, line: None }.matrix(CARD)
  }

  #[test]
  fn turns_a_card_away_at_its_right_and_back_again() {
    // Unturned, every point lands where it was.
    assert_eq!(mapped(turned([0.0; 3]), [130.0, 20.0]), Some([130.0, 20.0]));
    // Half-way round about the vertical axis, its right edge goes away and
    // comes in nearer the middle than its left, and stands shorter.
    let half = turned([0.0, 45.0, 0.0]);
    let right = mapped(half, [150.0, 50.0]).unwrap();
    let left = mapped(half, [50.0, 50.0]).unwrap();
    assert!(right[0] - 100.0 < 100.0 - left[0]);
    let height = |x: f64| {
      let [top, bottom] = [[x, 0.0], [x, 100.0]].map(|point| mapped(half, point).unwrap());
      bottom[1] - top[1]
    };
    assert!(height(150.0) < 100.0 && 100.0 < height(50.0));
    // Edge on, it has no way back.
    assert!(inverse(turned([0.0, 90.0, 0.0])).is_none());
    // Turned flat a quarter, right of the middle goes below it.
    let below = mapped(turned([0.0, 0.0, 90.0]), [110.0, 50.0]).unwrap();
    assert!((below[0] - 100.0).abs() < 1e-9 && (below[1] - 60.0).abs() < 1e-9);
  }

  #[test]
  fn a_turned_geometry_takes_the_canvas_back_onto_its_card() {
    let mut geometry = PreviewGeometry {
      ax: 50.0,
      ay: 0.0,
      bx: 150.0,
      by: 100.0,
      ..Default::default()
    };
    let turn = Turn {
      own: [20.0, 50.0, 10.0],
      line: Some(([0.0, 0.0, 30.0], [400.0, 50.0], 350.0)),
    };
    assert!(turn_geometry(&mut geometry, &turn, 0.0));
    let back = geometry.turn.map(f64::from);
    let corner = [150.0, 100.0];
    let landed = mapped(turn.matrix(CARD), corner).unwrap();
    let [x, y, w] =
      [0, 3, 6].map(|row| back[row] * landed[0] + back[row + 1] * landed[1] + back[row + 2]);
    assert!(w > 0.0);
    assert!((x / w - corner[0]).abs() < 1e-3 && (y / w - corner[1]).abs() < 1e-3);
    // The bounds hold where the corner landed, to single precision.
    let near = 1e-3;
    assert!(back[9] - near <= landed[0] && landed[0] <= back[11] + near);
    assert!(back[10] - near <= landed[1] && landed[1] <= back[12] + near);
    // Edge on, it draws nothing and reaches nowhere.
    let edge = Turn {
      own: [0.0, 90.0, 0.0],
      line: None,
    };
    assert!(!turn_geometry(&mut geometry, &edge, 0.0));
    assert!(geometry.turn[9] > geometry.turn[11]);
  }
}
