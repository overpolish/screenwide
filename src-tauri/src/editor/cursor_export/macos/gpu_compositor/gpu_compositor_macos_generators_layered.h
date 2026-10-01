// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once
#import "gpu_compositor_macos_generators.h"

/// The two layered generators and the switch the canvas calls, kept line for
/// line with `screenshots/mesh_generators_layered.wgsl`. Silk stacks three lit
/// sheets of fabric; Strata cuts the canvas into folded bands. Both drift with
/// the canvas seconds, which `gen_pixel` scales once by the generator's
/// `speed` from the table in `screenshots/mesh_generator.rs`.
static NSString *const generator_layered_shader_source = @R"METAL(
static float gen_silk_fbm3(float2 start) {
  float2 position = start;
  float value = 0.0;
  float amplitude = 0.5;
  for (int index = 0; index < 3; index += 1) {
    value += amplitude * gen_noise2(position);
    position = float2(0.8 * position.x + 0.6 * position.y, -0.6 * position.x + 0.8 * position.y) * 2.0;
    amplitude *= 0.5;
  }
  return value;
}

static float gen_silk_fbm2(float2 start) {
  float value = 0.5 * gen_noise2(start);
  float2 turned = float2(0.8 * start.x + 0.6 * start.y, -0.6 * start.x + 0.8 * start.y) * 2.0;
  return value + 0.25 * gen_noise2(turned);
}

static float2 gen_silk_warp(float2 position, float seed, bool lightweight) {
  float2 first = position * 1.2 + float2(1.7 + seed, 9.2);
  float2 second = position * 1.2 + float2(8.3, 2.8 + seed);
  if (lightweight) {
    return float2(gen_silk_fbm2(first), gen_silk_fbm2(second));
  }
  return float2(gen_silk_fbm3(first), gen_silk_fbm3(second));
}

/// One sheet's height and its slope, which together light it.
static float3 gen_silk_fold(float2 position, float seed, float frequency, bool lightweight, float time) {
  float2 warp = gen_silk_warp(position + seed * 3.7, seed, lightweight);
  float2 warped = position + warp * 0.55;
  // The reference rolls each band of the drape along with `iTime`.
  float phase = time;
  float height = 0.0;
  float2 slope = float2(0.0);
  float2 first = frequency * float2(0.7, 0.4);
  float first_phase = dot(warped, first) + seed * 2.1 + phase;
  height += sin(first_phase) * 0.35;
  slope += cos(first_phase) * first * 0.35;
  float2 second = frequency * float2(-0.3, 0.9);
  float second_phase = dot(warped, second) + seed * 1.3 - phase * 0.8;
  height += sin(second_phase) * 0.25;
  slope += cos(second_phase) * second * 0.25;
  float diagonal = frequency * 0.6;
  float diagonal_phase = (warped.x + warped.y) * diagonal + seed * 4.5 + phase * 1.3;
  height += sin(diagonal_phase) * 0.18;
  slope += cos(diagonal_phase) * float2(diagonal) * 0.18;
  float2 detail = frequency * float2(1.8, 1.2);
  float detail_phase = dot(warped, detail) + seed * 0.7 - phase * 0.6;
  height += sin(detail_phase) * 0.08;
  slope += cos(detail_phase) * detail * 0.08;
  if (!lightweight) {
    height += gen_noise2(warped * frequency * 0.9 + seed * 10.0) * 0.12 - 0.06;
  }
  return float3(height, slope);
}

static float gen_silk_specular(float2 slope, float3 light, float shine) {
  float squared = dot(slope, slope);
  if (squared < 0.0001) {
    return 0.0;
  }
  float2 across = float2(-slope.y, slope.x) / sqrt(squared);
  float3 tangent = normalize(float3(across, 0.0));
  float3 halfway = normalize(light + float3(0.0, 0.0, 1.0));
  float aligned = dot(tangent, halfway);
  return pow(sqrt(max(1.0 - aligned * aligned, 0.0)), shine);
}

static float4 gen_silk_layer(
    float2 position, float seed, float frequency, float3 tone, float opacity,
    float shine, float sheen, float time) {
  float3 primary = normalize(float3(0.4, 1.05, 0.8));
  float3 secondary = normalize(float3(-0.5, -0.3, 0.6));
  bool lightweight = opacity < 0.35;
  float3 fold = gen_silk_fold(position, seed, frequency, lightweight, time);
  float2 slope = fold.yz;
  float3 normal = normalize(float3(-slope * 1.8, 1.0));
  float lighting = max(dot(normal, primary), 0.0) * 0.75 + max(dot(normal, secondary), 0.0) * 0.12;
  float depth = smoothstep(-0.8, 0.4, fold.x);
  float shade = lighting * depth;
  float3 fabric = mix(tone * 0.14, tone * 0.55, smoothstep(0.0, 0.35, shade));
  fabric = mix(fabric, tone * 0.95, smoothstep(0.25, 0.7, shade) * 0.5);
  float specular = gen_silk_specular(slope, primary, shine) * 0.9;
  specular += gen_silk_specular(slope, secondary, shine * 0.6) * 0.15;
  specular *= sheen;
  fabric += mix(tone, float3(1.0), 0.72) * specular * specular * specular * 0.9;
  fabric += float3(0.45, 0.28, 0.15) * smoothstep(0.3, 0.9, depth) * lighting * 0.08;
  return float4(fabric, opacity * (0.65 + depth * 0.35));
}

