// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// One axis of the Gaussian the spotlights' blur lays over the annotations
// under their shade and over the cursor. The first layer holds what those
// annotations add to the canvas, as a difference from the canvas without
// them; the second the cursor alone. Past their edges both are nothing. Run
// along the rows, then along the columns.

struct MarkBlur {
  // One texel along the axis this pass blurs.
  direction: vec2<i32>,
  // The deviation in texels, and how many texels either side are taken.
  sigma: f32,
  reach: i32,
}

struct Blurred {
  @location(0) marks: vec4<f32>,
  @location(1) cursor: vec4<f32>,
}

@group(0) @binding(0) var<uniform> mark_blur: MarkBlur;
@group(0) @binding(1) var mark_blur_marks: texture_2d<f32>;
@group(0) @binding(2) var mark_blur_cursor: texture_2d<f32>;

@vertex
fn vs_main(@builtin(vertex_index) id: u32) -> @builtin(position) vec4<f32> {
  let position = vec2<f32>(f32((id << 1u) & 2u), f32(id & 2u));
  return vec4<f32>(position * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> Blurred {
  let size = vec2<i32>(textureDimensions(mark_blur_marks));
  let texel = vec2<i32>(floor(position.xy));
  let spread = 2.0 * mark_blur.sigma * mark_blur.sigma;
  var marks = vec4<f32>(0.0);
  var cursor = vec4<f32>(0.0);
  var weights = 0.0;
  for (var offset = -mark_blur.reach; offset <= mark_blur.reach; offset++) {
    let weight = exp(-f32(offset * offset) / spread);
    weights += weight;
    let at = texel + mark_blur.direction * offset;
    if (all(at >= vec2<i32>(0)) && all(at < size)) {
      marks += weight * textureLoad(mark_blur_marks, at, 0);
      cursor += weight * textureLoad(mark_blur_cursor, at, 0);
    }
  }
  return Blurred(marks / weights, cursor / weights);
}
