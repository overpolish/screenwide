// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The canvas's generator drawn once a frame into a picture at half the
// resolution the canvas is shown at, which the canvas pass reads in place of
// evaluating it at every pixel (`canvas_background.wgsl`,
// `compositor/background_field.rs`). Assembled with the generators by
// `build.rs`.

// The twin of `FieldConstants` in `compositor/background_field.rs`.
struct Field {
  // The canvas's size in pixels, then this picture's.
  size: vec4<f32>,
  colors: array<vec4<f32>, 4>,
  // The seed's domain shift as the canvas carries it in its first mesh
  // point: `w` set asks for the shader's own hash of the seed.
  shift: vec4<f32>,
  // The canvas seconds and the generator's speed.
  timing: vec4<f32>,
  // The generator, its palette's size and the seed.
  style: vec4<u32>,
}

@group(0) @binding(0) var<uniform> field: Field;

@vertex
fn vs_main(@builtin(vertex_index) id: u32) -> @builtin(position) vec4<f32> {
  let position = vec2<f32>(f32((id << 1u) & 2u), f32(id & 2u));
  return vec4<f32>(position * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
  // A texel's centre is the canvas point the canvas pass reads it at.
  let pixel = position.xy * field.size.xy / field.size.zw;
  let palette = GenPalette(field.colors[0].rgb, field.colors[1].rgb, field.colors[2].rgb,
                           field.colors[3].rgb, field.style.y);
  var shift = field.shift.xyz;
  if (field.shift.w != 0.0) {
    shift = gen_seed_shift(field.style.z);
  }
  return vec4<f32>(gen_pixel_shifted(field.style.x, pixel, field.size.xy, palette, shift,
                                     field.timing.x, field.timing.y), 1.0);
}
