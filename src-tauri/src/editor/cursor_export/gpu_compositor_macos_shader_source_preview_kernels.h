// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#define GPU_COMPOSITOR_MACOS_SHADER_SOURCE_PREVIEW_KERNELS @R"METAL(kernel void unpack_preview_bgra(
    texture2d<float, access::read> source [[texture(0)]],
    device uchar4 *output [[buffer(0)]],
    uint2 gid [[thread_position_in_grid]]) {
  uint2 dimensions(source.get_width(), source.get_height());
  if (any(gid >= dimensions)) return;
  output[gid.y * dimensions.x + gid.x] = uchar4(
    clamp(source.read(gid), 0.0, 1.0) * 255.0 + 0.5);
}

static float3 canvas_pixel(texture2d<float, access::sample> source_y,
                           texture2d<float, access::sample> source_uv,
                           float2 point, float2 dimensions,
                           constant CanvasUniforms &u, float seconds,
                           texture2d<float, access::sample> picture) {
  float3 background = canvas_background(picture, point, dimensions, u, seconds);
  float canvas_coverage = rounded_coverage(
    point, dimensions, float(u.background_radius));
  float2 crop_point = point - float2(u.crop_x, u.crop_y), crop_size = float2(u.crop_width, u.crop_height);
  float2 crop_origin = float2(u.crop_x, u.crop_y);
  float2 image_origin = float2(u.image_x, u.image_y), image_size = float2(u.image_width, u.image_height);
  float2 source_crop_origin = float2(u.source_crop_x, u.source_crop_y), source_crop_size = float2(u.source_crop_width, u.source_crop_height);
  float crop_coverage = rounded_coverage(crop_point, crop_size, float(u.radius));
  float2 image_point = point - image_origin;
  float image_coverage = crop_coverage *
    rounded_coverage(image_point, image_size, 0.0) *
    rounded_coverage(point - source_crop_origin, source_crop_size, 0.0);
  bool has_inset = u.recenter_inset_color.a > 0.0; float frame_coverage = has_inset ? crop_coverage : image_coverage;
  float2 shadow_origin = has_inset ? crop_origin : source_crop_origin, shadow_size = has_inset ? crop_size : source_crop_size;
  if (u.drop_shadow != 0) {
    float sigma = visible_foreground_sigma(crop_origin, crop_size, shadow_origin, shadow_size, dimensions);
    if (sigma > 1.0) {
      float shadow = visible_foreground_shadow(point, crop_origin, crop_size,
        float(u.radius), shadow_origin, shadow_size, sigma, 0.14);
      background *= 1.0 - shadow * (1.0 - frame_coverage);
    }
  }
  float3 result = background; if (has_inset) result = mix(result, u.recenter_inset_color.rgb, crop_coverage);
  if (image_coverage > 0.0) result = mix(result,
    source_pixel(source_y, source_uv, point, u), image_coverage);
  return result * canvas_coverage;
}

kernel void compose_canvas_luma(
    texture2d<float, access::sample> source_y [[texture(0)]],
    texture2d<float, access::sample> source_uv [[texture(1)]],
    texture2d<float, access::write> output [[texture(2)]],
    constant CanvasUniforms &u [[buffer(0)]], constant float &seconds [[buffer(1)]],
    texture2d<float, access::sample> background_picture [[texture(3)]],
    uint2 gid [[thread_position_in_grid]]) {
  uint2 dimensions(output.get_width(), output.get_height());
  if (any(gid >= dimensions)) return;
  float3 rgb = canvas_pixel(source_y, source_uv, float2(gid) + 0.5,
                            float2(dimensions), u, seconds,
                            background_picture);
  rgb = output_dither(rgb, float2(gid));
  output.write(16.0 / 255.0 + dot(rgb, float3(0.182586, 0.614231, 0.062007)), gid);
}

kernel void compose_canvas_chroma(
    texture2d<float, access::sample> source_y [[texture(0)]],
    texture2d<float, access::sample> source_uv [[texture(1)]],
    texture2d<float, access::write> output [[texture(2)]],
    constant CanvasUniforms &u [[buffer(0)]], constant float &seconds [[buffer(1)]],
    texture2d<float, access::sample> background_picture [[texture(3)]],
    uint2 gid [[thread_position_in_grid]]) {
  uint2 dimensions(output.get_width(), output.get_height());
  if (any(gid >= dimensions)) return;
  float3 rgb = 0.0;
  for (uint y = 0; y < 2; ++y)
    for (uint x = 0; x < 2; ++x)
      rgb += canvas_pixel(source_y, source_uv, float2(gid * 2 + uint2(x, y)) + 0.5,
                          float2(dimensions * 2), u, seconds,
                          background_picture);
  rgb *= 0.25;
  rgb = output_dither(rgb, float2(gid * 2));
  output.write(float4(
    0.5 + dot(rgb, float3(-0.100644, -0.338572, 0.439216)),
    0.5 + dot(rgb, float3(0.439216, -0.398942, -0.040274)), 0.0, 1.0), gid);
}

