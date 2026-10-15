// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The rotation gizmo: three great circles about a turned layer's middle, one
//! for each axis it turns about, each lying in the plane its axis turns. The
//! rings are the layer's own frame seen head on, so they tilt as the layer
//! does. Only a ring's near half can be taken hold of, the far half drawing
//! dimmed behind, and a drag turns the layer about the ring's axis as the
//! layer holds it rather than as the canvas does. Points are display points
//! from the gizmo's middle, Y running down; the turn's own frame runs Y up,
//! as `screenwide_scenes::rotation` describes.

mod ffi;

use screenwide_scenes::rotation::{euler_matrix, Rotation3};
use screenwide_scenes::SceneRotation;

/// The rings' radius over a large layer, in display points.
const MAX_RING_RADIUS: f64 = 48.0;

/// The rings' radius over a small one, which they may then overhang.
const MIN_RING_RADIUS: f64 = 24.0;

/// How much of a layer's shorter side the rings reach across from its
/// middle, so a small layer keeps room round them to be pressed and moved.
const RING_SHARE: f64 = 0.3;

/// How far from a ring a press still takes hold of it, in display points.
const HIT_REACH: f64 = 8.0;

/// How finely a press is measured against a ring: its polyline's segments.
const HIT_SAMPLES: usize = 192;

/// The step Shift holds a dragged axis to, in radians.
const SNAP: f64 = std::f64::consts::PI / 12.0;

/// The two columns spanning ring `axis`'s plane, in the order the ring runs:
/// its point at angle `t` is `cos t * u + sin t * v`. The axis's own column is
/// the ring's normal, so it drops out.
fn basis(m: Rotation3, axis: usize) -> ([f64; 3], [f64; 3]) {
  let u = if axis == 0 { 1 } else { 0 };
  let v = if axis == 2 { 1 } else { 2 };
  (m[u], m[v])
}

fn on_ring((u, v): ([f64; 3], [f64; 3]), radius: f64, t: f64) -> [f64; 3] {
  let (sin, cos) = t.sin_cos();
  [0, 1, 2].map(|i| radius * (cos * u[i] + sin * v[i]))
}

/// The radius of the rings over a layer whose shorter side is `short` display
/// points on screen.
pub fn ring_radius(short: f64) -> f64 {
  (short * RING_SHARE).clamp(MIN_RING_RADIUS, MAX_RING_RADIUS)
}

/// The ring a press takes hold of, and the angle along it the press lands
/// at, the nearest near-half ring within reach. Where rings cross, the
/// earlier axis keeps the point.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RingHit {
  pub axis: usize,
  pub angle: f64,
  /// The resize edges whose cursor runs the way a drag turns the ring there.
  pub edges: u32,
}

pub fn ring_at(rotation: SceneRotation, radius: f64, (x, y): (f64, f64)) -> Option<RingHit> {
  let m = rotation.matrix();
  let point = (x, -y);
  let mut best: Option<(f64, usize, f64)> = None;
  for axis in 0..3 {
    let ring = basis(m, axis);
    let mut previous = on_ring(ring, radius, 0.0);
    for index in 1..=HIT_SAMPLES {
      let t = std::f64::consts::TAU * index as f64 / HIT_SAMPLES as f64;
      let current = on_ring(ring, radius, t);
      let edge = (current[0] - previous[0], current[1] - previous[1]);
      let length = edge.0 * edge.0 + edge.1 * edge.1;
      let along = if length > 1e-12 {
        (((point.0 - previous[0]) * edge.0 + (point.1 - previous[1]) * edge.1) / length)
          .clamp(0.0, 1.0)
      } else {
        0.0
      };
      let distance =
        (point.0 - previous[0] - along * edge.0).hypot(point.1 - previous[1] - along * edge.1);
      let depth = previous[2] + along * (current[2] - previous[2]);
      let before = std::f64::consts::TAU * (index - 1) as f64 / HIT_SAMPLES as f64;
      if depth >= 0.0 && best.is_none_or(|(nearest, ..)| distance < nearest) {
        best = Some((distance, axis, before + along * (t - before)));
      }
      previous = current;
    }
  }
  let (_, axis, angle) = best.filter(|(distance, ..)| *distance <= HIT_REACH)?;
  let (tx, ty) = tangent(m, axis, angle);
  Some(RingHit {
    axis,
    angle,
    edges: cursor_edges((tx, -ty)).unwrap_or(1 | 4),
  })
}

/// The two directions spanning each ring, X's, Y's then Z's, as unit display
/// vectors with Y down and their depth toward the viewer third: ring `axis`'s
/// point at angle `t` is `radius * (cos t * u + sin t * v)`, on the far half
/// where its depth is negative. Rows `u` of the three rings, then `v`.
pub fn ring_basis(rotation: SceneRotation) -> [[f64; 3]; 6] {
  let m = rotation.matrix();
  let display = |[x, y, z]: [f64; 3]| [x, -y, z];
  let [(u0, v0), (u1, v1), (u2, v2)] = [0, 1, 2].map(|axis| basis(m, axis));
  [u0, u1, u2, v0, v1, v2].map(display)
}

/// The direction on screen, Y up, the point at `angle` on ring `axis` travels
/// as the angle grows; zero where an edge-on ring has none.
fn tangent(m: Rotation3, axis: usize, angle: f64) -> (f64, f64) {
  let (u, v) = basis(m, axis);
  let (sin, cos) = angle.sin_cos();
  let (x, y) = (-sin * u[0] + cos * v[0], -sin * u[1] + cos * v[1]);
  let length = x.hypot(y);
  if length > 1e-9 {
    (x / length, y / length)
  } else {
    (0.0, 0.0)
  }
}

