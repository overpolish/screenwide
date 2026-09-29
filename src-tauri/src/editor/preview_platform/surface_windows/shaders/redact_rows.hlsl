// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The rows pass: one pixel of the rows target a pixel of a blurred box,
// holding the Gaussian along its row. The twin of `redact_rows_rgba`.
#include "redact.hlsl"

// The row's taps are the box's own pixels only, so nothing outside the box
// reaches in. A redaction's blur lays each pixel over the surface first, so a
// transparent one counts as the surface it shows against. The spotlights'
// blur has no surface to lay one over - what it softens keeps its own alpha -
// so it weighs each pixel by its alpha instead. Written premultiplied, its
// weight in alpha, for the paint pass to finish.
float4 ps_main(float4 position : SV_Position) : SV_Target {
  int2 local = int2(position.xy);
  int wide = (int)(redact_bounds.z - redact_bounds.x);
  int reach = redact_reach(redact_size);
  float3 surface = saturate(redact_color.rgb);
  float4 sum = float4(0.0, 0.0, 0.0, 0.0);
  float total = 0.0;
  int last = min(local.x + reach, wide - 1);
  [loop] for (int x = max(local.x - reach, 0); x <= last; ++x) {
    float weight = redact_weight(x - local.x, redact_size);
    float4 pixel = saturate(redact_source.Load(int3(int2(redact_bounds.xy) + int2(x, local.y), 0)));
    sum += weight * (redact_mode == 4u ? float4(pixel.rgb * pixel.a, pixel.a)
                                       : float4(lerp(surface, pixel.rgb, pixel.a), 1.0));
    total += weight;
  }
  return sum / total;
}
