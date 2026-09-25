// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The paint pass: one pixel of the box, drawn over a copy of what the last
// pass left. The twin of `redact_source_rgba`.
#include "redact.hlsl"

// One cell's averaged colour, the nearest cell standing in past the box's
// edge so the blur never reaches outside it.
float3 redact_cell(int2 cell) {
  int2 last = int2(redact_grid) - 1;
  return redact_cells.Load(int3(clamp(cell, int2(0, 0), last), 0)).rgb;
}

// The cubic B-spline's four weights a fraction `t` past a cell's centre:
// smooth, and never negative, so the blur never rings past its cells.
float4 redact_spline(float t) {
  float s = 1.0 - t;
  float t2 = t * t, t3 = t2 * t;
  return float4(s * s * s, 3.0 * t3 - 6.0 * t2 + 4.0,
                -3.0 * t3 + 3.0 * t2 + 3.0 * t + 1.0, t3) / 6.0;
}

// The smooth surface through the cells' colours at `local`, a point in the
// box's own pixels.
float3 redact_blur(float2 local) {
  float2 at = (local - redact_grid_origin()) / max(redact_size, 1.0) - 0.5;
  float2 base = floor(at);
  float4 across = redact_spline(at.x - base.x);
  float4 down = redact_spline(at.y - base.y);
  float3 sum = float3(0.0, 0.0, 0.0);
  [unroll] for (int row = 0; row < 4; ++row)
    [unroll] for (int column = 0; column < 4; ++column)
      sum += across[column] * down[row] *
          redact_cell(int2(base) + int2(column - 1, row - 1));
  return sum;
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
  if (redact_mode == 2u) return redact_blur(float2(gid) + 0.5);
  if (redact_mode == 3u) return redact_cell(int2(redact_cell_of(gid)));
  return redact_color.rgb;
}

// The viewport is the box, so every pixel drawn is one of its own. A pixel
// the box does not take is discarded, which leaves the copy's own there.
float4 ps_main(float4 position : SV_Position) : SV_Target {
  uint2 pixel = uint2(position.xy);
  if (any(pixel < redact_bounds.xy) || any(pixel >= redact_bounds.zw)) discard;
  uint2 gid = pixel - redact_bounds.xy;
  // Its rounded outline's cover, and of that only the share an arriving fill
  // has reached, which rides in the colour's alpha.
  float cover = redact_cover(float2(gid) + 0.5, float2(redact_bounds.zw - redact_bounds.xy)) *
      saturate(redact_color.a);
  if (cover <= 0.0) discard;
  float4 painted = float4(round(saturate(redact_colour(gid)) * 255.0) / 255.0, 1.0);
  if (cover < 1.0) painted = lerp(redact_source.Load(int3(pixel, 0)), painted, cover);
  return painted;
}
