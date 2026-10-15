// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! C ABI the macOS editor overlay turns its chrome and its presses with:
//! the rotation gizmo's rings and the turn that carries a layer's chrome
//! where the compositor draws it. Angles cross in degrees.

use super::{composed, ring_at, ring_basis, ring_radius, within, RingDrag};
use screenwide_scenes::rotation::{inverted, turned_homography};
use screenwide_scenes::SceneRotation;

// The macOS overlay holds a drag as `ScreenwideTurnRingDrag`.
const _: () = assert!(std::mem::size_of::<RingDrag>() == 80);

/// The ring a press takes hold of: its axis, or -1 for none, the resize
/// edges whose cursor runs the way it turns there, and the angle along it.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct NativeRingHit {
  pub axis: i32,
  pub edges: u32,
  pub angle: f64,
}

fn rotation(x: f64, y: f64, z: f64) -> SceneRotation {
  SceneRotation { x, y, z }
}

/// The rings' radius, in display points, over a layer whose shorter side is
/// `short` display points on screen.
#[no_mangle]
pub extern "C" fn screenwide_turn_ring_radius(short: f64) -> f64 {
  ring_radius(short)
}

#[no_mangle]
pub extern "C" fn screenwide_turn_ring_at(
  x: f64,
  y: f64,
  z: f64,
  radius: f64,
  px: f64,
  py: f64,
) -> NativeRingHit {
  ring_at(rotation(x, y, z), radius, (px, py)).map_or(
    NativeRingHit {
      axis: -1,
      edges: 0,
      angle: 0.0,
    },
    |hit| NativeRingHit {
      axis: hit.axis as i32,
      edges: hit.edges,
      angle: hit.angle,
    },
  )
}

/// Writes the directions spanning the X, Y and Z rings into `out`'s eighteen
/// values: each ring's `u`, then each ring's `v`, as `x, y, depth` with Y
/// down, so ring `axis`'s point at angle `t` is `cos t * u + sin t * v` times
/// the radius.
///
/// # Safety
/// `out` must be valid for eighteen writes.
#[no_mangle]
pub unsafe extern "C" fn screenwide_turn_ring_basis(x: f64, y: f64, z: f64, out: *mut f64) {
  if out.is_null() {
    return;
  }
  let out = unsafe { &mut *out.cast::<[[f64; 3]; 6]>() };
  *out = ring_basis(rotation(x, y, z));
}

#[no_mangle]
pub extern "C" fn screenwide_turn_ring_begin(
  x: f64,
  y: f64,
  z: f64,
  radius: f64,
  axis: u32,
  angle: f64,
) -> RingDrag {
  RingDrag::begin(rotation(x, y, z), radius, axis as usize, angle)
}

/// Moves `drag` to a pointer `dx, dy` display points from its press, writing
/// the turn it reaches, in degrees, into `out`'s three values.
///
/// # Safety
/// `drag` must point at a live drag and `out` be valid for three writes.
#[no_mangle]
pub unsafe extern "C" fn screenwide_turn_ring_drag(
  drag: *mut RingDrag,
  dx: f64,
  dy: f64,
  snap: u8,
  out: *mut f64,
) {
  let (Some(drag), false) = (unsafe { drag.as_mut() }, out.is_null()) else {
    return;
  };
  let turned = drag.turned((dx, dy), snap != 0);
  let out = unsafe { std::slice::from_raw_parts_mut(out, 3) };
  out.copy_from_slice(&[turned.x, turned.y, turned.z]);
}

/// The three angles at `values`, in degrees.
///
/// # Safety
/// `values` must be valid for three reads.
unsafe fn read_rotation(values: *const f64) -> SceneRotation {
  let [x, y, z] = unsafe { *values.cast::<[f64; 3]>() };
  rotation(x, y, z)
}

/// Writes the turn seen of something turned `inner` within a frame turned
/// `outer` into `out`. Every turn is three angles in degrees.
///
/// # Safety
/// `outer` and `inner` must each be valid for three reads, `out` for three
/// writes.
#[no_mangle]
pub unsafe extern "C" fn screenwide_turn_compose(
  outer: *const f64,
  inner: *const f64,
  out: *mut f64,
) {
  if outer.is_null() || inner.is_null() || out.is_null() {
    return;
  }
  let seen = composed(unsafe { read_rotation(outer) }, unsafe {
    read_rotation(inner)
  });
  unsafe { std::slice::from_raw_parts_mut(out, 3) }.copy_from_slice(&[seen.x, seen.y, seen.z]);
}

/// Writes the turn within a frame turned `outer` that is seen as `seen`,
/// nearest `near`, into `out`. Every turn is three angles in degrees.
///
/// # Safety
/// `outer`, `seen` and `near` must each be valid for three reads, `out` for
/// three writes.
#[no_mangle]
pub unsafe extern "C" fn screenwide_turn_within(
  outer: *const f64,
  seen: *const f64,
  near: *const f64,
  out: *mut f64,
) {
  if outer.is_null() || seen.is_null() || near.is_null() || out.is_null() {
    return;
  }
  let own = within(
    unsafe { read_rotation(outer) },
    unsafe { read_rotation(seen) },
    unsafe { read_rotation(near) },
  );
  unsafe { std::slice::from_raw_parts_mut(out, 3) }.copy_from_slice(&[own.x, own.y, own.z]);
}

/// Writes the turn of a layer turned `x, y, z` about `cx, cy`, on a canvas
/// whose longer side is `extent`, into `forward`, and its inverse into
/// `inverse`, each nine values row by row. Answers zero, writing nothing,
/// where the layer is seen edge on and nothing of it can be pressed.
///
/// # Safety
/// `forward` and `inverse` must each be valid for nine writes.
#[no_mangle]
#[allow(clippy::too_many_arguments)] // C callers pass the turn and its centre apart.
pub unsafe extern "C" fn screenwide_turn_homographies(
  x: f64,
  y: f64,
  z: f64,
  cx: f64,
  cy: f64,
  extent: f64,
  forward: *mut f64,
  inverse: *mut f64,
) -> u8 {
  if forward.is_null() || inverse.is_null() {
    return 0;
  }
  let turn = turned_homography(rotation(x, y, z), (cx, cy), extent);
  let Some(back) = inverted(turn) else {
    return 0;
  };
  let forward = unsafe { std::slice::from_raw_parts_mut(forward, 9) };
  let inverse = unsafe { std::slice::from_raw_parts_mut(inverse, 9) };
  forward.copy_from_slice(turn.as_flattened());
  inverse.copy_from_slice(back.as_flattened());
  1
}
