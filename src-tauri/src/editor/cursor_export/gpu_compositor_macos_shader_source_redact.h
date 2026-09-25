// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

/// The passes that apply one redaction to a source buffer before anything
/// samples it. The paint pass reads no covered pixel: the fill colour and
/// each pixelated zone's inks come from Rust, and which block of a zone takes
/// which shade comes from a hash of the seed and the block's place in the box.
/// A classic pixelation's blocks and a blur's cells are averaged by the cells
/// pass first, into a buffer the paint pass reads instead of the pixels. The
/// only pixels the paint pass reads are those wholly outside a rounded
/// corner, which it blends into the corner's soft edge. The twin of
/// `ScreenwideRedaction` in `gpu_compositor_macos_redact.h`.
#define GPU_COMPOSITOR_MACOS_SHADER_SOURCE_REDACT @R"METAL(
struct RedactUniforms {
  uint x0, y0, x1, y1;
  uint source_width;
  uint mode;
  uint seed;
  float size;
  packed_float4 color;
  uint grid[2];
  uint entry_count;
  float radius;
};

/// A 32-bit PCG hash: a well-mixed integer for each block from its place.
static uint redact_hash(uint value) {
  uint state = value * 747796405u + 2891336453u;
  uint word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
  return (word >> 22u) ^ word;
}

/// The share of blocks left the plain surface colour, as the gaps between
/// words and lines leave pixelated text.
constant float redact_blank_share = 0.3;

/// One colour from the float Rust packs it in, 24 bits of RGB; the twin of
/// `redact::palette::packed`, whose `-1` is no colour.
static bool redact_ink(float packed, thread float3 &ink) {
  if (!(packed >= 0.0) || packed > 16777215.0) return false;
  uint rgb = uint(packed);
  ink = float3(float((rgb >> 16u) & 0xffu), float((rgb >> 8u) & 0xffu),
               float(rgb & 0xffu)) / 255.0;
  return true;
}

/// One block's shade. A block is either the surface or the surface blended
/// towards one of its zone's inks by an amount the hash chooses, most of them
/// lightly: the spread pixelated text shows, in each word's own colours, with
/// none of its shapes. A zone with no ink of its own is plain surface. A box
/// packed without a picture has no zones, and the surface pulled towards the
/// contrasting end of the scale stands in.
static float3 redact_block(constant RedactUniforms &r, const device float2 *zones,
                           uint2 block) {
  uint hash = redact_hash(r.seed ^ redact_hash(block.x ^ redact_hash(block.y)));
  float amount = float(hash & 0xffffu) / 65535.0;
  float3 fill = float3(r.color.rgb);
  if (r.entry_count == 0u || r.grid[0] == 0u) {
    float luminance = dot(fill, float3(0.2126, 0.7152, 0.0722));
    float3 towards = luminance > 0.5 ? float3(0.0) : float3(1.0);
    return mix(fill, towards, 0.06 + 0.5 * amount * amount);
  }
  uint2 zone = block / max(r.grid[1], 1u);
  uint index = min(zone.y * r.grid[0] + min(zone.x, r.grid[0] - 1u), r.entry_count - 1u);
  float3 inks[2];
  uint count = 0u;
  if (redact_ink(zones[index].x, inks[count])) count += 1u;
  if (redact_ink(zones[index].y, inks[count])) count += 1u;
  if (count == 0u || float((hash >> 24u) & 0xffu) / 255.0 < redact_blank_share)
    return fill;
  return mix(fill, inks[((hash >> 16u) & 0xffu) % count], pow(amount, 1.5));
}

/// Where the cell grid starts, in the box's own pixels: centred on the box,
/// so the part cells its edges leave are shared out equally, and a grid whose
/// cells grow grows from the middle.
static float2 redact_grid_origin(constant RedactUniforms &r) {
  float2 box = float2(float(r.x1 - r.x0), float(r.y1 - r.y0));
  return (box - float2(float(r.grid[0]), float(r.grid[1])) * max(r.size, 1.0)) * 0.5;
}

