// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// A reference (`references/drawing.rs` in the app): its picture with its
// corners rounded, floating as the islands do, on the same shadow they draw
// for themselves (`--island-shadow`): a close contact shadow and a soft wide
// one. The shadow lies outside the picture only, as the picture can be shown
// see-through and nothing should darken what shows through it; it stays at
// full strength however faint the picture is, so the reference's edge always
// reads. Both quads' uv runs from 0 to 1 across the picture, the shadow's on
// past it into the room round it; the snapshot is the picture.

const reference_radius_points: f32 = 6.0;
// The island's two shadows, in points: how far each falls, how soft it is
// (a CSS blur radius, twice the deviation) and how dark.
const reference_contact_shadow: vec3<f32> = vec3<f32>(1.0, 3.0, 0.25);
const reference_wide_shadow: vec3<f32> = vec3<f32>(10.0, 28.0, 0.35);

/// Where a probe stands in the picture's drawn pixels: the picture's half
/// size, the probe from its middle, and pixels to the point.
struct ReferenceProbe {
  half: vec2<f32>,
  offset: vec2<f32>,
  scale: f32,
}

fn reference_probe(uv: vec2<f32>) -> ReferenceProbe {
  // The picture's size in drawn pixels, read off how fast uv runs.
  let size = 1.0 / max(fwidth(uv), vec2<f32>(1e-6));
  return ReferenceProbe(size * 0.5, (uv - 0.5) * size, max(osc.reference.x, 1.0));
}

/// How far `offset` lies outside a box `half` each side of its middle with
/// corners rounded by `radius`; negative inside.
fn reference_box(offset: vec2<f32>, half: vec2<f32>, radius: f32) -> f32 {
  let q = abs(offset) - half + radius;
  return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - radius;
}

fn reference_radius(probe: ReferenceProbe) -> f32 {
  return min(reference_radius_points * probe.scale, min(probe.half.x, probe.half.y));
}

/// The picture, its corners rounded.
fn reference_picture(uv: vec2<f32>) -> vec4<f32> {
  let probe = reference_probe(uv);
  let cover = saturate(0.5 - reference_box(probe.offset, probe.half, reference_radius(probe)));
  if (cover <= 0.0) {
    discard;
  }
  return vec4<f32>(frozen_picture(uv), cover);
}

/// The error function, to within a few thousandths: how much of a gaussian
/// lies short of `x` deviations, mapped onto -1 to 1.
fn reference_erf(x: f32) -> f32 {
  let a = 0.147;
  let squared = x * x;
  let inner = squared * (1.2732395 + a * squared) / (1.0 + a * squared);
  return sign(x) * sqrt(1.0 - exp(-inner));
}

/// How dark one of the shadows is at the probe: the picture's shape, fallen
/// `shadow.x` and softened by `shadow.y`, at `shadow.z`.
fn reference_shadow_at(probe: ReferenceProbe, shadow: vec3<f32>) -> f32 {
  let fallen = probe.offset - vec2<f32>(0.0, shadow.x * probe.scale);
  let deviation = max(shadow.y * 0.5 * probe.scale, 1.0);
  let outside = reference_box(fallen, probe.half, reference_radius(probe));
  return shadow.z * 0.5 * (1.0 - reference_erf(outside / (deviation * 1.4142135)));
}

/// The shadow round the picture, outside it only.
fn reference_shadow(uv: vec2<f32>) -> vec4<f32> {
  let probe = reference_probe(uv);
  let outside = saturate(0.5 + reference_box(probe.offset, probe.half, reference_radius(probe)));
  let contact = reference_shadow_at(probe, reference_contact_shadow);
  let wide = reference_shadow_at(probe, reference_wide_shadow);
  let dark = (1.0 - (1.0 - contact) * (1.0 - wide)) * outside;
  if (dark <= 0.0) {
    discard;
  }
  return vec4<f32>(0.0, 0.0, 0.0, dark);
}
