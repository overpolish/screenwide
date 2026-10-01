// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#define GPU_COMPOSITOR_MACOS_SHADER_SOURCE_COMPOSITION @R"METAL(static float3 background_image_pixel(
    texture2d<float, access::sample> picture, float2 point,
    float2 dimensions) {
  constexpr sampler picture_sampler(coord::normalized, address::clamp_to_edge,
                                    filter::linear);
  float2 size = float2(picture.get_width(), picture.get_height());
  if (any(size <= 0.0) || any(dimensions <= 0.0)) return float3(0.0);
  float scale = max(dimensions.x / size.x, dimensions.y / size.y);
  float2 covered = size * scale;
  return picture.sample(
    picture_sampler, (point - (dimensions - covered) * 0.5) / covered).rgb;
}

static float3 mesh_pixel(float2 point, float2 dimensions,
                         constant CanvasUniforms &u, float seconds) {
  float shortest = min(dimensions.x, dimensions.y);
  float frequency = 3.5 / shortest;
  float phase = seconds * 0.28;
  float2 drift = float2(sin(phase), cos(phase * 0.83)) * shortest * 0.012;
  float2 warped_point = point + drift;
  float warp_scale = shortest * u.mesh_warp_percent / 100.0;
  float2 warp = float2(
    fractal_noise(warped_point * frequency + phase * 0.035, u.mesh_seed),
    fractal_noise(warped_point * frequency + float2(19.7, -7.3) - phase * 0.03,
                  u.mesh_seed ^ 0xa511e9b3)
  ) * warp_scale;
  float2 aspect = dimensions / shortest;
  float3 weighted = u.mesh_colors[u.mesh_point_count].rgb * 0.18;
  float total = 0.18;
  for (uint index = 0; index < u.mesh_point_count; ++index) {
    float4 first = u.mesh_points[index * 2];
    float4 second = u.mesh_points[index * 2 + 1];
    float local_phase = phase + float(index) * 1.73;
    float2 animated_center = first.xy + float2(sin(local_phase), cos(local_phase * 0.91)) * 0.012;
    float2 delta = (point + warp) / shortest - animated_center * aspect;
    float2 rotated = float2(delta.x * second.x + delta.y * second.y,
                            -delta.x * second.y + delta.y * second.x);
    float distance = length(rotated / max(first.zw, float2(0.01)));
    float weight = 1.0 / (pow(max(distance, 0.025), 3.5) + 0.012);
    weighted += u.mesh_colors[index].rgb * weight;
    total += weight;
  }
  float depth = fractal_noise((point + drift) * frequency * 0.7,
                              u.mesh_seed ^ 0xd1b54a35) * 13.0 / 255.0;
  return clamp(weighted / total + depth, 0.0, 1.0);
}

static float3 canvas_background(
    texture2d<float, access::sample> picture, float2 point, float2 dimensions,
    constant CanvasUniforms &u, float seconds) {
  if (u.has_background_image != 0)
    return background_image_pixel(picture, point, dimensions);
  if (u.mesh_enabled == 0) return float3(u.background_color.rgb);
  // A generator reads its colours, its seed and the same drifting seconds,
  // which `gen_pixel` scales by the generator's own speed.
  if (u.mesh_generator == 0) return mesh_pixel(point, dimensions, u, seconds);
  return gen_pixel(u.mesh_generator, point, dimensions,
                   gen_palette(u.mesh_colors[0], u.mesh_colors[1],
                               u.mesh_colors[2], u.mesh_colors[3],
                               u.mesh_generator_color_count), u.mesh_seed,
                   seconds, u.mesh_generator_speed);
}

static float3 yuv_to_rgb(float y, float2 uv) {
  float adjusted_y = max(0.0, (y - 16.0 / 255.0) * (255.0 / 219.0));
  float2 adjusted_uv = uv - 0.5;
  return clamp(float3(
    adjusted_y + 1.5748 * adjusted_uv.y,
    adjusted_y - 0.1873 * adjusted_uv.x - 0.4681 * adjusted_uv.y,
    adjusted_y + 1.8556 * adjusted_uv.x), 0.0, 1.0);
}

static float3 source_pixel(texture2d<float, access::sample> source_y,
                           texture2d<float, access::sample> source_uv,
                           float2 output_point, constant CanvasUniforms &u) {
  constexpr sampler linear_sampler(coord::normalized, address::clamp_to_edge,
                                   filter::linear);
  float2 source = (output_point - float2(u.image_x, u.image_y)) /
                  float2(u.image_width, u.image_height);
  return yuv_to_rgb(source_y.sample(linear_sampler, source).r,
                    source_uv.sample(linear_sampler, source).rg);
}

static float4 rgba_source_pixel(const device uchar4 *source,
                                uint source_width, uint source_height,
                                float2 output_point,
                                constant CanvasUniforms &u) {
  float2 coordinate = (output_point - float2(u.image_x, u.image_y)) /
                      float2(u.image_width, u.image_height);
  float2 pixel = clamp(coordinate * float2(source_width, source_height) - 0.5,
                       0.0,
                       float2(source_width - 1, source_height - 1));
  uint2 first = uint2(floor(pixel));
  uint2 second = min(first + 1, uint2(source_width - 1, source_height - 1));
  float2 amount = fract(pixel);
  float4 top = mix(float4(source[first.y * source_width + first.x]) / 255.0,
                   float4(source[first.y * source_width + second.x]) / 255.0,
                   amount.x);
  float4 bottom = mix(float4(source[second.y * source_width + first.x]) / 255.0,
                      float4(source[second.y * source_width + second.x]) / 255.0,
                      amount.x);
  return mix(top, bottom, amount.y);
}

static float rounded_coverage(float2 point, float2 size, float radius) {
  // Half-pixel smoothing over the signed distance antialiases rounded corners
  // instead of the old binary inside test. A zero radius stays a hard edge:
  // axis-aligned boundaries are already pixel-exact and smoothing them would
  // darken the outermost row.
  float distance = rounded_box_distance(point, size, radius);
  if (radius <= 0.0) return distance < 0.0 ? 1.0 : 0.0;
)METAL"
