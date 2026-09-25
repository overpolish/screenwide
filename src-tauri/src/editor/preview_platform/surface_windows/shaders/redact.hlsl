// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// What the passes that apply one redaction to a copy of the source share,
// before the canvas samples that copy. The twin of
// `gpu_compositor_macos_shader_source_redact.h`. The paint pass reads no
// covered pixel: the fill colour and each pixelated zone's inks come from
// Rust, and which block of a zone takes which shade comes from a hash of the
// seed and the block's place in the box. A classic pixelation's blocks and a
// blur's cells are averaged by the cells pass first, into a texture the paint
// pass reads instead of the pixels. The only pixels the paint pass reads are
// those wholly outside a rounded corner, which it blends into the corner's
// soft edge.
//
// Included textually by `redact_cells.hlsl` and `redact_paint.hlsl`, each of
// which `compile_shader` in build.rs builds with its own `ps_main`.

// The twin of `RedactRecord` in `annotations/redact/records.rs`.
cbuffer Redaction : register(b0) {
  uint4 redact_bounds; // [x0, y0, x1, y1) in whole source pixels
  uint redact_source_width;
  uint redact_mode;
  uint redact_seed;
  float redact_size;
  float4 redact_color;
  uint2 redact_grid;
  uint redact_entry_count;
  float redact_radius;
  uint redact_zone_first;
  uint3 redact_padding;
};
Texture2D<float4> redact_source : register(t0);
StructuredBuffer<float2> redact_zones : register(t1);
Texture2D<float4> redact_cells : register(t2);

// The share of blocks left the plain surface colour, as the gaps between
// words and lines leave pixelated text.
static const float redact_blank_share = 0.3;
// How far the seed moves each channel of a blurred cell's average, either
// way: enough that the output never equals the averages of a guessed
// original, too little to see.
static const uint redact_jitter = 6u;

// One triangle over the viewport, which each pass sets to what it writes.
float4 vs_main(uint id : SV_VertexID) : SV_Position {
  float2 position = float2((id << 1) & 2, id & 2);
  return float4(position * float2(2, -2) + float2(-1, 1), 0, 1);
}

// A 32-bit PCG hash: a well-mixed integer for each block from its place.
uint redact_hash(uint value) {
  uint state = value * 747796405u + 2891336453u;
  uint word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
  return (word >> 22u) ^ word;
}

// One colour from the float Rust packs it in, 24 bits of RGB; the twin of
// `redact::palette::packed`, whose `-1` is no colour.
bool redact_ink(float value, out float3 ink) {
  ink = float3(0.0, 0.0, 0.0);
  if (value < 0.0 || value > 16777215.0) return false;
  uint rgb = (uint)value;
  ink = float3((rgb >> 16u) & 0xffu, (rgb >> 8u) & 0xffu, rgb & 0xffu) / 255.0;
  return true;
}

// One block's shade: the surface, or the surface blended towards one of its
// zone's inks by an amount the hash chooses, most of them lightly. A zone
// with no ink of its own is plain surface. A box packed without a picture
// has no zones, and the surface pulled towards the contrasting end of the
// scale stands in.
float3 redact_block(uint2 block) {
  uint hash = redact_hash(redact_seed ^ redact_hash(block.x ^ redact_hash(block.y)));
  float amount = float(hash & 0xffffu) / 65535.0;
  float3 fill = redact_color.rgb;
  if (redact_entry_count == 0u || redact_grid.x == 0u) {
    float luminance = dot(fill, float3(0.2126, 0.7152, 0.0722));
    float3 towards = luminance > 0.5 ? float3(0.0, 0.0, 0.0) : float3(1.0, 1.0, 1.0);
    return lerp(fill, towards, 0.06 + 0.5 * amount * amount);
  }
  uint2 zone = block / max(redact_grid.y, 1u);
  uint index = min(zone.y * redact_grid.x + min(zone.x, redact_grid.x - 1u),
                   redact_entry_count - 1u);
  float2 entry = redact_zones[redact_zone_first + index];
  float3 first, second;
  bool has_first = redact_ink(entry.x, first);
  bool has_second = redact_ink(entry.y, second);
  uint count = (has_first ? 1u : 0u) + (has_second ? 1u : 0u);
  if (count == 0u || float((hash >> 24u) & 0xffu) / 255.0 < redact_blank_share) return fill;
  uint pick = ((hash >> 16u) & 0xffu) % count;
  float3 ink = has_first && pick == 0u ? first : second;
  return lerp(fill, ink, pow(amount, 1.5));
}

// Where the cell grid starts, in the box's own pixels: centred on the box, so
// the part cells its edges leave are shared out equally, and a grid whose
// cells grow grows from the middle.
float2 redact_grid_origin() {
  float2 box = float2(redact_bounds.zw - redact_bounds.xy);
  return (box - float2(redact_grid) * max(redact_size, 1.0)) * 0.5;
}

// The cell a pixel `local` pixels into the box falls in: the one division
// both passes make, so a pixel is averaged into the cell it is painted from.
uint2 redact_cell_of(uint2 local) {
  float2 cell = floor((float2(local) - redact_grid_origin()) / max(redact_size, 1.0));
  uint2 last = max(redact_grid, uint2(1u, 1u)) - 1u;
  return min(uint2(max(cell, 0.0)), last);
}