kernel void overlay_screen_luma(
    texture2d<float, access::sample> source_y [[texture(0)]],
    texture2d<float, access::sample> source_uv [[texture(1)]],
    texture2d<float, access::read_write> luma [[texture(2)]],
    constant CanvasUniforms &u [[buffer(0)]],
    uint2 gid [[thread_position_in_grid]]) {
  uint2 dimensions(luma.get_width(), luma.get_height());
  if (any(gid >= dimensions)) return;
  float2 point = float2(gid) + 0.5;
  float2 crop_origin = float2(u.crop_x, u.crop_y);
  float2 crop_size = float2(u.crop_width, u.crop_height);
  float2 image_origin = float2(u.image_x, u.image_y);
  float2 image_size = float2(u.image_width, u.image_height);
  float coverage = rounded_coverage(point - crop_origin, crop_size, float(u.radius)) *
    rounded_coverage(point - image_origin, image_size, 0.0);
  float existing = luma.read(gid).r;
  if (u.drop_shadow != 0) {
    float sigma = visible_foreground_sigma(
      crop_origin, crop_size, image_origin, image_size, float2(dimensions));
    if (sigma > 1.0) {
      float shadow = visible_foreground_shadow(
        point, crop_origin, crop_size, float(u.radius), image_origin,
        image_size, sigma, 0.14);
      existing = mix(existing, 16.0 / 255.0, shadow * (1.0 - coverage));
    }
  }
  if (coverage > 0.0) {
    float3 rgb = source_pixel(source_y, source_uv, point, u);
    float value = 16.0 / 255.0 + dot(rgb, float3(0.182586, 0.614231, 0.062007));
    existing = mix(existing, value, coverage);
  }
  luma.write(existing, gid);
}

kernel void overlay_screen_chroma(
    texture2d<float, access::sample> source_y [[texture(0)]],
    texture2d<float, access::sample> source_uv [[texture(1)]],
    texture2d<float, access::read_write> chroma [[texture(2)]],
    constant CanvasUniforms &u [[buffer(0)]],
    uint2 gid [[thread_position_in_grid]]) {
  uint2 dimensions(chroma.get_width(), chroma.get_height());
  if (any(gid >= dimensions)) return;
  float2 output_dimensions = float2(dimensions * 2);
  float2 crop_origin = float2(u.crop_x, u.crop_y);
  float2 crop_size = float2(u.crop_width, u.crop_height);
  float2 image_origin = float2(u.image_x, u.image_y);
  float2 image_size = float2(u.image_width, u.image_height);
  float3 rgb_sum = 0.0;
  float coverage_sum = 0.0;
  float shadow_sum = 0.0;
  for (uint y = 0; y < 2; ++y) {
    for (uint x = 0; x < 2; ++x) {
      float2 point = float2(gid * 2 + uint2(x, y)) + 0.5;
      float coverage = rounded_coverage(
        point - crop_origin, crop_size, float(u.radius)) *
        rounded_coverage(point - image_origin, image_size, 0.0);
      if (u.drop_shadow != 0) {
        float sigma = visible_foreground_sigma(
          crop_origin, crop_size, image_origin, image_size, output_dimensions);
        if (sigma > 1.0) {
          shadow_sum += visible_foreground_shadow(
            point, crop_origin, crop_size, float(u.radius), image_origin,
            image_size, sigma, 0.14) * (1.0 - coverage);
        }
      }
      if (coverage > 0.0) {
        rgb_sum += source_pixel(source_y, source_uv, point, u) * coverage;
        coverage_sum += coverage;
      }
    }
  }
  float2 existing = chroma.read(gid).rg;
  float shadow = shadow_sum * 0.25;
  if (shadow > 0.0) existing = mix(existing, float2(0.5), shadow);
  float coverage = coverage_sum * 0.25;
  if (coverage > 0.0) {
    float3 rgb = rgb_sum / max(coverage_sum, 0.0001);
    float2 value = float2(
      0.5 + dot(rgb, float3(-0.100644, -0.338572, 0.439216)),
      0.5 + dot(rgb, float3(0.439216, -0.398942, -0.040274)));
    existing = mix(existing, value, coverage);
  }
  chroma.write(float4(existing, 0.0, 1.0), gid);
}

)METAL"
