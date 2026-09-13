// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#define GPU_COMPOSITOR_MACOS_SHADER_SOURCE_CAMERA_KERNELS @R"METAL(static float4 camera_pixel(texture2d<float, access::sample> camera,
                           float2 point, constant CameraUniforms &u) {
  float coverage = rounded_coverage(
    point, float2(u.frame_width, u.frame_height), float(u.radius));
  if (coverage <= 0.0) return float4(0.0);
  constexpr sampler linear_sampler(coord::normalized, address::clamp_to_edge,
                                   filter::linear);
  float2 source = float2(u.crop_x, u.crop_y) +
                  point * float2(u.crop_width, u.crop_height) /
                      float2(u.frame_width, u.frame_height);
  return float4(camera.sample(linear_sampler,
                              source / float2(u.source_width, u.source_height)).rgb,
                coverage);
}

kernel void alpha_composite_rgba(
    const device uchar4 *base [[buffer(0)]],
    const device uchar4 *overlay [[buffer(1)]],
    device uchar4 *output [[buffer(2)]],
    uint gid [[thread_position_in_grid]],
    uint count [[threads_per_grid]]) {
  if (gid >= count) return;
  float4 below = float4(base[gid]) / 255.0;
  float4 above = float4(overlay[gid]) / 255.0;
  float inverse = 1.0 - above.a;
  float4 result = float4(
    above.rgb + below.rgb * inverse,
    above.a + below.a * inverse);
  output[gid] = uchar4(clamp(result, 0.0, 1.0) * 255.0 + 0.5);
}

kernel void overlay_camera_luma(
    texture2d<float, access::sample> camera [[texture(0)]],
    texture2d<float, access::read_write> luma [[texture(1)]],
    constant CameraUniforms &u [[buffer(0)]],
    uint2 gid [[thread_position_in_grid]]) {
  if (gid.x >= luma.get_width() || gid.y >= luma.get_height()) return;
  float2 point = float2(gid) + 0.5 - float2(u.frame_x, u.frame_y);
  float distance = rounded_box_distance(
    point, float2(u.frame_width, u.frame_height), float(u.radius));
  if (distance > 0.0) {
    float sigma = margin_capped_sigma(
      float2(u.frame_x, u.frame_y), float2(u.frame_width, u.frame_height),
      float2(luma.get_width(), luma.get_height()),
      shadow_sigma(float2(u.frame_width, u.frame_height)));
    float shadow = u.drop_shadow != 0 && sigma > 1.0
      ? soft_shadow(point, float2(u.frame_width, u.frame_height),
                    float(u.radius), sigma, 0.14)
      : 0.0;
    if (shadow > 0.0001) {
      float existing = luma.read(gid).r;
      luma.write(mix(existing, 16.0 / 255.0, shadow), gid);
    }
    return;
  }
  float4 rgba = camera_pixel(camera, point, u);
  if (rgba.a <= 0.0001) return;
  float camera_y = 16.0 / 255.0 +
                   dot(rgba.rgb, float3(0.182586, 0.614231, 0.062007));
  luma.write(mix(luma.read(gid).r, camera_y, rgba.a), gid);
}

kernel void overlay_camera_chroma(
    texture2d<float, access::sample> camera [[texture(0)]],
    texture2d<float, access::read_write> chroma [[texture(1)]],
    constant CameraUniforms &u [[buffer(0)]],
    uint2 gid [[thread_position_in_grid]]) {
  if (gid.x >= chroma.get_width() || gid.y >= chroma.get_height()) return;
  uint2 output_origin = gid * 2;
  float3 rgb_sum = 0.0;
  float alpha_sum = 0.0;
  float shadow_sum = 0.0;
  for (uint y = 0; y < 2; ++y) {
    for (uint x = 0; x < 2; ++x) {
      float2 point = float2(output_origin + uint2(x, y)) + 0.5 -
                     float2(u.frame_x, u.frame_y);
      float distance = rounded_box_distance(
        point, float2(u.frame_width, u.frame_height), float(u.radius));
      float sigma = margin_capped_sigma(
        float2(u.frame_x, u.frame_y), float2(u.frame_width, u.frame_height),
        float2(chroma.get_width() * 2, chroma.get_height() * 2),
        shadow_sigma(float2(u.frame_width, u.frame_height)));
      shadow_sum += u.drop_shadow != 0 && distance > 0.0 && sigma > 1.0
        ? soft_shadow(point, float2(u.frame_width, u.frame_height),
                      float(u.radius), sigma, 0.14)
        : 0.0;
      float4 rgba = camera_pixel(camera, point, u);
      rgb_sum += rgba.rgb * rgba.a;
      alpha_sum += rgba.a;
    }
  }
  float alpha = alpha_sum * 0.25;
  if (alpha <= 0.0001) {
    float shadow = shadow_sum * 0.25;
    if (shadow > 0.0001) {
      float2 existing = chroma.read(gid).rg;
      chroma.write(float4(mix(existing, float2(0.5), shadow), 0.0, 1.0), gid);
    }
    return;
  }
  float3 rgb = rgb_sum / max(alpha_sum, 0.0001);
  float2 camera_uv = float2(
      0.5 + dot(rgb, float3(-0.100644, -0.338572, 0.439216)),
      0.5 + dot(rgb, float3(0.439216, -0.398942, -0.040274)));
  float2 existing = chroma.read(gid).rg;
  chroma.write(float4(mix(existing, camera_uv, alpha), 0.0, 1.0), gid);
}

/// Premultiplied bilinear artwork lookup. Ports `sample_image`
/// (cursor_effects/raster.rs:152-188), which addresses texel `i` at
/// coordinate `i` and interpolates colour weighted by alpha so transparent
/// texels never bleed their colour into the edge.
)METAL"