/// The cell a pixel `local` pixels into the box falls in: the one division
/// both passes make, so a pixel is averaged into the cell it is painted from.
static uint2 redact_cell_of(constant RedactUniforms &r, uint2 local) {
  float2 cell = floor((float2(local) - redact_grid_origin(r)) / max(r.size, 1.0));
  uint2 last = uint2(max(r.grid[0], 1u), max(r.grid[1], 1u)) - 1u;
  return min(uint2(max(cell, 0.0)), last);
}

/// How far the seed moves each channel of a blurred cell's average, either
/// way: enough that the output never equals the averages of a guessed
/// original, too little to see.
constant uint redact_jitter = 6u;

/// An RGBA source buffer, as the cells pass reads it.
struct RedactRgbaPixels {
  const device uchar4 *pixels;
  uint width;
  uchar4 at(uint2 point) const { return pixels[point.y * width + point.x]; }
};

/// One cell's exact average, summed by a threadgroup of 256 over every
/// pixel in the cell, each laid over the surface first so a transparent one
/// counts as the surface it shows against. A blurred cell's average is then
/// nudged by the seed. Written as 24 bits of RGB, one word a cell.
template <typename Pixels>
static void redact_average(Pixels pixels, constant RedactUniforms &r, device uint *cells,
                           uint index, uint lane, threadgroup uint (*sums)[256]) {
  uint across = max(r.grid[0], 1u);
  uint2 cell = uint2(index % across, index / across);
  uint2 box = uint2(r.x1 - r.x0, r.y1 - r.y0);
  float size = max(r.size, 1.0);
  float2 origin = redact_grid_origin(r);
  // Every pixel the division could put in this cell, one either side of
  // where it nominally starts and ends.
  uint2 low = uint2(max(floor(float2(cell) * size + origin) - 1.0, 0.0));
  uint2 high = uint2(clamp(ceil(float2(cell + 1u) * size + origin) + 1.0, 0.0, float2(box)));
  uint2 span = select(uint2(0u), high - low, high > low);
  uint4 sum = uint4(0u);
  uint3 surface = uint3(round(saturate(float3(r.color.rgb)) * 255.0));
  for (uint at = lane; at < span.x * span.y; at += 256u) {
    uint2 local = low + uint2(at % span.x, at / span.x);
    if (any(redact_cell_of(r, local) != cell)) continue;
    uchar4 pixel = pixels.at(uint2(r.x0, r.y0) + local);
    uint alpha = uint(pixel.a);
    sum.rgb += (uint3(pixel.rgb) * alpha + surface * (255u - alpha) + 127u) / 255u;
    sum.a += 1u;
  }
  for (uint channel = 0u; channel < 4u; ++channel) sums[channel][lane] = sum[channel];
  threadgroup_barrier(mem_flags::mem_threadgroup);
  for (uint stride = 128u; stride > 0u; stride >>= 1u) {
    if (lane < stride)
      for (uint channel = 0u; channel < 4u; ++channel)
        sums[channel][lane] += sums[channel][lane + stride];
    threadgroup_barrier(mem_flags::mem_threadgroup);
  }
  if (lane != 0u) return;
  uint count = sums[3][0];
  uint3 average = count == 0u
      ? surface
      : (uint3(sums[0][0], sums[1][0], sums[2][0]) + count / 2u) / count;
  if (r.mode == 2u) {
    for (uint channel = 0u; channel < 3u; ++channel) {
      uint hash = redact_hash(r.seed ^ redact_hash(index ^ redact_hash(channel)));
      int moved = int(average[channel]) + int(hash % (2u * redact_jitter + 1u)) -
          int(redact_jitter);
      average[channel] = uint(clamp(moved, 0, 255));
    }
  }
  cells[index] = (average.r << 16u) | (average.g << 8u) | average.b;
}

/// The cells pass over an RGBA source: one threadgroup a cell.
kernel void redact_cells_rgba(
    const device uchar4 *pixels [[buffer(0)]],
    constant RedactUniforms &r [[buffer(1)]],
    device uint *cells [[buffer(3)]],
    uint index [[threadgroup_position_in_grid]],
    uint lane [[thread_index_in_threadgroup]]) {
  threadgroup uint sums[4][256];
  redact_average(RedactRgbaPixels{pixels, r.source_width}, r, cells, index, lane, sums);
}

