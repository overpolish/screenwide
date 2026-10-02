// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The recording bar's camera confidence thumbnail: the camera frame sampled
// straight down to a few dozen pixels, as opaque RGBA. A BGRA frame is one
// texture; a 4:2:0 frame is its luma and its interleaved chroma.

// The twin of `Uniforms` in `confidence/scaler.rs`.
struct Thumbnail {
  size: vec2<u32>,
  full_range: u32,
  spare: u32,
}

@group(0) @binding(0) var<uniform> thumbnail: Thumbnail;
// The BGRA frame, or a 4:2:0 frame's luma.
@group(0) @binding(1) var first: texture_2d<f32>;
// A 4:2:0 frame's chroma; the BGRA pass binds `first` here again.
@group(0) @binding(2) var second: texture_2d<f32>;
@group(0) @binding(3) var linear_sampler: sampler;

// One triangle over the target.
@vertex
fn vs_main(@builtin(vertex_index) id: u32) -> @builtin(position) vec4<f32> {
  let corner = vec2<f32>(f32((id << 1u) & 2u), f32(id & 2u));
  return vec4<f32>(corner * 2.0 - 1.0, 0.0, 1.0);
}

fn coordinate(position: vec4<f32>) -> vec2<f32> {
  return position.xy / vec2<f32>(thumbnail.size);
}

@fragment
fn fs_bgra(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
  let color = textureSample(first, linear_sampler, coordinate(position));
  return vec4<f32>(saturate(color.rgb), 1.0);
}

// BT.709, in video or full range.
fn yuv_to_rgb(y: f32, uv: vec2<f32>, full_range: bool) -> vec3<f32> {
  let luma = select((y - 16.0 / 255.0) * (255.0 / 219.0), y, full_range);
  let chroma = select((uv - 128.0 / 255.0) * (255.0 / 224.0), uv - 0.5, full_range);
  return saturate(vec3<f32>(
    luma + 1.5748 * chroma.y,
    luma - 0.1873 * chroma.x - 0.4681 * chroma.y,
    luma + 1.8556 * chroma.x,
  ));
}

@fragment
fn fs_biplanar(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
  let at = coordinate(position);
  let rgb = yuv_to_rgb(
    textureSample(first, linear_sampler, at).r,
    textureSample(second, linear_sampler, at).rg,
    thumbnail.full_range != 0u,
  );
  return vec4<f32>(rgb, 1.0);
}
