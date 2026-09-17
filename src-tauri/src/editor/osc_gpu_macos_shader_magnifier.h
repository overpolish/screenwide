// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#ifndef SCREENWIDE_OSC_GPU_MACOS_SHADER_MAGNIFIER_H
#define SCREENWIDE_OSC_GPU_MACOS_SHADER_MAGNIFIER_H

/// The magnifier compute pass, and the library preamble every later part
/// of the source relies on.
#define SCREENWIDE_REGION_OSC_SHADER_MAGNIFIER @R"METAL(
#include <metal_stdlib>
using namespace metal;

struct region_magnifier_state {
  uint active;
  uint pane_index;
  uint layer_id;
  uint sample_camera;
  uint edges;
  uint light_mode;
  float sample_u;
  float sample_v;
  float source_min_u;
  float source_min_v;
  float source_max_u;
  float source_max_v;
  int box_x;
  int box_y;
  uint box_width;
  uint box_height;
};

kernel void region_magnifier(
    const device uchar4 *source [[buffer(0)]],
    texture2d<float, access::read_write> output [[texture(0)]],
    constant uint2 &source_dimensions [[buffer(1)]],
    constant region_magnifier_state &magnifier [[buffer(2)]],
    uint2 gid [[thread_position_in_grid]]) {
  if (magnifier.active == 0 || gid.x >= magnifier.box_width ||
      gid.y >= magnifier.box_height || any(source_dimensions == 0)) return;
  int2 output_point = int2(magnifier.box_x, magnifier.box_y) + int2(gid);
  if (any(output_point < 0) || output_point.x >= int(output.get_width()) ||
      output_point.y >= int(output.get_height())) return;
  float2 box_size = float2(magnifier.box_width, magnifier.box_height);
  float2 local = float2(gid) + 0.5;
  // The loupe box is 96 device-independent points wide and its corners are
  // the control radius, 8, so the backing scale falls out of the box size.
  float radius = max(box_size.x / 12.0, 1.0);
  float2 half_size = box_size * 0.5;
  float2 rounded = abs(local - half_size) - (half_size - radius);
  float distance = length(max(rounded, 0.0)) +
                   min(max(rounded.x, rounded.y), 0.0) - radius;
  // One device pixel of feathering on the outer edge. The render pass keeps
  // drawing the scene wherever this is below 1, using the same expression, so
  // the cutout and the loupe meet on exactly one edge.
  float coverage = 1.0 - smoothstep(-0.5, 0.5, distance);
  if (coverage <= 0.0) return;
  float2 source_center = float2(magnifier.sample_u, magnifier.sample_v) *
                         float2(source_dimensions);
  float2 source_point = source_center +
      (local / box_size - 0.5) * 40.0;
  int2 sample_point = int2(floor(source_point));
  float2 sample_uv = source_point / float2(source_dimensions);
  bool in_source = all(sample_point >= 0) &&
      all(sample_point < int2(source_dimensions)) &&
      all(sample_uv >= float2(magnifier.source_min_u,
                              magnifier.source_min_v)) &&
      all(sample_uv <= float2(magnifier.source_max_u,
                              magnifier.source_max_v));
  float4 pixel = in_source
      ? float4(source[uint(sample_point.y) * source_dimensions.x +
                      uint(sample_point.x)]) / 255.0
      : float4(0.15, 0.15, 0.16, 1.0);
  bool shade = ((magnifier.edges & 1u) != 0u && local.x < half_size.x) ||
               ((magnifier.edges & 2u) != 0u && local.x >= half_size.x) ||
               ((magnifier.edges & 4u) != 0u && local.y < half_size.y) ||
               ((magnifier.edges & 8u) != 0u && local.y >= half_size.y);
  if (shade) {
    float3 shade_color = magnifier.light_mode != 0
        ? float3(0.0) : float3(1.0);
    pixel.rgb = mix(pixel.rgb, shade_color, 0.1);
  }
  // The border is the bounding box's palette: a 1 px white core with a 1 px
  // dark hairline outside it, so the loupe reads over any desktop content.
  // Each boundary is feathered over the same one device pixel as the outer
  // edge, which is what keeps the corners from stepping.
  float core = smoothstep(-2.5, -1.5, distance);
  float hairline = smoothstep(-1.5, -0.5, distance);
  pixel.rgb = mix(pixel.rgb, float3(1.0), core);
  pixel.rgb = mix(pixel.rgb, float3(0.15, 0.15, 0.16), hairline);
  // The drawable holds premultiplied alpha, and the render pass composites
  // the scene over this with source-over blending.
  pixel = float4(pixel.rgb * coverage, coverage);
  output.write(pixel, uint2(output_point));
}
)METAL"

#endif
