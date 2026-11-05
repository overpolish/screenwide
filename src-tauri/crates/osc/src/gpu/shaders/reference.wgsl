// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// A reference (`references/drawing.rs` in the app): its picture with its
// corners rounded, floating as the islands do, on the same shadow they draw
// for themselves (`--island-shadow`): a close contact shadow and a soft wide
// one. The shadow lies outside the picture only, as the picture can be shown
// see-through and nothing should darken what shows through it; it stays at
// full strength however faint the picture is, so the reference's edge always
// reads. A picture with see-through pixels of its own casts its shadow from
// what it shows, through the clear parts too. Both quads' uv runs from 0 to
// 1 across the picture, the shadow's on past it into the room round it; the
// snapshot is the picture, with mip levels where it casts its shadow.

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

/// The picture, its corners rounded, as see-through as its own pixels are.
fn reference_picture(uv: vec2<f32>) -> vec4<f32> {
  let probe = reference_probe(uv);
  let cover = saturate(0.5 - reference_box(probe.offset, probe.half, reference_radius(probe)));
  let picture = frozen_picture(uv);
  if (cover * picture.a <= 0.0) {
    discard;
  }
  return vec4<f32>(picture.rgb, picture.a * cover);
}

/// The error function, to within a few thousandths: how much of a gaussian
/// lies short of `x` deviations, mapped onto -1 to 1.
fn reference_erf(x: f32) -> f32 {
  let a = 0.147;
  let squared = x * x;
  let inner = squared * (1.2732395 + a * squared) / (1.0 + a * squared);
  return sign(x) * sqrt(1.0 - exp(-inner));
}

/// How much of the picture's box, its corners rounded, falls on the probe
/// once fallen `shadow.x` and softened by `shadow.y`: from 1 well inside
/// it to 0 well outside.
fn reference_box_cast(probe: ReferenceProbe, shadow: vec3<f32>) -> f32 {
  let fallen = probe.offset - vec2<f32>(0.0, shadow.x * probe.scale);
  let deviation = max(shadow.y * 0.5 * probe.scale, 1.0);
  let outside = reference_box(fallen, probe.half, reference_radius(probe));
  return 0.5 * (1.0 - reference_erf(outside / (deviation * 1.4142135)));
}

/// How dark one of the shadows is at the probe: the picture's shape, fallen
/// `shadow.x` and softened by `shadow.y`, at `shadow.z`.
fn reference_shadow_at(probe: ReferenceProbe, shadow: vec3<f32>) -> f32 {
  return shadow.z * reference_box_cast(probe, shadow);
}

/// How much of the picture shows at `offset` from its middle, in drawn
/// pixels, read from mip level `level`: its own alpha within its rounded
/// corners, and nothing past its edge.
fn reference_cover_at(probe: ReferenceProbe, offset: vec2<f32>, level: f32) -> f32 {
  let uv = offset / (probe.half * 2.0) + 0.5;
  if (any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0))) {
    return 0.0;
  }
  let corners = saturate(0.5 - reference_box(offset, probe.half, reference_radius(probe)));
  return textureSampleLevel(snapshot, linear_sampler, uv, level).a * corners;
}

/// The picture's alpha at `offset` from its middle, in drawn pixels, read
/// from mip level `level`, its edge carried on past it: a blur reaching
/// over the edge reads what stands at the edge, and the box's own softened
/// edge (`reference_box_cast`) takes it away from there.
fn reference_alpha_near(probe: ReferenceProbe, offset: vec2<f32>, level: f32) -> f32 {
  let uv = clamp(offset / (probe.half * 2.0) + 0.5, vec2<f32>(0.0), vec2<f32>(1.0));
  return textureSampleLevel(snapshot, linear_sampler, uv, level).a;
}

/// One of the shadows cast by what the picture shows rather than by its
/// box, for a picture with see-through pixels: its alpha, fallen and
/// blurred, within the shadow its box casts. The blur reads the mip level
/// whose texels are about as wide as it, with a few samples a deviation
/// apart, weighted as a gaussian is, so it costs the same at any size the
/// picture is drawn. The box's shadow fades it out past the picture's edge
/// as an opaque picture's fades, where the samples alone would stop short
/// at the edge.
fn reference_cast_at(probe: ReferenceProbe, shadow: vec3<f32>) -> f32 {
  let fallen = probe.offset - vec2<f32>(0.0, shadow.x * probe.scale);
  let deviation = max(shadow.y * 0.5 * probe.scale, 1.0);
  let texels_per_pixel = f32(textureDimensions(snapshot).x) / (probe.half.x * 2.0);
  let coarsest = f32(textureNumLevels(snapshot) - 1u);
  let level = clamp(floor(log2(max(deviation * texels_per_pixel, 1.0))), 0.0, coarsest);
  var sum = 0.0;
  var weights = 0.0;
  for (var y = -1; y <= 1; y++) {
    for (var x = -1; x <= 1; x++) {
      let step = vec2<f32>(f32(x), f32(y));
      let weight = exp(-0.5 * dot(step, step));
      sum += weight * reference_alpha_near(probe, fallen + step * deviation, level);
      weights += weight;
    }
  }
  return shadow.z * sum / weights * reference_box_cast(probe, shadow);
}

/// The shadow round the picture, outside it only: outside its box, or for
/// a picture with see-through pixels (`osc.reference.y`), wherever it is
/// see-through, cast by what it shows.
fn reference_shadow(uv: vec2<f32>) -> vec4<f32> {
  let probe = reference_probe(uv);
  let edge = reference_box(probe.offset, probe.half, reference_radius(probe));
  var outside = saturate(0.5 + edge);
  var contact = 0.0;
  var wide = 0.0;
  if (osc.reference.y > 0.5) {
    // Past where the wide shadow reaches, nothing is cast.
    let reach = (reference_wide_shadow.x + reference_wide_shadow.y * 1.5) * probe.scale;
    if (edge > reach) {
      discard;
    }
    outside = 1.0 - reference_cover_at(probe, probe.offset, 0.0);
    contact = reference_cast_at(probe, reference_contact_shadow);
    wide = reference_cast_at(probe, reference_wide_shadow);
  } else {
    contact = reference_shadow_at(probe, reference_contact_shadow);
    wide = reference_shadow_at(probe, reference_wide_shadow);
  }
  let dark = (1.0 - (1.0 - contact) * (1.0 - wide)) * outside;
  if (dark <= 0.0) {
    discard;
  }
  return vec4<f32>(0.0, 0.0, 0.0, dark);
}
