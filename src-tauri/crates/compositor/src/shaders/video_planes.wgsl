// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// A video's 4:2:0 planes in and out of the canvas. Decoders hand macOS
// frames as BT.709 video-range luma and interleaved chroma, and its encoder
// takes the same, so a frame is made RGB before the canvas samples it and
// split back into planes once it is drawn. Coefficients are the ones every
// macOS export has written with, so exports keep their colour.

@group(0) @binding(0) var first: texture_2d<f32>;
@group(0) @binding(1) var second: texture_2d<f32>;
@group(0) @binding(2) var linear_sampler: sampler;

@vertex
fn vs_main(@builtin(vertex_index) id: u32) -> @builtin(position) vec4<f32> {
  let corner = vec2<f32>(f32((id << 1u) & 2u), f32(id & 2u));
  return vec4<f32>(corner * 2.0 - 1.0, 0.0, 1.0);
}

// Chroma is taken as centred on 0.5 without video range's 224-step stretch:
// macOS exports have always decoded it so.
fn yuv_to_rgb(y: f32, uv: vec2<f32>) -> vec3<f32> {
  let luma = max(0.0, (y - 16.0 / 255.0) * (255.0 / 219.0));
  let chroma = uv - 0.5;
  return saturate(vec3<f32>(
    luma + 1.5748 * chroma.y,
    luma - 0.1873 * chroma.x - 0.4681 * chroma.y,
    luma + 1.8556 * chroma.x,
  ));
}

// `first` is the luma plane and `second` the chroma plane at half size. Luma
// is read where the pixel is; chroma is filtered to it, as sampling both
// planes at the canvas's own point did.
@fragment
fn fs_from_planes(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
  let luma = textureLoad(first, vec2<i32>(position.xy), 0).r;
  let at = position.xy / vec2<f32>(textureDimensions(first));
  let chroma = textureSampleLevel(second, linear_sampler, at, 0.0).rg;
  return vec4<f32>(yuv_to_rgb(luma, chroma), 1.0);
}

// `first` is the drawn canvas, its colour already faded past the canvas's
// rounded corners.
@fragment
fn fs_luma(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
  let rgb = textureLoad(first, vec2<i32>(position.xy), 0).rgb;
  return vec4<f32>(16.0 / 255.0 + dot(rgb, vec3<f32>(0.182586, 0.614231, 0.062007)), 0.0, 0.0, 1.0);
}

// One chroma sample is the mean of the 2x2 canvas pixels it covers; an odd
// last row or column repeats its edge.
@fragment
fn fs_chroma(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
  let last = vec2<i32>(textureDimensions(first)) - vec2<i32>(1);
  let base = vec2<i32>(position.xy) * 2;
  var rgb = vec3<f32>(0.0);
  for (var y = 0; y < 2; y++) {
    for (var x = 0; x < 2; x++) {
      rgb += textureLoad(first, min(base + vec2<i32>(x, y), last), 0).rgb;
    }
  }
  rgb *= 0.25;
  return vec4<f32>(
    0.5 + dot(rgb, vec3<f32>(-0.100644, -0.338572, 0.439216)),
    0.5 + dot(rgb, vec3<f32>(0.439216, -0.398942, -0.040274)),
    0.0,
    1.0,
  );
}
