// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The crop tool's loupe over the macOS editor workspace: the lens drawn into
// the workspace drawable over every layer, from the picture the selected
// layer was drawn from. Assembled with `lens.wgsl`.

@group(0) @binding(0) var<uniform> loupe: Lens;
@group(0) @binding(1) var picture: texture_2d<f32>;
@group(0) @binding(2) var nearest: sampler;

@vertex
fn vs_main(@builtin(vertex_index) id: u32) -> @builtin(position) vec4<f32> {
  let corner = vec2<f32>(f32((id << 1u) & 2u), f32(id & 2u));
  return vec4<f32>(corner * 2.0 - 1.0, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
  let color = lens_color(loupe, position.xy, picture, nearest);
  if (color.a <= 0.0) {
    discard;
  }
  // The drawable holds premultiplied colour, and the lens replaces what the
  // layers drew under it; the OSC drawn next keeps drawing wherever the lens
  // is not fully opaque.
  return vec4<f32>(color.rgb * color.a, color.a);
}
