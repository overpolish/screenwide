// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#define GPU_COMPOSITOR_MACOS_SHADER_SOURCE_CURSOR_KERNELS @R"METAL(static float4 artwork_texel(texture2d_array<float, access::read> images,
                            uint slice, uint bitmap_width, uint bitmap_height,
                            float x, float y) {
  float last_x = float(max(bitmap_width, 1u) - 1u);
  float last_y = float(max(bitmap_height, 1u) - 1u);
  x = clamp(x, 0.0, last_x);
  y = clamp(y, 0.0, last_y);
  uint x0 = uint(floor(x));
  uint y0 = uint(floor(y));
  uint x1 = min(x0 + 1u, uint(last_x));
  uint y1 = min(y0 + 1u, uint(last_y));
  float fraction_x = x - float(x0);
  float fraction_y = y - float(y0);
  float4 samples[4] = {
      images.read(uint2(x0, y0), slice), images.read(uint2(x1, y0), slice),
      images.read(uint2(x0, y1), slice), images.read(uint2(x1, y1), slice)};
  float weights[4] = {
      (1.0 - fraction_x) * (1.0 - fraction_y), fraction_x * (1.0 - fraction_y),
      (1.0 - fraction_x) * fraction_y, fraction_x * fraction_y};
  float alpha = 0.0;
  float3 colour = 0.0;
  for (uint index = 0; index < 4; ++index) {
    alpha += samples[index].a * weights[index];
    colour += samples[index].rgb * samples[index].a * weights[index];
  }
  if (alpha <= 0.0) return 0.0;
  return float4(colour / alpha, alpha);
}

/// One artwork sample in cursor space. Ports `CursorRaster::sample`
/// (cursor_effects/raster.rs:80-118): the output point is rotated and scaled
/// into the recorded cursor box, then mapped onto the artwork. Vector
/// fallback artwork keeps its design aspect inside that box instead.
static float4 cursor_artwork_sample(
    texture2d_array<float, access::read> images, constant OverlayUniforms &u,
    float2 point, float2 anchor) {
  float2 delta = point - anchor;
  float cosine = cos(u.cursor.rotation_radians);
  float sine = sin(u.cursor.rotation_radians);
  float2 local =
      float2(cosine * delta.x + sine * delta.y,
             -sine * delta.x + cosine * delta.y) / max(u.cursor.scale, 0.0001) +
      float2(u.cursor.hotspot_x, u.cursor.hotspot_y);
  float2 box = float2(u.cursor.width, u.cursor.height);
  if (u.artwork.clip_local_box != 0 &&
      (any(local < 0.0) || any(local >= box)))
    return 0.0;
  float2 bitmap = float2(u.artwork.width, u.artwork.height);
  if (u.artwork.use_design == 0)
    return artwork_texel(images, u.cursor.style, u.artwork.width,
                         u.artwork.height, local.x / max(box.x, 0.0001) * bitmap.x,
                         local.y / max(box.y, 0.0001) * bitmap.y);
  float2 design_size = float2(u.artwork.design_width, u.artwork.design_height);
  float artwork_scale =
      max(min(box.x / design_size.x, box.y / design_size.y), 0.01);
  float2 design = local / artwork_scale +
                  float2(u.artwork.origin_x, u.artwork.origin_y);
  if (any(design < 0.0) || any(design >= design_size)) return 0.0;
  return artwork_texel(images, u.cursor.style, u.artwork.width, u.artwork.height,
                       design.x / design_size.x * bitmap.x,
                       design.y / design_size.y * bitmap.y);
}

/// Ports `CursorRaster::sample_for_draw` (cursor_effects/raster.rs:120-145):
/// system artwork already carries an antialiased alpha edge, so only the
/// hard-edged vector fallback is supersampled over the pixel's 4x4 box.
static float4 cursor_draw_sample(texture2d_array<float, access::read> images,
                                 constant OverlayUniforms &u, float2 point,
                                 float2 anchor) {
  if (u.artwork.supersample == 0)
    return cursor_artwork_sample(images, u, point, anchor);
  const float offsets[4] = {-0.375, -0.125, 0.125, 0.375};
  float alpha = 0.0;
  float3 colour = 0.0;
  for (uint y = 0; y < 4; ++y) {
    for (uint x = 0; x < 4; ++x) {
      float4 sample = cursor_artwork_sample(
          images, u, point + float2(offsets[x], offsets[y]), anchor);
      alpha += sample.a;
      colour += sample.rgb * sample.a;
    }
  }
  if (alpha <= 0.0) return 0.0;
  return float4(colour / alpha, alpha / 16.0);
}

/// The drawn cursor at one output pixel. Ports `CursorCompositor::draw_output`
/// (cursor_effects.rs:602-643) and `draw_blurred`
/// (cursor_effects/raster.rs:239-292): exposure taps are Gaussian weighted
/// along the frame's travel, spaced no more than two output pixels apart.
static float4 cursor_pixel(texture2d_array<float, access::read> images,
                           constant OverlayUniforms &u, float2 point) {
  if (u.cursor.visible == 0 || u.artwork.width == 0 || u.artwork.height == 0)
    return 0.0;
  float2 anchor = float2(u.cursor.x, u.cursor.y);
  float2 delta = float2(u.cursor.blur_delta_x, u.cursor.blur_delta_y);
  float travel = length(delta);
  // MAX_BLUR_DISTANCE (cursor_effects.rs:36). The settings gate lives on the
  // CPU: a disabled motion blur arrives here as a zero delta.
  float distance = min(travel, 80.0);
  if (!(distance > 1.25 && travel > 0.0)) {
    float4 sample = cursor_draw_sample(images, u, point, anchor);
    sample.a *= u.cursor.opacity;
    return sample;
  }
  float2 direction = delta / travel;
  // motion_blur_sample_count (cursor_effects.rs:323-325).
  uint count = uint(clamp(ceil(distance / 2.0) + 1.0, 8.0, 48.0));
  float total_weight = 0.0;
  float alpha = 0.0;
  float3 colour = 0.0;
  for (uint index = 0; index < count; ++index) {
    float progress = float(index) / float(count - 1);
    float centered = (progress - 0.5) / 0.34;
    float weight = exp(-0.5 * centered * centered);
    float4 sample = cursor_draw_sample(
        images, u, point, anchor + direction * ((progress - 0.8) * distance));
    alpha += sample.a * weight;
    colour += sample.rgb * sample.a * weight;
    total_weight += weight;
  }
  alpha /= total_weight;
  if (alpha <= 0.0) return 0.0;
  return float4(colour / (total_weight * alpha), alpha * u.cursor.opacity);
}

)METAL"
