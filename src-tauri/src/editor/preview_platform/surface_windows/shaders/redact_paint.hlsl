// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The paint pass: one pixel of the box, drawn over a copy of what the last
// pass left. The twin of `redact_source_rgba`.
#include "redact.hlsl"

// One cell's averaged colour, the nearest cell standing in past the box's
// edge.
float3 redact_cell(int2 cell) {
  int2 last = int2(redact_grid) - 1;
  return redact_cells.Load(int3(clamp(cell, int2(0, 0), last), 0)).rgb;
}

// The blur's second pass at the box's pixel `gid`: the Gaussian down its
// column of what the rows pass wrote, within the box, back out of
// premultiplied colour.
float3 redact_blur(uint2 gid) {
  int tall = (int)(redact_bounds.w - redact_bounds.y);
  int reach = redact_reach(redact_size);
  float4 sum = float4(0.0, 0.0, 0.0, 0.0);
  float total = 0.0;
  int last = min((int)gid.y + reach, tall - 1);
  [loop] for (int y = max((int)gid.y - reach, 0); y <= last; ++y) {
    float weight = redact_weight(y - (int)gid.y, redact_size);
    sum += weight * redact_rows.Load(int3((int)gid.x, y, 0));
    total += weight;
  }
  float4 average = sum / total;
  return average.a > 0.0 ? average.rgb / average.a : float3(0.0, 0.0, 0.0);
}

// How much of the pixel centred at `local` the box takes: all of every pixel
// its rounded outline touches, since a pixel's farthest point is half its
// diagonal from its centre, then a soft edge one pixel wide beyond it. So
// only a pixel wholly outside the outline is ever blended, and what shows
// through it is picture the box never covered.
float redact_cover(float2 local, float2 size) {
  if (redact_radius <= 0.0) return 1.0;
  float2 q = abs(local - size * 0.5) - (size * 0.5 - redact_radius);
  float distance = length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - redact_radius;
  return saturate(1.70710678 - distance);
}

// The colour the box's pixel `gid` is painted with.
float3 redact_colour(uint2 gid) {
  if (redact_mode == 1u) return redact_block(uint2(float2(gid) / max(redact_size, 1.0)));
  if (redact_mode == 2u || redact_mode == 4u) return redact_blur(gid);
  if (redact_mode == 3u) return redact_cell(int2(redact_cell_of(gid)));
  return redact_color.rgb;
}

// How much of the pixel centred at `point` the spotlights' blur takes: as
// much as the record's cover, which rides in the colour's alpha, lifted by
// each spotlight's light as far as it is present. The zones are the holes,
// four each: the box's corners, its rounding and fade, and its presence
// beside the blur's, in source pixels. The twin of `redact_spotlight_share`.
float redact_spotlight_share(float2 point) {
  float lit = 0.0;
  [loop] for (uint at = 0u; at + 3u < redact_entry_count; at += 4u) {
    uint first = redact_zone_first + at;
    float2 low = redact_zones[first];
    float2 high = redact_zones[first + 1u];
    float2 shape = redact_zones[first + 2u];
    float presence = saturate(redact_zones[first + 3u].x);
    float rounding = min(shape.x, min(high.x - low.x, high.y - low.y) * 0.5);
    float2 q = abs(point - (low + high) * 0.5) - ((high - low) * 0.5 - rounding);
    float distance = length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - rounding;
    float soft = max(shape.y, 0.0);
    float fall = saturate((distance + soft + 0.5) / (soft + 1.0));
    lit = max(lit, presence * (1.0 - fall * fall * (3.0 - 2.0 * fall)));
  }
  return max(saturate(redact_color.a) - lit, 0.0);
}

// The viewport is the box, so every pixel drawn is one of its own. A pixel
// the box does not take is discarded, which leaves the copy's own there.
float4 ps_main(float4 position : SV_Position) : SV_Target {
  uint2 pixel = uint2(position.xy);
  if (any(pixel < redact_bounds.xy) || any(pixel >= redact_bounds.zw)) discard;
  uint2 gid = pixel - redact_bounds.xy;
  // Its rounded outline's cover, and of that only the share an arriving fill
  // has reached, which rides in the colour's alpha. The spotlights' blur
  // takes what its holes leave.
  float cover = redact_mode == 4u
      ? redact_spotlight_share(float2(pixel) + 0.5)
      : redact_cover(float2(gid) + 0.5, float2(redact_bounds.zw - redact_bounds.xy)) *
          saturate(redact_color.a);
  if (cover <= 0.0) discard;
  float4 original = redact_source.Load(int3(pixel, 0));
  // What the spotlights' blur softens keeps its own alpha: it is the same
  // picture, only less sharp.
  float4 painted = float4(round(saturate(redact_colour(gid)) * 255.0) / 255.0,
                          redact_mode == 4u ? original.a : 1.0);
  if (cover < 1.0) painted = lerp(original, painted, cover);
  return painted;
}
