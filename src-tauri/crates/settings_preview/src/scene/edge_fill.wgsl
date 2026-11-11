// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// A frame drawn across a whole surface bigger than it, its last column and
// row repeated over the rest; see `edge_fill.rs`.

@group(0) @binding(0) var frame: texture_2d<f32>;

@vertex
fn vertex_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
  let corner = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
  return vec4<f32>(corner * 2.0 - 1.0, 0.0, 1.0);
}

@fragment
fn fragment_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
  let last = vec2<i32>(textureDimensions(frame)) - vec2<i32>(1);
  return textureLoad(frame, clamp(vec2<i32>(position.xy), vec2<i32>(0), last), 0);
}
