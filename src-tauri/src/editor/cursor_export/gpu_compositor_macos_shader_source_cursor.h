// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#define GPU_COMPOSITOR_MACOS_SHADER_SOURCE_CURSOR @R"METAL(  return 1.0 - smoothstep(-0.75, 0.75, distance);
}

/// The crop tool's result layer, drawn over the uncropped ghost.
///
/// Crop mode shows the whole source so what is being cropped away stays
/// visible, which means the ghost underneath is flat: no rounding, no shadow.
/// The layer the crop actually produces is drawn a second time here, at its
/// place in the same canvas, carrying the radius and the shadow. It samples
/// the same source through the same image mapping, so the crop rectangle is
/// simply a rounded window onto pixels already in the right place.
static float crop_preview_shadow(float2 point, float2 dimensions,
                                 constant CanvasUniforms &u) {
  if (u.crop_preview_drop_shadow == 0) return 0.0;
  float2 origin = float2(u.crop_preview_x, u.crop_preview_y);
  float2 size = float2(u.crop_preview_width, u.crop_preview_height);
  float sigma = margin_capped_sigma(origin, size, dimensions,
                                    shadow_sigma(size));
  if (sigma <= 1.0) return 0.0;
  return soft_shadow(point - origin, size, u.crop_preview_radius, sigma, 0.14);
}

static float4 crop_preview_rgba(float4 result, const device uchar4 *source,
                                uint source_width, uint source_height,
                                float2 point, float2 dimensions,
                                constant CanvasUniforms &u) {
  if (u.crop_preview == 0 || u.crop_preview_width <= 0.0 ||
      u.crop_preview_height <= 0.0)
    return result;
  float coverage = rounded_coverage(
    point - float2(u.crop_preview_x, u.crop_preview_y),
    float2(u.crop_preview_width, u.crop_preview_height),
    u.crop_preview_radius);
  // The shadow belongs to what lies outside the layer, so the layer itself is
  // never tinted by it.
  float shadow = crop_preview_shadow(point, dimensions, u);
  if (shadow > 0.0) result.rgb *= 1.0 - shadow * (1.0 - coverage);
  if (coverage > 0.0) {
    float4 pixel = rgba_source_pixel(source, source_width, source_height,
                                     point, u);
    float alpha = pixel.a * coverage;
    result.rgb = pixel.rgb * alpha + result.rgb * (1.0 - alpha);
    result.a = alpha + result.a * (1.0 - alpha);
  }
  return result;
}

static float4 canvas_rgba_pixel(const device uchar4 *source,
                                uint source_width, uint source_height,
                                float2 point, float2 dimensions,
                                constant CanvasUniforms &u, float seconds,
                                texture2d<float, access::sample> picture) {
  float3 background = canvas_background(picture, point, dimensions, u, seconds);
  float background_alpha = u.foreground_only != 0 ? 0.0 : 1.0;
  float2 crop_point = point - float2(u.crop_x, u.crop_y), crop_size = float2(u.crop_width, u.crop_height);
  float2 crop_origin = float2(u.crop_x, u.crop_y);
  float2 image_origin = float2(u.image_x, u.image_y), image_size = float2(u.image_width, u.image_height);
  float2 source_crop_origin = float2(u.source_crop_x, u.source_crop_y), source_crop_size = float2(u.source_crop_width, u.source_crop_height);
  float crop_coverage = rounded_coverage(crop_point, crop_size, float(u.radius));
  float2 image_point = point - image_origin;
  float image_coverage = crop_coverage *
    rounded_coverage(image_point, image_size, 0.0) *
    rounded_coverage(point - source_crop_origin, source_crop_size, 0.0);
  float frame_coverage = u.recenter_inset_color.a > 0.0 ? crop_coverage : image_coverage;
  float2 shadow_origin = u.recenter_inset_color.a > 0.0 ? crop_origin : source_crop_origin, shadow_size = u.recenter_inset_color.a > 0.0 ? crop_size : source_crop_size;
  if (u.drop_shadow != 0) {
    float sigma = visible_foreground_sigma(crop_origin, crop_size, shadow_origin, shadow_size, dimensions);
    if (sigma > 1.0) {
      float shadow = visible_foreground_shadow(point, crop_origin, crop_size,
        float(u.radius), shadow_origin, shadow_size, sigma, 0.14);
      if (u.foreground_only != 0) {
        background = float3(0.0);
        background_alpha = shadow * (1.0 - frame_coverage);
      } else {
        background *= 1.0 - shadow * (1.0 - frame_coverage);
      }
    }
  }
  float4 result = float4(background * background_alpha, background_alpha);
  if (u.recenter_inset_color.a > 0.0)
    result = mix(result, float4(u.recenter_inset_color.rgb, 1.0), crop_coverage);
  if (image_coverage > 0.0) {
    float4 video = rgba_source_pixel(source, source_width, source_height, point, u);
    float source_alpha = video.a * image_coverage;
    result.rgb = video.rgb * source_alpha + result.rgb * (1.0 - source_alpha);
    result.a = source_alpha + result.a * (1.0 - source_alpha);
  }
  return crop_preview_rgba(result, source, source_width, source_height, point,
                           dimensions, u);
}

