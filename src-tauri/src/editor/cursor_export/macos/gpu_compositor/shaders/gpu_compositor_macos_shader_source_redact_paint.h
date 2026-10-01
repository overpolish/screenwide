// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

/// A redaction's blur and its paint pass, after what `..._redact.h` shares.
/// A blur, a redaction's or the spotlights', is a Gaussian over the box's own
/// pixels in two passes: the rows pass blurs along each row into a buffer,
/// and the paint pass blurs down each column of that buffer as it paints.
/// Its width is the record's `size`, a standard deviation, which widens from
/// nothing as the blur arrives, so an arriving blur softens smoothly rather
/// than laying a finished one over the sharp picture.
#define GPU_COMPOSITOR_MACOS_SHADER_SOURCE_REDACT_PAINT @R"METAL(
/// How many pixels either side of its centre a blur of deviation `sigma`
/// reaches: three deviations, past which a tap weighs about a hundredth of
/// the centre's. Held to what one pass can afford.
static int redact_reach(float sigma) {
  return int(min(ceil(max(sigma, 0.0) * 3.0), 160.0));
}

/// `-1 / (2 sigma^2)`, which a tap's squared offset scales into its weight.
static float redact_fall(float sigma) {
  return -0.5 / max(sigma * sigma, 1e-6);
}

/// The Gaussian's weight `offset` pixels from its centre.
static float redact_weight(int offset, float fall) {
  float along = float(offset);
  return exp(along * along * fall);
}

/// The blur's first pass at the box's pixel `gid`: the Gaussian along its
/// row, over the box's own pixels only, so nothing outside the box reaches
/// in. A redaction's blur lays each pixel over the surface first, so a
/// transparent one counts as the surface it shows against. The spotlights'
/// blur has no surface to lay one over - what it softens keeps its own
/// alpha - so it weighs each pixel by its alpha instead. Written
/// premultiplied, its weight in alpha, for the paint pass to finish.
template <typename Pixels>
static void redact_blur_row(Pixels pixels, constant RedactUniforms &r, device half4 *rows,
                            uint2 gid) {
  uint2 box = uint2(r.x1 - r.x0, r.y1 - r.y0);
  if (gid.x >= box.x || gid.y >= box.y) return;
  int reach = redact_reach(r.size);
  float fall = redact_fall(r.size);
  float3 surface = saturate(float3(r.color.rgb));
  float4 sum = float4(0.0);
  float total = 0.0;
  int last = min(int(gid.x) + reach, int(box.x) - 1);
  for (int x = max(int(gid.x) - reach, 0); x <= last; ++x) {
    float weight = redact_weight(x - int(gid.x), fall);
    float4 pixel = float4(pixels.at(uint2(r.x0 + uint(x), r.y0 + gid.y))) / 255.0;
    sum += weight * (r.mode == 4u ? float4(pixel.rgb * pixel.a, pixel.a)
                                  : float4(mix(surface, pixel.rgb, pixel.a), 1.0));
    total += weight;
  }
  rows[gid.y * box.x + gid.x] = half4(sum / total);
}

/// The rows pass over an RGBA source: one thread a pixel of the box.
kernel void redact_rows_rgba(
    const device uchar4 *pixels [[buffer(0)]],
    constant RedactUniforms &r [[buffer(1)]],
    device half4 *rows [[buffer(4)]],
    uint2 gid [[thread_position_in_grid]]) {
  redact_blur_row(RedactRgbaPixels{pixels, r.source_width}, r, rows, gid);
}

