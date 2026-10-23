// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// One mip level made from the level above it: each texel the average of the
// two by two block above, read with one linear sample at the block's middle.

@group(0) @binding(0) var above: texture_2d<f32>;
@group(0) @binding(1) var between: sampler;

@vertex
fn vs_main(@builtin(vertex_index) id: u32) -> @builtin(position) vec4<f32> {
  let corner = vec2<f32>(f32((id << 1u) & 2u), f32(id & 2u));
  return vec4<f32>(corner * 2.0 - 1.0, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
  let size = vec2<f32>(textureDimensions(above));
  return textureSampleLevel(above, between, position.xy * 2.0 / size, 0.0);
}
