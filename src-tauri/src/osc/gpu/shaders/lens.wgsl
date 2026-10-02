// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The magnifier lens the region OSC and the editor's crop tool show: a
// rounded box showing the picture round its anchor pixel for pixel, a shade
// over the half the dragged edges face, and the bounding box's two-tone rim.
// Assembled into the modules that draw it, after the part carrying their
// directives.

struct Lens {
  box_rect: vec4<f32>, // x/y/width/height in drawn pixels
  source: vec4<f32>, // source width/height in pixels
  sample: vec4<f32>, // anchor u/v inside the source
  source_range: vec4<f32>, // min u/v, max u/v
  flags: vec4<u32>, // edges bitmask, active, light mode
}

fn lens_distance(offset: vec2<f32>, half_size: vec2<f32>, radius: f32) -> f32 {
  let local = abs(offset) - (half_size - radius);
  return length(max(local, vec2<f32>(0.0))) + min(max(local.x, local.y), 0.0) - radius;
}

// The lens box's signed distance at `position`. The box is 96
// device-independent points wide and its corners are the control radius, 8,
// so the backing scale falls out of the box size.
fn lens_box_distance(lens: Lens, position: vec2<f32>) -> f32 {
  let box_size = max(lens.box_rect.zw, vec2<f32>(1.0));
  let half_size = box_size * 0.5;
  let radius = max(box_size.x / 12.0, 1.0);
  return lens_distance(position - lens.box_rect.xy - half_size, half_size, radius);
}

// How much of the drawn pixel at `position` the lens covers: one drawn pixel
// of feathering on its outer edge. Whatever draws under the lens keeps
// drawing wherever this is below 1, so the two edges blend into each other
// instead of meeting at a hard step.
fn lens_coverage(lens: Lens, position: vec2<f32>) -> f32 {
  if (lens.flags.y == 0u || lens.box_rect.z <= 0.0) {
    return 0.0;
  }
  return 1.0 - smoothstep(-0.5, 0.5, lens_box_distance(lens, position));
}

// The lens at `position`, in straight colour with its coverage in alpha.
// `picture` is read nearest, which keeps its magnified pixels square.
fn lens_color(lens: Lens, position: vec2<f32>, picture: texture_2d<f32>,
              nearest: sampler) -> vec4<f32> {
  let coverage = lens_coverage(lens, position);
  if (coverage <= 0.0) {
    return vec4<f32>(0.0);
  }
  let distance = lens_box_distance(lens, position);
  let box_size = max(lens.box_rect.zw, vec2<f32>(1.0));
  let local = position - lens.box_rect.xy;
  let half_size = box_size * 0.5;
  let source_dimensions = max(lens.source.xy, vec2<f32>(1.0));
  let source_center = lens.sample.xy * source_dimensions;
  let source_point = source_center + (local / box_size - 0.5) * 40.0;
  let sample_point = floor(source_point);
  let sample_uv = source_point / source_dimensions;
  let in_source = all(sample_point >= vec2<f32>(0.0)) && all(sample_point < source_dimensions) &&
      all(sample_uv >= lens.source_range.xy) && all(sample_uv <= lens.source_range.zw);
  var pixel = vec4<f32>(0.15, 0.15, 0.16, 1.0);
  if (in_source) {
    pixel = textureSampleLevel(picture, nearest, (sample_point + 0.5) / source_dimensions, 0.0);
  }
  let edges = lens.flags.x;
  let shade = ((edges & 1u) != 0u && local.x < half_size.x) ||
      ((edges & 2u) != 0u && local.x >= half_size.x) ||
      ((edges & 4u) != 0u && local.y < half_size.y) ||
      ((edges & 8u) != 0u && local.y >= half_size.y);
  if (shade) {
    let shade_color = select(vec3<f32>(1.0), vec3<f32>(0.0), lens.flags.z != 0u);
    pixel = vec4<f32>(mix(pixel.rgb, shade_color, 0.1), pixel.a);
  }
  // The border is the bounding box's palette: a 1 px white core with a 1 px
  // dark hairline outside it, so the loupe reads over any picture. Each
  // boundary is feathered over the same one drawn pixel as the outer edge,
  // which keeps the corners from stepping.
  let core = smoothstep(-2.5, -1.5, distance);
  let hairline = smoothstep(-1.5, -0.5, distance);
  var rgb = mix(pixel.rgb, vec3<f32>(1.0), core);
  rgb = mix(rgb, vec3<f32>(0.15, 0.15, 0.16), hairline);
  return vec4<f32>(rgb, coverage);
}
