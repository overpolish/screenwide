// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#define GPU_COMPOSITOR_MACOS_SHADER_SOURCE_CURSOR_OVERLAY_KERNELS @R"METAL(kernel void overlay_luma(texture2d_array<float, access::read> cursor [[texture(0)]],
                         texture2d<float, access::read_write> luma [[texture(1)]],
                         constant OverlayUniforms &u [[buffer(0)]],
                         uint2 gid [[thread_position_in_grid]]) {
  if (gid.x >= u.cursor_width || gid.y >= u.cursor_height) return;
  int2 output = int2(u.x, u.y) + int2(gid);
  if (output.x < 0 || output.y < 0 || output.x >= int(u.output_width) ||
      output.y >= int(u.output_height)) return;
  if (u.clip_at_video_edge != 0) {
    float2 crop_point = float2(output) + 0.5 - float2(u.crop_x, u.crop_y);
    float2 crop_size = float2(u.crop_width, u.crop_height);
    if (any(crop_point < 0.0) || any(crop_point >= crop_size) ||
        !rounded_pixel_visible(crop_point, crop_size, float(u.crop_radius))) return;
  }
  float4 rgba = cursor_pixel(cursor, u, float2(output) + 0.5);
  if (rgba.a <= 0.0001) return;
  float3 rgb = rgba.rgb;
  float cursor_y = 16.0 / 255.0 + dot(rgb, float3(0.182586, 0.614231, 0.062007));
  float existing = luma.read(uint2(output)).r;
  luma.write(mix(existing, cursor_y, rgba.a), uint2(output));
}

kernel void overlay_chroma(texture2d_array<float, access::read> cursor [[texture(0)]],
                           texture2d<float, access::read_write> chroma [[texture(1)]],
                           constant OverlayUniforms &u [[buffer(0)]],
                           uint2 gid [[thread_position_in_grid]]) {
  uint2 cursor_origin = gid * 2;
  if (cursor_origin.x >= u.cursor_width || cursor_origin.y >= u.cursor_height) return;
  int2 output_pixel = int2(u.x, u.y) + int2(cursor_origin);
  int2 output = output_pixel / 2;
  if (output.x < 0 || output.y < 0 || output.x >= int((u.output_width + 1) / 2) ||
      output.y >= int((u.output_height + 1) / 2)) return;
  if (u.clip_at_video_edge != 0) {
    float2 crop_point = float2(output_pixel) + 1.0 - float2(u.crop_x, u.crop_y);
    float2 crop_size = float2(u.crop_width, u.crop_height);
    if (any(crop_point < 0.0) || any(crop_point >= crop_size) ||
        !rounded_pixel_visible(crop_point, crop_size, float(u.crop_radius))) return;
  }
  // The chroma plane is half resolution, so one thread averages the four
  // luma-resolution cursor pixels it covers.
  float3 rgb_sum = 0.0;
  float alpha_sum = 0.0;
  for (uint y = 0; y < 2; ++y) {
    for (uint x = 0; x < 2; ++x) {
      float4 rgba =
          cursor_pixel(cursor, u, float2(output_pixel + int2(x, y)) + 0.5);
      rgb_sum += rgba.rgb * rgba.a;
      alpha_sum += rgba.a;
    }
  }
  float alpha = alpha_sum * 0.25;
  if (alpha <= 0.0001) return;
  float3 rgb = rgb_sum / max(alpha_sum, 0.0001);
  float2 cursor_uv = float2(
      0.5 + dot(rgb, float3(-0.100644, -0.338572, 0.439216)),
      0.5 + dot(rgb, float3(0.439216, -0.398942, -0.040274)));
  float2 existing = chroma.read(uint2(output)).rg;
  chroma.write(float4(mix(existing, cursor_uv, alpha), 0.0, 1.0), uint2(output));
}
)METAL"
