// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The cells pass: one pixel of the cells target a cell of the box, holding
// that cell's exact average. The twin of `redact_cells_rgba`.
#include "redact.hlsl"

// Every pixel in the cell, each laid over the surface first so a transparent
// one counts as the surface it shows against. A blurred cell's average is
// then nudged by the seed.
float4 ps_main(float4 position : SV_Position) : SV_Target {
  uint2 cell = uint2(position.xy);
  uint index = cell.y * max(redact_grid.x, 1u) + cell.x;
  uint2 box = redact_bounds.zw - redact_bounds.xy;
  float size = max(redact_size, 1.0);
  float2 origin = redact_grid_origin();
  // Every pixel the division could put in this cell, one either side of
  // where it nominally starts and ends.
  uint2 low = uint2(max(floor(float2(cell) * size + origin) - 1.0, 0.0));
  uint2 high = uint2(clamp(ceil(float2(cell + 1u) * size + origin) + 1.0, 0.0, float2(box)));
  uint2 span = high > low ? high - low : uint2(0u, 0u);
  uint3 surface = uint3(round(saturate(redact_color.rgb) * 255.0));
  uint3 sum = uint3(0u, 0u, 0u);
  uint count = 0u;
  [loop] for (uint y = 0u; y < span.y; ++y) {
    [loop] for (uint x = 0u; x < span.x; ++x) {
      uint2 local = low + uint2(x, y);
      if (any(redact_cell_of(local) != cell)) continue;
      uint4 pixel = uint4(round(saturate(redact_source.Load(int3(redact_bounds.xy + local, 0))) * 255.0));
      sum += (pixel.rgb * pixel.a + surface * (255u - pixel.a) + 127u) / 255u;
      count += 1u;
    }
  }
  uint3 average = count == 0u ? surface : (sum + count / 2u) / count;
  if (redact_mode == 2u) {
    [unroll] for (uint channel = 0u; channel < 3u; ++channel) {
      uint hash = redact_hash(redact_seed ^ redact_hash(index ^ redact_hash(channel)));
      int moved = int(average[channel]) + int(hash % (2u * redact_jitter + 1u)) - int(redact_jitter);
      average[channel] = uint(clamp(moved, 0, 255));
    }
  }
  return float4(float3(average) / 255.0, 1.0);
}
