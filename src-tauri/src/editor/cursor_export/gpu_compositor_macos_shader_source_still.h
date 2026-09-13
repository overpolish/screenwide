// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#define GPU_COMPOSITOR_MACOS_SHADER_SOURCE_STILL @R"METAL(    float2 point) {
  if (cursor.cursor.visible == 0) return 0.0;
  float blur = min(length(float2(cursor.cursor.blur_delta_x,
                                 cursor.cursor.blur_delta_y)), 80.0);
  float radius = length(float2(cursor.cursor.width, cursor.cursor.height)) *
                     cursor.cursor.scale + blur + 4.0;
  bool visible = all(abs(point - float2(cursor.cursor.x, cursor.cursor.y)) <=
                     radius);
  if (canvas.clip_cursor_at_video_edge != 0) {
    float2 crop_point = point - float2(canvas.crop_x, canvas.crop_y);
    float2 crop_size = float2(canvas.crop_width, canvas.crop_height);
    visible = visible && all(crop_point >= 0.0) &&
      all(crop_point < crop_size) && rounded_pixel_visible(
        crop_point, crop_size, float(canvas.radius));
  }
  return visible ? cursor_pixel(images, cursor, point) : float4(0.0);
}

/// Alpha-aware bilinear sampling preserves legacy RGBA overlays.
static float4 still_cursor_pixel(const device uchar4 *cursor, float2 destination_point, uint2 destination_size, uint2 source_size) {
  float2 point = clamp((destination_point + 0.5) * float2(source_size) / float2(destination_size) - 0.5, 0.0, float2(source_size - 1));
  uint2 low = uint2(floor(point)), high = min(low + 1, source_size - 1);
  float4 a = float4(cursor[low.y * source_size.x + low.x]) / 255.0, b = float4(cursor[low.y * source_size.x + high.x]) / 255.0;
  float4 c = float4(cursor[high.y * source_size.x + low.x]) / 255.0, d = float4(cursor[high.y * source_size.x + high.x]) / 255.0;
  a.rgb *= a.a; b.rgb *= b.a; c.rgb *= c.a; d.rgb *= d.a;
  float4 pixel = mix(mix(a, b, fract(point.x)), mix(c, d, fract(point.x)), fract(point.y));
  if (pixel.a > 0.0) pixel.rgb /= pixel.a; return pixel;
}

)METAL"