/// The resize edges whose cursor runs along `direction`, Y down: left and
/// right across, top and bottom up and down, and the corners between.
fn cursor_edges((x, y): (f64, f64)) -> Option<u32> {
  if x == 0.0 && y == 0.0 {
    return None;
  }
  let eighth = (y.atan2(x).rem_euclid(std::f64::consts::PI) / std::f64::consts::FRAC_PI_4).round();
  Some(match eighth as u32 % 4 {
    0 => 2,
    1 => 2 | 8,
    2 => 8,
    _ => 1 | 8,
  })
}

/// One drag of a ring, measured from where it was pressed so a lost sample
/// never adds up: the turn then, in radians, the turn the last sample left,
/// which the next is held near, the way the pressed point travels, and the
/// radius of the ring it travels round.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RingDrag {
  pub axis: u32,
  pub press: [f64; 3],
  pub last: [f64; 3],
  /// Unit screen direction, Y up, of the pressed point as its ring turns.
  pub tangent: [f64; 2],
  pub radius: f64,
}

impl RingDrag {
  pub fn begin(rotation: SceneRotation, radius: f64, axis: usize, angle: f64) -> Self {
    let axis = axis.min(2);
    let press = rotation.radians();
    let (x, y) = tangent(rotation.matrix(), axis, angle);
    Self {
      axis: axis as u32,
      press,
      last: press,
      tangent: [x, y],
      radius,
    }
  }

  /// The turn a pointer `delta` display points from the press reaches. The
  /// travel along the pressed point's path, over the ring's radius, is how far
  /// the layer turns about the ring's axis as it held it at the press. `snap`
  /// lands the dragged axis on a whole step, rather than whole steps away from
  /// wherever the press happened to be.
  pub fn turned(&mut self, (dx, dy): (f64, f64), snap: bool) -> SceneRotation {
    let axis = self.axis.min(2) as usize;
    // The Y ring runs from X to Z, against the Y axis, so the same turn runs
    // its angle backwards.
    let sign = if axis == 1 { -1.0 } else { 1.0 };
    let mut delta =
      sign * (dx * self.tangent[0] - dy * self.tangent[1]) / self.radius.max(f64::EPSILON);
    if snap {
      let pressed = self.press[axis];
      delta = ((pressed + delta) / SNAP).round() * SNAP - pressed;
    }
    let turned = multiply(euler_matrix(self.press), axis_turn(axis, delta));
    self.last = decompose_near(turned, self.last);
    SceneRotation::from_radians(self.last)
  }
}

/// The turn seen of something turned `inner` within a frame turned `outer`,
/// as rings drawn over it show it: `inner` about its own axes, then `outer`.
pub fn composed(outer: SceneRotation, inner: SceneRotation) -> SceneRotation {
  let seen = multiply(outer.matrix(), inner.matrix());
  SceneRotation::from_radians(decompose_near(seen, inner.radians()))
}

/// The turn within a frame turned `outer` that is seen as `seen`, nearest
/// `near`: what a ring drag over something inside a turned frame leaves it
/// turned by, as the inverse of [`composed`].
pub fn within(outer: SceneRotation, seen: SceneRotation, near: SceneRotation) -> SceneRotation {
  let back = transposed(outer.matrix());
  SceneRotation::from_radians(decompose_near(
    multiply(back, seen.matrix()),
    near.radians(),
  ))
}

/// The inverse of a rotation.
fn transposed(m: Rotation3) -> Rotation3 {
  [0, 1, 2].map(|column| [0, 1, 2].map(|row| m[row][column]))
}

/// A turn of `angle` about one axis.
fn axis_turn(axis: usize, angle: f64) -> Rotation3 {
  let (sin, cos) = angle.sin_cos();
  let mut m = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
  let (a, b) = ((axis + 1) % 3, (axis + 2) % 3);
  m[a][a] = cos;
  m[a][b] = sin;
  m[b][a] = -sin;
  m[b][b] = cos;
  m
}

/// `a` then `b`, as columns.
fn multiply(a: Rotation3, b: Rotation3) -> Rotation3 {
  let mut out = [[0.0; 3]; 3];
  for (column, out_column) in out.iter_mut().enumerate() {
    for (row, value) in out_column.iter_mut().enumerate() {
      *value = (0..3).map(|k| a[k][row] * b[column][k]).sum();
    }
  }
  out
}

fn wrap(angle: f64) -> f64 {
  angle - std::f64::consts::TAU * (angle / std::f64::consts::TAU).round()
}

/// The Euler angles of `m` under Rz * Ry * Rx nearest `anchor`. Every turn has
/// two sets, so the one nearer keeps a sweep through a quarter turn about Y
/// continuous, and each angle is unwrapped to within half a turn of the
/// anchor's, so whole turns add up rather than wrapping.
fn decompose_near(m: Rotation3, anchor: [f64; 3]) -> [f64; 3] {
  let y = (-m[0][2]).clamp(-1.0, 1.0).asin();
  let primary = if y.cos().abs() > 1e-6 {
    [m[1][2].atan2(m[2][2]), y, m[0][1].atan2(m[0][0])]
  } else {
    // Gimbal lock: only X and Z together are known, so Z is held at zero.
    [(-m[2][1]).atan2(m[1][1]), y, 0.0]
  };
  let alternate = [
    primary[0] + std::f64::consts::PI,
    std::f64::consts::PI - primary[1],
    primary[2] + std::f64::consts::PI,
  ];
  let distance = |set: [f64; 3]| (0..3).map(|i| wrap(set[i] - anchor[i]).abs()).sum::<f64>();
  let chosen = if distance(alternate) < distance(primary) {
    alternate
  } else {
    primary
  };
  [0, 1, 2].map(|i| anchor[i] + wrap(chosen[i] - anchor[i]))
}

#[cfg(test)]
mod tests;