static float4 overlay_canvas_foreground_rgba(
    float4 result, const device uchar4 *source, uint source_width,
    uint source_height, float2 point, float2 dimensions,
    constant CanvasUniforms &u) {
  float2 crop_origin = float2(u.crop_x, u.crop_y), crop_size = float2(u.crop_width, u.crop_height);
  float2 image_origin = float2(u.image_x, u.image_y), image_size = float2(u.image_width, u.image_height);
  float2 source_crop_origin = float2(u.source_crop_x, u.source_crop_y), source_crop_size = float2(u.source_crop_width, u.source_crop_height);
  float crop_coverage = rounded_coverage(
    point - crop_origin, crop_size, float(u.radius));
  float image_coverage = crop_coverage * rounded_coverage(
    point - image_origin, image_size, 0.0) * rounded_coverage(
    point - source_crop_origin, source_crop_size, 0.0);
  float frame_coverage = u.recenter_inset_color.a > 0.0 ? crop_coverage : image_coverage;
  float2 shadow_origin = u.recenter_inset_color.a > 0.0 ? crop_origin : source_crop_origin, shadow_size = u.recenter_inset_color.a > 0.0 ? crop_size : source_crop_size;
  if (u.drop_shadow != 0) {
    float sigma = visible_foreground_sigma(crop_origin, crop_size, shadow_origin, shadow_size, dimensions);
    if (sigma > 1.0) {
      float shadow = visible_foreground_shadow(point, crop_origin, crop_size, float(u.radius), shadow_origin, shadow_size, sigma, 0.14);
      result.rgb *= 1.0 - shadow * (1.0 - frame_coverage);
    }
  }
  if (u.recenter_inset_color.a > 0.0)
    result = mix(result, float4(u.recenter_inset_color.rgb, 1.0), crop_coverage);
  if (image_coverage > 0.0) {
    float4 video = rgba_source_pixel(
      source, source_width, source_height, point, u);
    float source_alpha = video.a * image_coverage;
    result.rgb = video.rgb * source_alpha + result.rgb * (1.0 - source_alpha);
    result.a = source_alpha + result.a * (1.0 - source_alpha);
  }
  return crop_preview_rgba(result, source, source_width, source_height, point,
                           dimensions, u);
}

static float4 cursor_pixel(
    texture2d_array<float, access::read> images,
    constant OverlayUniforms &u, float2 point);

static float4 canvas_cursor_pixel(
    texture2d_array<float, access::read> images,
    constant OverlayUniforms &cursor, constant CanvasUniforms &canvas,
)METAL"
