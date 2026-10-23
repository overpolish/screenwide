// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// How wide a focus's blur is at a point, for any module that joins this
// file: the editor's focus blur (`focus_blur.wgsl`) and the live overlay
// (`annotate_overlay.wgsl`), so a focus drawn live softens what is around it
// exactly as it does in the editor. The blur is none inside the box and grows
// with the distance past its rounded edge, smoothly, to the whole blur its
// strength asks for.

// The twin of `GpuFocusRegion` in `annotation_gpu/focus_region.rs`.
struct FocusRegion {
  // The box's top-left and bottom-right corners.
  box: vec4<f32>,
  // The corner radius, the strength from one to five, how present the box is
  // this frame, and the inverse turn's weight at the box's middle, which the
  // editor's depth term measures from; the overlay's screens are never turned.
  radius: f32,
  strength: f32,
  presence: f32,
  reference: f32,
}

// How far past a rounded box a point lies: negative inside it.
fn focus_box_distance(point: vec2<f32>, region: FocusRegion) -> f32 {
  let half_size = (region.box.zw - region.box.xy) * 0.5;
  let q = abs(point - (region.box.xy + region.box.zw) * 0.5) - half_size + region.radius;
  return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - region.radius;
}

// How far into its whole blur a point `distance` past a box's edge is: none
// at the edge, all of it `reach` past it.
fn focus_falloff(distance: f32, reach: f32) -> f32 {
  let fall = clamp(distance / max(reach, 1.0), 0.0, 1.0);
  return fall * fall * (3.0 - 2.0 * fall);
}

// The blur `region` asks for `amount` of the way into it, when the middle
// strength's whole blur is `widest`.
fn focus_width(region: FocusRegion, widest: f32, amount: f32) -> f32 {
  return widest * (region.strength / 3.0) * region.presence * amount;
}
