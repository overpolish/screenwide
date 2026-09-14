// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once
#import <Foundation/Foundation.h>

/// The Metal half of the ported mesh generators, kept line for line with
/// `screenshots/mesh_generator_common.wgsl` and `mesh_generators.wgsl` so the
/// three ports stay diffable. The generators know nothing about
/// `CanvasUniforms`: a palette, a seed shift and the canvas seconds are all
/// they take, with every knob but the colours at its reference default. The
/// seconds are the same ones the classic mesh drifts with, scaled once in
/// `gen_pixel` by the generator's `speed` from the table in
/// `screenshots/mesh_generator.rs`, so no generator carries a pace of its own.
/// `static` rather than the `shader_source` pattern: the helper that
/// concatenates these is inline in a header two translation units include, so
/// the source has to come with it rather than live in one object file.
static NSString *const generator_shader_source = @R"METAL(
#include <metal_stdlib>
using namespace metal;

struct GenPalette {
  float3 c0;
  float3 c1;
  float3 c2;
  float3 c3;
  uint count;
};

/// WGSL builds this straight from its struct constructor. Metal has one too,
/// but naming it here keeps the canvas's call short enough to fit the frozen
/// size of the shader source it lives in.
static GenPalette gen_palette(float4 c0, float4 c1, float4 c2, float4 c3, uint count) {
  return GenPalette{c0.rgb, c1.rgb, c2.rgb, c3.rgb, count};
}

static float3 gen_palette_at(GenPalette palette, uint index) {
  uint last = max(palette.count, 1u) - 1u;
  uint wanted = min(index, last);
  if (wanted == 0u) { return palette.c0; }
  if (wanted == 1u) { return palette.c1; }
  if (wanted == 2u) { return palette.c2; }
  return palette.c3;
}

/// A palette read as a ramp: the reference generators that take a position
/// along their colours all share this walk between neighbours.
static float3 gen_ramp(GenPalette palette, float position) {
  uint count = max(palette.count, 1u);
  if (count == 1u) { return palette.c0; }
  float scaled = clamp(position, 0.0, 1.0) * float(count - 1u);
  uint lower = uint(floor(scaled));
  return mix(
    gen_palette_at(palette, lower),
    gen_palette_at(palette, lower + 1u),
    fract(scaled));
}

static float gen_hash11(float value) {
  float carried = fract(value * 0.1031);
  carried = carried * (carried + 33.33);
  carried = carried * (carried + carried);
  return fract(carried);
}

static float gen_hash21(float2 position) {
  float3 carried = fract(float3(position.x, position.y, position.x) * 0.1031);
  carried += dot(carried, carried.yzx + 33.33);
  return fract((carried.x + carried.y) * carried.z);
}

static float gen_hash31(float3 position) {
  float3 carried = fract(position * 0.1031);
  carried += dot(carried, carried.zyx + 31.32);
  return fract((carried.x + carried.y) * carried.z);
}

static float gen_noise2(float2 position) {
  float2 cell = floor(position);
  float2 local = fract(position);
  float2 eased = local * local * (3.0 - 2.0 * local);
  return mix(
    mix(gen_hash21(cell), gen_hash21(cell + float2(1.0, 0.0)), eased.x),
    mix(gen_hash21(cell + float2(0.0, 1.0)), gen_hash21(cell + float2(1.0, 1.0)), eased.x),
    eased.y);
}

static float gen_noise3(float3 position) {
  float3 cell = floor(position);
  float3 local = fract(position);
  float3 eased = local * local * (3.0 - 2.0 * local);
  float lower = mix(
    mix(gen_hash31(cell), gen_hash31(cell + float3(1.0, 0.0, 0.0)), eased.x),
    mix(gen_hash31(cell + float3(0.0, 1.0, 0.0)), gen_hash31(cell + float3(1.0, 1.0, 0.0)), eased.x),
    eased.y);
  float upper = mix(
    mix(gen_hash31(cell + float3(0.0, 0.0, 1.0)), gen_hash31(cell + float3(1.0, 0.0, 1.0)), eased.x),
    mix(gen_hash31(cell + float3(0.0, 1.0, 1.0)), gen_hash31(cell + float3(1.0, 1.0, 1.0)), eased.x),
    eased.y);
  return mix(lower, upper, eased.z);
}

/// The reference shaders are written with GLSL's column-major `mat2`, where
/// `mat2(a, b, c, d) * v` is `(a * v.x + c * v.y, b * v.x + d * v.y)`. Writing
/// the two rotations out avoids carrying that convention into three shading
/// languages.
static float2 gen_rotate(float2 position, float angle) {
  float sine = sin(angle);
  float cosine = cos(angle);
  return float2(cosine * position.x + sine * position.y, -sine * position.x + cosine * position.y);
}