/// Silk. Folds 1, drape 100%, sheen 100%, saturation 135%, vignette 100%.
static float3 gen_silk(float2 pixel, float2 dimensions, GenPalette palette, float3 shift, float time) {
  float2 point = gen_centred(pixel, dimensions, shift);
  float3 background = palette.c0;
  float3 first = gen_palette_at(palette, 1u);
  float3 second = gen_palette_at(palette, 2u);
  float3 third = gen_palette_at(palette, 3u);
  float distance = length(point);
  float3 color = background + background * 0.5 * exp(-distance * distance * 2.0);
  float4 back = gen_silk_layer(point * 0.8 + float2(0.15, 0.0), 0.0, 2.0, first, 0.30, 26.0, 0.7, time);
  float4 middle = gen_silk_layer(point + float2(0.0, -0.1), 1.0, 3.2, second, 0.38, 40.0, 0.9, time);
  float4 front = gen_silk_layer(point * 1.2, 2.0, 4.5, third, 0.50, 55.0, 1.0, time);
  color = mix(color, back.rgb, back.a);
  color += first * 0.10 * back.a * middle.a;
  color = mix(color, middle.rgb, middle.a);
  color += second * 0.08 * middle.a * front.a;
  color = mix(color, front.rgb, front.a);
  color += third * 0.05 * (back.a + middle.a + front.a) / 3.0;
  float vignette = 1.0 - smoothstep(0.25, 1.15, length(point * float2(0.85, 1.0)));
  color *= 0.6 + 0.4 * vignette;
  color = mix(float3(dot(color, float3(0.299, 0.587, 0.114))), color, 1.35);
  color = max(color, float3(0.0));
  return color * (2.51 * color + 0.03) / (color * (2.43 * color + 0.59) + 0.14);
}

static float gen_strata_fbm(float2 start) {
  float2 position = start;
  float value = 0.0;
  float amplitude = 0.5;
  for (int index = 0; index < 5; index += 1) {
    value += amplitude * gen_noise2(position);
    position = float2(
      0.877582562 * position.x - 0.479425539 * position.y,
      0.479425539 * position.x + 0.877582562 * position.y) * 2.0 + float2(100.0);
    amplitude *= 0.5;
  }
  return value;
}

/// The plates pushing the bands out of true. Tectonics 60%.
static float2 gen_strata_warp(float2 position, float time) {
  // The reference drifts the plates with `iTime`.
  float phase = time;
  float across = gen_strata_fbm(position * 1.5 + float2(phase, 0.0)) - 0.5;
  float down = gen_strata_fbm(position * 1.5 + float2(50.0, 30.0 + phase)) - 0.5;
  float compression = sin(position.x * 2.0 + phase) * 0.08;
  float shear = sin(position.y * 3.0 - phase * 0.7) * 0.06;
  return position + float2(across * 0.25 + shear, down * 0.18 + compression) * 0.6;
}

static float gen_strata_boundary(float across, float base, float layer) {
  float first = gen_hash11(layer * 7.13);
  float second = gen_hash11(layer * 13.37);
  float third = gen_hash11(layer * 23.71);
  float fold = (0.04 + first * 0.06) * 0.6 * sin(across * (1.5 + first * 2.5) + second * 6.28318530718);
  fold += (0.015 + second * 0.025) * 0.6 * sin(across * (3.0 + second * 3.0) + third * 6.28318530718);
  fold += gen_noise2(float2(across * 4.0 + first * 100.0, layer)) * 0.6 * 0.02;
  return base + fold;
}

static float gen_strata_grain(float2 position, float layer) {
  float angle = gen_hash11(layer * 41.93) * 0.3 * 3.141592654;
  float cosine = cos(angle);
  float sine = sin(angle);
  float2 turned = float2(
    position.x * cosine - position.y * sine,
    position.x * sine + position.y * cosine);
  float value = gen_noise2(float2(turned.x * 120.0, turned.y * 18.0) + layer * 30.0) * 0.08;
  value += gen_noise2(float2(turned.x * 80.0, turned.y * 12.0) + layer * 50.0 + 100.0) * 0.05;
  value += gen_noise2(float2(turned.x * 15.0, turned.y * 70.0) + layer * 40.0) * 0.03;
  value += gen_noise2(position * 50.0 + layer * 25.0) * 0.025;
  return value - 0.04;
}