/// One cell's averaged colour, the nearest cell standing in past the box's
/// edge so the blur never reaches outside it.
static float3 redact_cell(constant RedactUniforms &r, const device uint *cells, int2 cell) {
  int2 last = int2(int(r.grid[0]), int(r.grid[1])) - 1;
  cell = clamp(cell, int2(0), last);
  uint rgb = cells[uint(cell.y) * r.grid[0] + uint(cell.x)];
  return float3(float((rgb >> 16u) & 0xffu), float((rgb >> 8u) & 0xffu),
                float(rgb & 0xffu)) / 255.0;
}

/// The cubic B-spline's four weights a fraction `t` past a cell's centre:
/// smooth, and never negative, so the blur never rings past its cells.
static float4 redact_spline(float t) {
  float s = 1.0 - t;
  float t2 = t * t, t3 = t2 * t;
  return float4(s * s * s, 3.0 * t3 - 6.0 * t2 + 4.0,
                -3.0 * t3 + 3.0 * t2 + 3.0 * t + 1.0, t3) / 6.0;
}

/// The smooth surface through the cells' colours at `local`, a point in the
/// box's own pixels.
static float3 redact_blur(constant RedactUniforms &r, const device uint *cells,
                          float2 local) {
  float2 at = (local - redact_grid_origin(r)) / max(r.size, 1.0) - 0.5;
  float2 base = floor(at);
  float4 across = redact_spline(at.x - base.x);
  float4 down = redact_spline(at.y - base.y);
  float3 sum = float3(0.0);
  for (int row = 0; row < 4; ++row)
    for (int column = 0; column < 4; ++column)
      sum += across[column] * down[row] *
          redact_cell(r, cells, int2(base) + int2(column - 1, row - 1));
  return sum;
}

/// How much of the pixel centred at `local` the box takes: all of every
/// pixel its rounded outline touches, since a pixel's farthest point is half
/// its diagonal from its centre, then a soft edge one pixel wide beyond it.
/// So only a pixel wholly outside the outline is ever blended, and what shows
/// through it is picture the box never covered.
static float redact_cover(float2 local, float2 size, float radius) {
  if (radius <= 0.0) return 1.0;
  float2 q = abs(local - size * 0.5) - (size * 0.5 - radius);
  float distance = length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - radius;
  return saturate(1.70710678 - distance);
}

/// How much of the box's pixel `gid` a redaction takes: its rounded
/// outline's cover, and of that only the share an arriving fill has reached,
/// which rides in the colour's alpha.
static float redact_share(constant RedactUniforms &r, uint2 gid) {
  return redact_cover(float2(gid) + 0.5, float2(r.x1 - r.x0, r.y1 - r.y0), r.radius) *
      saturate(float(r.color.a));
}

/// The colour a redaction paints the box's pixel `gid` with.
static float3 redact_colour(constant RedactUniforms &r, const device float2 *entries,
                            const device uint *cells, uint2 gid) {
  if (r.mode == 1u) return redact_block(r, entries, uint2(float2(gid) / max(r.size, 1.0)));
  if (r.mode == 2u) return redact_blur(r, cells, float2(gid) + 0.5);
  if (r.mode == 3u) return redact_cell(r, cells, int2(redact_cell_of(r, gid)));
  return float3(r.color.rgb);
}

kernel void redact_source_rgba(
    device uchar4 *pixels [[buffer(0)]],
    constant RedactUniforms &r [[buffer(1)]],
    const device float2 *entries [[buffer(2)]],
    const device uint *cells [[buffer(3)]],
    uint2 gid [[thread_position_in_grid]]) {
  uint2 point = uint2(r.x0, r.y0) + gid;
  if (point.x >= r.x1 || point.y >= r.y1) return;
  float cover = redact_share(r, gid);
  if (cover <= 0.0) return;
  float3 rgb = redact_colour(r, entries, cells, gid);
  uint index = point.y * r.source_width + point.x;
  float4 painted = float4(round(saturate(rgb) * 255.0) / 255.0, 1.0);
  if (cover < 1.0) painted = mix(float4(pixels[index]) / 255.0, painted, cover);
  pixels[index] = uchar4(round(saturate(painted) * 255.0));
}
)METAL"