/// The seed as a move of the domain: a turn and a slide, applied to the
/// pattern position before a generator reads it. It is what keeps two tiles of
/// the same generator distinct at time zero.
static float3 gen_seed_shift(uint seed) {
  float value = float(seed);
  float first = fract(sin(value * 12.9898 + 4.1) * 43758.5453);
  float second = fract(sin(value * 78.2330 + 1.7) * 43758.5453);
  float third = fract(sin(value * 39.4250 + 9.3) * 43758.5453);
  return float3((first - 0.5) * 1.5, (second - 0.5) * 1.5, third * 6.2831853);
}

static float2 gen_place(float2 position, float3 shift) {
  float sine = sin(shift.z);
  float cosine = cos(shift.z);
  return float2(
    position.x * cosine - position.y * sine,
    position.x * sine + position.y * cosine) + shift.xy;
}

/// The frame every generator but Strata and Paint Mixer works in: the canvas
/// centred and measured in heights.
static float2 gen_centred(float2 pixel, float2 dimensions, float3 shift) {
  return gen_place((pixel - dimensions * 0.5) / dimensions.y, shift);
}

/// Aurora. Detail 8, warp 100%, palette mix 70%, sheen 100%.
static float3 gen_aurora(float2 pixel, float2 dimensions, GenPalette palette, float3 shift, float time) {
  float shortest = min(dimensions.x, dimensions.y);
  float2 point = gen_place((pixel - dimensions * 0.5) * (2.0 / shortest), shift);
  // The reference winds the accumulator forward from `iTime`.
  float phase = time;
  float first = 0.0;
  float second = phase;
  for (int index = 0; index < 8; index += 1) {
    first += cos(float(index) + second + first * point.y);
    second += sin(float(index) * point.x - first);
  }
  float3 base = 0.55 + 0.45 * cos(float3(first, second, first + second) * 0.6 + float3(0.0, 2.0, 4.0));
  float3 modulation = float3(cos(0.7 * second), cos(0.7 * first), sin(0.9 * (first + second)));
  float3 iridescence = cos(base * modulation * 0.5 + 0.5);
  float sheen = clamp(dot(iridescence, float3(0.299, 0.587, 0.114)), 0.0, 1.0);
  float hue = 0.5 + 0.5 * sin(1.2 * point.x + 0.8 * point.y + 0.25 * first);
  float3 recoloured = sheen * mix(float3(1.0), gen_ramp(palette, hue), 0.9);
  return mix(iridescence, recoloured, 0.7);
}

/// Gentle Gradient: a bilinear corner mix under a swirl. Swirl 100%, gloss 100%.
static float3 gen_gentle(float2 pixel, float2 dimensions, GenPalette palette, float3 shift, float time) {
  float2 point = gen_centred(pixel, dimensions, shift);
  float phase = time;
  float angle = (sin(point.x * 2.0 + phase) + cos(point.y * 2.3 - phase * 0.8)) * 0.6;
  float2 swirled = gen_rotate(point, angle);
  float2 ramp = swirled * 0.5 + 0.5;
  float3 color = mix(palette.c0, gen_palette_at(palette, 1u), clamp(ramp.x, 0.0, 1.0));
  color = mix(color, gen_palette_at(palette, 2u), clamp(ramp.y, 0.0, 1.0));
  return clamp(color * (float3(0.85) + color * 0.35), float3(0.0), float3(1.0));
}

/// Ribbons. Bands 6, ripple 100%, direction 225 degrees.
static float gen_ribbon_phase(float2 point, float time) {
  float2 direction = float2(cos(3.926990817), sin(3.926990817));
  float flow = dot(point, direction) * 3.0 + time;
  float across = sin(point.x * 12.0 + time) * cos(point.y * 8.0) * 0.3;
  float down = cos(point.y * 10.0 - time * 0.8) * sin(point.x * 7.0) * 0.4;
  return flow + (across + down) * 0.5;
}

static float3 gen_ribbons(float2 pixel, float2 dimensions, GenPalette palette, float3 shift, float time) {
  float2 point = gen_centred(pixel, dimensions, shift);
  float ramp = fract(gen_ribbon_phase(point, time));
  float2 point_xp = gen_centred(pixel + float2(0.5, 0.0), dimensions, shift);
  float2 point_xm = gen_centred(pixel - float2(0.5, 0.0), dimensions, shift);
  float2 point_yp = gen_centred(pixel + float2(0.0, 0.5), dimensions, shift);
  float2 point_ym = gen_centred(pixel - float2(0.0, 0.5), dimensions, shift);
  float footprint = 0.5 * (abs(gen_ribbon_phase(point_xp, time) - gen_ribbon_phase(point_xm, time)) +
                           abs(gen_ribbon_phase(point_yp, time) - gen_ribbon_phase(point_ym, time)));
  float band = floor(ramp * 6.0);
  float fraction = ramp * 6.0 - band;
  float width = clamp(max(footprint * 6.0, 0.0001), 0.0, 0.5);
  float3 color = gen_ramp(palette, band / 5.0);
  float previous = band > 0.0 ? band - 1.0 : 5.0;
  float next = band < 5.0 ? band + 1.0 : 0.0;
  color = mix(gen_ramp(palette, previous / 5.0), color, smoothstep(-width, width, fraction));
  return mix(color, gen_ramp(palette, next / 5.0), smoothstep(-width, width, fraction - 1.0));
}

