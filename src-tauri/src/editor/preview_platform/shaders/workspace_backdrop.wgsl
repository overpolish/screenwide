// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The macOS editor workspace's backdrop: a checkerboard under every layer, so
// what a canvas shows past its edge, which is never exported, reads as lying
// off it. The squares are laid on the drawable's own pixels, so they keep
// their size on screen however far the workspace is zoomed.

struct Backdrop {
  // The two squares' colours, premultiplied.
  even: vec4<f32>,
  odd: vec4<f32>,
  // A square's side in drawable pixels; the rest are spare.
  square: vec4<f32>,
}

@group(0) @binding(0) var<uniform> backdrop: Backdrop;

@vertex
fn vs_main(@builtin(vertex_index) id: u32) -> @builtin(position) vec4<f32> {
  let corner = vec2<f32>(f32((id << 1u) & 2u), f32(id & 2u));
  return vec4<f32>(corner * 2.0 - 1.0, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
  let square = vec2<i32>(floor(position.xy / max(backdrop.square.x, 1.0)));
  return select(backdrop.even, backdrop.odd, ((square.x + square.y) & 1) != 0);
}
