// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// A short-video app's interface laid over the canvas in the chrome drawable:
// one quad over the canvas's rect, which may run past the drawable while the
// workspace is zoomed in, sampling the overlay drawn for it (premultiplied).

struct Placement {
  // The canvas in the drawable's pixels: x, y, width, height.
  rect: vec4<f32>,
  // The drawable's width and height, how opaque the overlay is drawn while
  // it fades, then nothing.
  target_size: vec4<f32>,
}

@group(0) @binding(0) var<uniform> placement: Placement;
@group(0) @binding(1) var picture: texture_2d<f32>;
@group(0) @binding(2) var smooth_sampler: sampler;

struct Corner {
  @builtin(position) position: vec4<f32>,
  @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) id: u32) -> Corner {
  var corners = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 0.0),
    vec2<f32>(1.0, 0.0),
    vec2<f32>(0.0, 1.0),
    vec2<f32>(0.0, 1.0),
    vec2<f32>(1.0, 0.0),
    vec2<f32>(1.0, 1.0),
  );
  let uv = corners[id];
  let pixel = placement.rect.xy + uv * placement.rect.zw;
  let clip = vec2<f32>(
    pixel.x / placement.target_size.x * 2.0 - 1.0,
    1.0 - pixel.y / placement.target_size.y * 2.0,
  );
  var corner: Corner;
  corner.position = vec4<f32>(clip, 0.0, 1.0);
  corner.uv = uv;
  return corner;
}

@fragment
fn fs_main(corner: Corner) -> @location(0) vec4<f32> {
  // Premultiplied, so the whole colour fades with its alpha.
  return textureSampleLevel(picture, smooth_sampler, corner.uv, 0.0) * placement.target_size.z;
}
