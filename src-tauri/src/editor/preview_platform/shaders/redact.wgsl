// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// What the passes that apply one redaction to a copy of the source share,
// before the canvas samples that copy. The twin of
// `gpu_compositor_macos_shader_source_redact.h` and `..._redact_paint.h`.
// The paint pass reads no covered pixel: the fill colour and each pixelated
// zone's inks come from Rust, and which block of a zone takes which shade
// comes from a hash of the seed and the block's place in the box. A classic
// pixelation's blocks are averaged by the cells pass first, and a blur's rows
// by the rows pass, each into a texture the paint pass reads instead of the
// pixels. A blur is a Gaussian over the box's own pixels whose deviation is
// the record's size, which widens from nothing as the blur arrives. The only
// pixels the paint pass reads are those wholly outside a rounded corner,
// which it blends into the corner's soft edge.
//
// Assembled ahead of `redact_cells.wgsl`, `redact_rows.wgsl` and
// `redact_paint.wgsl`, each its own module with its own `fs_main`.

// The twin of `RedactRecord` in `annotations/redact/records.rs`.
struct Redaction {
  bounds: vec4<u32>, // [x0, y0, x1, y1) in whole source pixels
  source_width: u32,
  mode: u32,
  seed: u32,
  size: f32,
  color: vec4<f32>,
  grid: vec2<u32>,
  entry_count: u32,
  radius: f32,
  zone_first: u32,
  padding_0: u32,
  padding_1: u32,
  padding_2: u32,
}

@group(0) @binding(0) var<uniform> redact: Redaction;
@group(0) @binding(1) var redact_source: texture_2d<f32>;
@group(0) @binding(2) var<storage, read> redact_zones: array<vec2<f32>>;
@group(0) @binding(3) var redact_cells: texture_2d<f32>;
@group(0) @binding(4) var redact_rows: texture_2d<f32>;

// The share of blocks left the plain surface colour, as the gaps between
// words and lines leave pixelated text.
const redact_blank_share: f32 = 0.3;

// How many pixels either side of its centre a blur of deviation `sigma`
// reaches: three deviations, past which a tap weighs about a hundredth of the
// centre's. Held to what one pass can afford.
fn redact_reach(sigma: f32) -> i32 {
  return i32(min(ceil(max(sigma, 0.0) * 3.0), 160.0));
}

// The Gaussian's weight `offset` pixels from its centre, for a deviation of
// `sigma`.
fn redact_weight(offset: i32, sigma: f32) -> f32 {
  let along = f32(offset);
  return exp(-0.5 * along * along / max(sigma * sigma, 1e-6));
}

// One triangle over the viewport, which each pass sets to what it writes.
@vertex
fn vs_main(@builtin(vertex_index) id: u32) -> @builtin(position) vec4<f32> {
  let position = vec2<f32>(f32((id << 1u) & 2u), f32(id & 2u));
  return vec4<f32>(position * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 0.0, 1.0);
}

// A 32-bit PCG hash: a well-mixed integer for each block from its place.
fn redact_hash(value: u32) -> u32 {
  let state = value * 747796405u + 2891336453u;
  let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
  return (word >> 22u) ^ word;
}

// One colour from the float Rust packs it in, 24 bits of RGB, in `rgb`, and
// in `a` whether there is one; the twin of `redact::palette::packed`, whose
// `-1` is no colour.
fn redact_ink(value: f32) -> vec4<f32> {
  if (value < 0.0 || value > 16777215.0) {
    return vec4<f32>(0.0);
  }
  let rgb = u32(value);
  return vec4<f32>(vec3<f32>(f32((rgb >> 16u) & 0xffu), f32((rgb >> 8u) & 0xffu),
                             f32(rgb & 0xffu)) / 255.0, 1.0);
}

// One block's shade: the surface, or the surface blended towards one of its
// zone's inks by an amount the hash chooses, most of them lightly. A zone
// with no ink of its own is plain surface. A box packed without a picture
// has no zones, and the surface pulled towards the contrasting end of the
// scale stands in.
fn redact_block(block: vec2<u32>) -> vec3<f32> {
  let hash = redact_hash(redact.seed ^ redact_hash(block.x ^ redact_hash(block.y)));
  let amount = f32(hash & 0xffffu) / 65535.0;
  let fill = redact.color.rgb;
  if (redact.entry_count == 0u || redact.grid.x == 0u) {
    let luminance = dot(fill, vec3<f32>(0.2126, 0.7152, 0.0722));
    let towards = select(vec3<f32>(1.0), vec3<f32>(0.0), luminance > 0.5);
    return mix(fill, towards, 0.06 + 0.5 * amount * amount);
  }
  let zone = block / max(redact.grid.y, 1u);
  let index = min(zone.y * redact.grid.x + min(zone.x, redact.grid.x - 1u),
                  redact.entry_count - 1u);
  let entry = redact_zones[redact.zone_first + index];
  let first = redact_ink(entry.x);
  let second = redact_ink(entry.y);
  let count = u32(first.a) + u32(second.a);
  if (count == 0u || f32((hash >> 24u) & 0xffu) / 255.0 < redact_blank_share) {
    return fill;
  }
  let pick = ((hash >> 16u) & 0xffu) % count;
  let ink = select(second.rgb, first.rgb, first.a > 0.0 && pick == 0u);
  return mix(fill, ink, pow(amount, 1.5));
}

// Where the cell grid starts, in the box's own pixels: centred on the box, so
// the part cells its edges leave are shared out equally, and a grid whose
// cells grow grows from the middle.
fn redact_grid_origin() -> vec2<f32> {
  let box = vec2<f32>(redact.bounds.zw - redact.bounds.xy);
  return (box - vec2<f32>(redact.grid) * max(redact.size, 1.0)) * 0.5;
}

// The cell a pixel `local` pixels into the box falls in: the one division
// both passes make, so a pixel is averaged into the cell it is painted from.
fn redact_cell_of(local: vec2<u32>) -> vec2<u32> {
  let cell = floor((vec2<f32>(local) - redact_grid_origin()) / max(redact.size, 1.0));
  let last = max(redact.grid, vec2<u32>(1u, 1u)) - 1u;
  return min(vec2<u32>(max(cell, vec2<f32>(0.0))), last);
}
