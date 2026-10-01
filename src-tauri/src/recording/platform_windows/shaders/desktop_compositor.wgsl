// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// One display's piece of a recording that crosses display boundaries: the
// piece's monitor pixels, scaled into its place on the shared canvas.

// The twin of `PieceConstants` in `desktop_compositor.rs`.
struct Piece {
  output_size: vec2<u32>,
  source_size: vec2<u32>,
  source_origin: vec2<u32>,
  source_extent: vec2<u32>,
  destination_origin: vec2<u32>,
  destination_extent: vec2<u32>,
}

@group(0) @binding(0) var<uniform> piece: Piece;
@group(0) @binding(1) var source_texture: texture_2d<f32>;
@group(0) @binding(2) var linear_sampler: sampler;

struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) id: u32) -> VertexOutput {
  var corners = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0),
    vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0),
  );
  let corner = corners[id];
  let pixel = vec2<f32>(piece.destination_origin) + corner * vec2<f32>(piece.destination_extent);
  let output = vec2<f32>(piece.output_size);
  var out: VertexOutput;
  out.position = vec4<f32>(pixel.x / output.x * 2.0 - 1.0, 1.0 - pixel.y / output.y * 2.0, 0.0, 1.0);
  out.uv = (vec2<f32>(piece.source_origin) + corner * vec2<f32>(piece.source_extent)) /
      vec2<f32>(piece.source_size);
  return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
  return textureSample(source_texture, linear_sampler, input.uv);
}