/// Currents. Detail 5, warp 100%, turbulence 100%.
static float3 gen_currents(float2 pixel, float2 dimensions, GenPalette palette, float3 shift, float time) {
  float2 point = gen_centred(pixel, dimensions, shift);
  float phase = time;
  float turbulence = gen_noise3(float3(point * 2.0, phase)) * 2.0 - 1.0;
  float2 warped = point * 1.2;
  float amplitude = 0.7;
  for (int index = 0; index < 5; index += 1) {
    float frequency = float(index) + 1.0;
    float2 offset = float2(
      sin(frequency * warped.y + turbulence * frequency + phase),
      cos(frequency * warped.x + turbulence - phase));
    warped -= amplitude * offset;
    amplitude *= 0.6;
  }
  float primary = 0.5 + 0.5 * sin(0.8 * (warped.x + warped.y) + turbulence);
  float secondary = 0.5 + 0.5 * cos(warped.x - warped.y);
  float3 color = mix(gen_ramp(palette, primary), gen_ramp(palette, secondary), 0.35);
  return color * (0.75 + 0.25 * sin(warped.y + turbulence));
}

/// Paint Mixer: the four colours as the corners of a mirrored bilinear mix.
/// Complexity 16, warp 100%, frequency 50%.
static float3 gen_paint(float2 pixel, float2 dimensions, GenPalette palette, float3 shift, float time) {
  float2 point = gen_place((pixel - dimensions * 0.5) / dimensions, shift) + 0.5;
  float drift = time;
  for (int index = 1; index <= 16; index += 1) {
    float frequency = float(index) * 5.0 + 1.0;
    float amplitude = 1.0 / frequency;
    float phase = frequency * 0.3 + drift;
    point.x += amplitude * sin(frequency * point.y + phase);
    point.y += amplitude * sin(frequency * point.x + phase + 30.0);
  }
  float2 mirrored = abs(2.0 * fract(point * 0.5) - 1.0);
  float3 top = mix(palette.c0, gen_palette_at(palette, 1u), mirrored.x);
  float3 bottom = mix(gen_palette_at(palette, 2u), gen_palette_at(palette, 3u), mirrored.x);
  return mix(top, bottom, mirrored.y);
}

static float gen_fluid_fbm(float3 start, float decay) {
  float3 position = start;
  float value = 0.0;
  float amplitude = 0.5;
  for (int index = 0; index < 5; index += 1) {
    value += amplitude * (gen_noise3(position) * 2.0 - 1.0);
    float2 rotated = gen_rotate(position.xy, 0.7) * 2.03 + float2(1.7, 9.2);
    position = float3(rotated, position.z + 2.3);
    amplitude *= decay;
  }
  return value;
}

/// Fluid: domain-warped marble. Detail 50%, marble 100%, vibrance 100%.
static float3 gen_fluid(float2 pixel, float2 dimensions, GenPalette palette, float3 shift, float time) {
  float2 point = gen_centred(pixel, dimensions, shift);
  float detail = 0.5;
  // The reference walks the noise's third axis with `iTime`.
  float phase = time;
  float2 first = float2(
    gen_fluid_fbm(float3(point, phase), detail),
    gen_fluid_fbm(float3(point + float2(5.2, 1.3), phase), detail));
  float2 second = float2(
    gen_fluid_fbm(float3(point + 4.0 * first + float2(1.7, 9.2), phase), detail),
    gen_fluid_fbm(float3(point + 4.0 * first + float2(8.3, 2.8), phase), detail));
  float field = gen_fluid_fbm(float3(point + 3.5 * second, phase), detail);
  float3 third = gen_palette_at(palette, 3u);
  float3 color = mix(palette.c0, gen_palette_at(palette, 1u), clamp(field * field * 2.0, 0.0, 1.0));
  color = mix(color, gen_palette_at(palette, 2u), clamp(length(first) * 0.5, 0.0, 1.0));
  color = mix(color, third, clamp(abs(second.x) * 0.6, 0.0, 1.0));
  float highlight = smoothstep(0.5, 1.2, field * field * 3.0 + length(second) * 0.5);
  color += third * 0.35 * highlight;
  return pow(max(color, float3(0.0)), float3(1.1));
}
)METAL";