/// The blur's second pass at the box's pixel `gid`: the Gaussian down its
/// column of what the rows pass wrote, within the box, back out of
/// premultiplied colour.
static float3 redact_blur(constant RedactUniforms &r, const device half4 *rows, uint2 gid) {
  uint width = r.x1 - r.x0;
  int reach = redact_reach(r.size);
  float fall = redact_fall(r.size);
  float4 sum = float4(0.0);
  float total = 0.0;
  int last = min(int(gid.y) + reach, int(r.y1 - r.y0) - 1);
  for (int y = max(int(gid.y) - reach, 0); y <= last; ++y) {
    float weight = redact_weight(y - int(gid.y), fall);
    sum += weight * float4(rows[uint(y) * width + gid.x]);
    total += weight;
  }
  float4 average = sum / total;
  return average.a > 0.0 ? average.rgb / average.a : float3(0.0);
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

/// How much of the pixel centred at `local` the spotlights' blur takes: as
/// much as the record's cover, which rides in the colour's alpha, lifted by
/// each spotlight's light as far as it is present. The entries are the holes,
/// four each: the box's corners, its rounding and fade, and its presence
/// beside the blur's, in source pixels. The twin of Rust's
/// `spotlight::geometry::{light, shade}`.
static float redact_spotlight_share(constant RedactUniforms &r, const device float2 *holes,
                                    float2 local) {
  float2 point = local + float2(float(r.x0), float(r.y0));
  float lit = 0.0;
  for (uint at = 0u; at + 3u < r.entry_count; at += 4u) {
    float2 low = float2(holes[at]), high = float2(holes[at + 1u]);
    float2 shape = float2(holes[at + 2u]);
    float presence = saturate(holes[at + 3u].x);
    float rounding = min(shape.x, min(high.x - low.x, high.y - low.y) * 0.5);
    float2 q = abs(point - (low + high) * 0.5) - ((high - low) * 0.5 - rounding);
    float distance = length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - rounding;
    float soft = max(shape.y, 0.0);
    float fall = saturate((distance + soft + 0.5) / (soft + 1.0));
    lit = max(lit, presence * (1.0 - fall * fall * (3.0 - 2.0 * fall)));
  }
  return max(saturate(float(r.color.a)) - lit, 0.0);
}

/// How much of the box's pixel `gid` a redaction takes: its rounded
/// outline's cover, and of that only the share an arriving fill has reached,
/// which rides in the colour's alpha. The spotlights' blur takes what its
/// holes leave.
static float redact_share(constant RedactUniforms &r, const device float2 *entries,
                          uint2 gid) {
  if (r.mode == 4u) return redact_spotlight_share(r, entries, float2(gid) + 0.5);
  return redact_cover(float2(gid) + 0.5, float2(r.x1 - r.x0, r.y1 - r.y0), r.radius) *
      saturate(float(r.color.a));
}

/// The colour a redaction paints the box's pixel `gid` with.
static float3 redact_colour(constant RedactUniforms &r, const device float2 *entries,
                            const device uint *cells, const device half4 *rows, uint2 gid) {
  if (r.mode == 1u) return redact_block(r, entries, uint2(float2(gid) / max(r.size, 1.0)));
  if (r.mode == 2u || r.mode == 4u) return redact_blur(r, rows, gid);
  if (r.mode == 3u) return redact_cell(r, cells, int2(redact_cell_of(r, gid)));
  return float3(r.color.rgb);
}

kernel void redact_source_rgba(
    device uchar4 *pixels [[buffer(0)]],
    constant RedactUniforms &r [[buffer(1)]],
    const device float2 *entries [[buffer(2)]],
    const device uint *cells [[buffer(3)]],
    const device half4 *rows [[buffer(4)]],
    uint2 gid [[thread_position_in_grid]]) {
  uint2 point = uint2(r.x0, r.y0) + gid;
  if (point.x >= r.x1 || point.y >= r.y1) return;
  float cover = redact_share(r, entries, gid);
  if (cover <= 0.0) return;
  float3 rgb = redact_colour(r, entries, cells, rows, gid);
  uint index = point.y * r.source_width + point.x;
  float4 original = float4(pixels[index]) / 255.0;
  // What the spotlights' blur softens keeps its own alpha: it is the same
  // picture, only less sharp.
  float4 painted = float4(round(saturate(rgb) * 255.0) / 255.0,
                          r.mode == 4u ? original.a : 1.0);
  if (cover < 1.0) painted = mix(original, painted, cover);
  pixels[index] = uchar4(round(saturate(painted) * 255.0));
}
)METAL"