/// Strata. Layers 12, tectonics 60%, texture 70%, saturation 82%, vignette 100%.
static float3 gen_strata_sample(float2 pixel, float2 dimensions, GenPalette palette, float3 shift, float time) {
  float2 frame = gen_place(pixel / dimensions - 0.5, shift) + 0.5;
  float2 warped = gen_strata_warp(float2(frame.x * dimensions.x / dimensions.y, frame.y), time);
  float spacing = 1.0 / 13.0;
  float layer = -1.0;
  float along = 0.0;
  float previous = -0.2;
  for (int index = 0; index < 12; index += 1) {
    float boundary = gen_strata_boundary(warped.x, float(index + 1) * spacing, float(index));
    if (warped.y >= previous && warped.y < boundary) {
      layer = float(index);
      along = (warped.y - previous) / max(boundary - previous, 0.001);
      break;
    }
    previous = boundary;
  }
  if (layer < 0.0) {
    layer = 12.0;
    along = 0.5;
  }
  float chosen = gen_hash11(layer * 17.31 + 3.7) * float(max(palette.count, 1u));
  float3 color = gen_palette_at(palette, uint(clamp(floor(chosen), 0.0, float(max(palette.count, 1u) - 1u))));
  color += (gen_hash11(layer * 31.17) - 0.5) * 0.06;
  color += gen_strata_grain(warped, layer) * 0.7;
  color *= 0.85 + 0.15 * smoothstep(0.0, 0.15, along) * (1.0 - smoothstep(0.85, 1.0, along));
  color *= 0.82 + 0.18 * smoothstep(0.0, 0.08, along);
  color += float3(0.04, 0.035, 0.025) * smoothstep(0.92, 1.0, along);
  color *= clamp(1.0 - 0.3 * length((frame - 0.5) * 1.5), 0.0, 1.0);
  color += gen_noise2(pixel * 0.15) * 0.03 * 0.7 + (gen_hash21(pixel) - 0.5) * 0.015 * 0.7;
  return mix(float3(dot(color, float3(0.299, 0.587, 0.114))), color, 0.82);
}

static float3 gen_strata(float2 pixel, float2 dimensions, GenPalette palette, float3 shift, float time) {
  float2 quarter = float2(0.25, 0.25);
  return (gen_strata_sample(pixel - quarter, dimensions, palette, shift, time) +
          gen_strata_sample(pixel + float2(quarter.x, -quarter.y), dimensions, palette, shift, time) +
          gen_strata_sample(pixel + float2(-quarter.x, quarter.y), dimensions, palette, shift, time) +
          gen_strata_sample(pixel + quarter, dimensions, palette, shift, time)) * 0.25;
}

/// The generator a canvas names, by the id the settings carry. Zero is the
/// app's own mesh, which the caller handles rather than this switch. `time` is
/// the canvas seconds and `speed` the generator's factor from the table in
/// `screenshots/mesh_generator.rs`; the two are multiplied here, once, and
/// every generator reads the product as its own `time`. Zero paints the still.
static float3 gen_pixel(
    uint generator, float2 pixel, float2 dimensions, GenPalette palette,
    uint seed, float time, float speed) {
  float3 shift = gen_seed_shift(seed);
  float t = time * speed;
  switch (generator) {
    case 1u: { return gen_silk(pixel, dimensions, palette, shift, t); }
    case 2u: { return gen_aurora(pixel, dimensions, palette, shift, t); }
    case 3u: { return gen_fluid(pixel, dimensions, palette, shift, t); }
    case 4u: { return gen_gentle(pixel, dimensions, palette, shift, t); }
    case 5u: { return gen_currents(pixel, dimensions, palette, shift, t); }
    case 6u: { return gen_paint(pixel, dimensions, palette, shift, t); }
    case 7u: { return gen_ribbons(pixel, dimensions, palette, shift, t); }
    case 8u: { return gen_strata(pixel, dimensions, palette, shift, t); }
    default: { return palette.c0; }
  }
}
)METAL";

extern __attribute__((visibility("hidden"))) NSString *const shader_source;

/// The canvas library's source: the generators have to be declared before
/// `canvas_background` calls them, so every site that builds it concatenates
/// them ahead of the compositor's own shaders.
static inline NSString *screenwide_gpu_canvas_shader_source(void) {
  return [[generator_shader_source
      stringByAppendingString:generator_layered_shader_source]
      stringByAppendingString:shader_source];
}
