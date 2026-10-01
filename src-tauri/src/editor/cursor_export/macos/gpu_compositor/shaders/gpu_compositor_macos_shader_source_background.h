// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#define GPU_COMPOSITOR_MACOS_SHADER_SOURCE_BACKGROUND @R"METAL(static float hash(float2 position, uint seed) {
  return fract(sin(dot(position, float2(127.1, 311.7)) + float(seed) * 0.017) * 43758.5453) * 2.0 - 1.0;
}

static float noise(float2 position, uint seed) {
  float2 cell = floor(position);
  float2 local = fract(position);
  float2 eased = local * local * (3.0 - 2.0 * local);
  float top = mix(hash(cell, seed), hash(cell + float2(1.0, 0.0), seed), eased.x);
  float bottom = mix(hash(cell + float2(0.0, 1.0), seed), hash(cell + 1.0, seed), eased.x);
  return mix(top, bottom, eased.y);
}

static float fractal_noise(float2 position, uint seed) {
  return noise(position, seed) * 0.58
    + noise(position * 2.07 + float2(11.3, -4.9), seed ^ 0x68bc21eb) * 0.28
    + noise(position * 4.19 + float2(-8.7, 13.1), seed ^ 0x02e5be93) * 0.14;
}

// A stable fraction of one 8-bit step prevents smooth shadows and gradients
// from landing on the same quantisation boundary across large areas. Keeping
// it spatial (rather than changing it every frame) avoids shimmer and needless
// bitrate while making the RGBA preview and encoded canvas use the same image.
static float3 output_dither(float3 colour, float2 point) {
  // A full 8-bit step of spatial noise: every fractional gradient value
  // crosses its quantisation threshold somewhere nearby, which is what keeps
  // smooth gradients from banding after the encoder quantises them. The
  // offset stays positive so no pixel moves more than one step from its
  // undithered value.
  float value = hash(point, 0x9e3779b9) * (1.0 / 255.0);
  return clamp(colour + value, 0.0, 1.0);
}

static bool rounded_pixel_visible(float2 point, float2 size, float radius) {
  if (radius <= 0.0) return true;
  float2 edge = min(point, size - point);
  float2 corner = max(float2(0.0), radius - edge);
  return length(corner) <= radius;
}

static float rounded_box_distance(float2 point, float2 size, float radius) {
  float2 half_size = size * 0.5;
  float2 offset = abs(point - half_size) - (half_size - radius);
  return length(max(offset, 0.0)) + min(max(offset.x, offset.y), 0.0) - radius;
}

static float shadow_sigma(float2 size) {
  return clamp(min(size.x, size.y) * 0.055, 10.0, 110.0);
}

static float margin_capped_sigma(float2 origin, float2 size, float2 canvas,
                                 float base_sigma) {
  // The blur must fit the background actually visible around the object: a
  // near-full-canvas image would otherwise sit inside its own falloff and the
  // margins read as a dark tint instead of a shadow.
  float2 lower = origin;
  float2 upper = canvas - (origin + size);
  float margin = max(0.0, min(min(lower.x, lower.y), min(upper.x, upper.y)));
  return min(base_sigma, margin * 0.45);
}

static float soft_shadow(float2 point, float2 size, float radius,
                         float sigma, float opacity) {
  // CSS box-shadow uses a softer, offset falloff than a centred Gaussian.
  // Moving the sampled box down avoids a dark outline around every edge and
  // gives the recording and camera the same lifted appearance as the preview.
  float2 offset_point = point - float2(0.0, sigma * 0.35);
  float distance = max(0.0, rounded_box_distance(offset_point, size, radius));
  return distance < sigma * 4.0
    ? exp(-(distance * distance) / (2.0 * sigma * sigma)) * opacity
    : 0.0;
}

static float visible_foreground_shadow(
    float2 point, float2 crop_origin, float2 crop_size, float crop_radius,
    float2 image_origin, float2 image_size, float sigma, float opacity) {
  float2 offset_point = point - float2(0.0, sigma * 0.35);
  float crop_distance = rounded_box_distance(
    offset_point - crop_origin, crop_size, crop_radius);
  float image_distance = rounded_box_distance(
    offset_point - image_origin, image_size, 0.0);
  float distance = max(0.0, max(crop_distance, image_distance));
  return distance < sigma * 4.0
    ? exp(-(distance * distance) / (2.0 * sigma * sigma)) * opacity
    : 0.0;
}

static float visible_foreground_sigma(
    float2 crop_origin, float2 crop_size, float2 image_origin,
    float2 image_size, float2 canvas) {
  float2 origin = max(crop_origin, image_origin);
  float2 end = min(crop_origin + crop_size, image_origin + image_size);
  float2 size = max(end - origin, 0.0);
  return min(size.x, size.y) > 0.0
    ? margin_capped_sigma(origin, size, canvas, shadow_sigma(size))
    : 0.0;
}

/// The chosen picture, filled to the canvas: scaled until both sides reach,
/// centred, and trimmed on the axis that overflows. Matches the CPU
/// `cover_fit` framing so a picture frames the same way on every path.
)METAL"
