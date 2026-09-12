// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The eight ported background generators, line for line with the WGSL in
// src/screenshots/mesh_generator_common.wgsl, mesh_generators.wgsl and
// mesh_generators_layered.wgsl. The names match so the three ports stay
// diffable; the maths must not drift.
//
// Every generator takes the canvas seconds as its `time`, the same `motion.x`
// the classic mesh drifts with, scaled once in `gen_pixel` by the generator's
// `speed` from the table in `screenshots/mesh_generator.rs`, so no generator
// carries a pace of its own. Zero paints the still.

struct GenPalette {
  float3 c0;
  float3 c1;
  float3 c2;
  float3 c3;
  uint count;
};

float3 gen_palette_at(GenPalette palette, uint index) {
  uint last = max(palette.count, 1u) - 1u;
  uint wanted = min(index, last);
  if (wanted == 0u) return palette.c0;
  if (wanted == 1u) return palette.c1;
  if (wanted == 2u) return palette.c2;
  return palette.c3;
}

// A palette read as a ramp: the reference generators that take a position
// along their colours all share this walk between neighbours.
float3 gen_ramp(GenPalette palette, float position) {
  uint count = max(palette.count, 1u);
  if (count == 1u) return palette.c0;
  float scaled = saturate(position) * (float)(count - 1u);
  uint lower = (uint)floor(scaled);
  return lerp(gen_palette_at(palette, lower), gen_palette_at(palette, lower + 1u), frac(scaled));
}

float gen_hash11(float value) {
  float carried = frac(value * 0.1031);
  carried = carried * (carried + 33.33);
  carried = carried * (carried + carried);
  return frac(carried);
}

float gen_hash21(float2 position) {
  float3 carried = frac(float3(position.x, position.y, position.x) * 0.1031);
  carried += dot(carried, carried.yzx + 33.33);
  return frac((carried.x + carried.y) * carried.z);
}

float gen_hash31(float3 position) {
  float3 carried = frac(position * 0.1031);
  carried += dot(carried, carried.zyx + 31.32);
  return frac((carried.x + carried.y) * carried.z);
}

float gen_noise2(float2 position) {
  float2 cell = floor(position);
  float2 local = frac(position);
  float2 eased = local * local * (3.0 - 2.0 * local);
  return lerp(
    lerp(gen_hash21(cell), gen_hash21(cell + float2(1.0, 0.0)), eased.x),
    lerp(gen_hash21(cell + float2(0.0, 1.0)), gen_hash21(cell + float2(1.0, 1.0)), eased.x),
    eased.y);
}

float gen_noise3(float3 position) {
  float3 cell = floor(position);
  float3 local = frac(position);
  float3 eased = local * local * (3.0 - 2.0 * local);
  float lower = lerp(
    lerp(gen_hash31(cell), gen_hash31(cell + float3(1.0, 0.0, 0.0)), eased.x),
    lerp(gen_hash31(cell + float3(0.0, 1.0, 0.0)), gen_hash31(cell + float3(1.0, 1.0, 0.0)), eased.x),
    eased.y);
  float upper = lerp(
    lerp(gen_hash31(cell + float3(0.0, 0.0, 1.0)), gen_hash31(cell + float3(1.0, 0.0, 1.0)), eased.x),
    lerp(gen_hash31(cell + float3(0.0, 1.0, 1.0)), gen_hash31(cell + float3(1.0, 1.0, 1.0)), eased.x),
    eased.y);
  return lerp(lower, upper, eased.z);
}

// The reference shaders are written with GLSL's column-major `mat2`, where
// `mat2(a, b, c, d) * v` is `(a * v.x + c * v.y, b * v.x + d * v.y)`. Writing
// the two rotations out avoids carrying that convention into three shading
// languages.
float2 gen_rotate(float2 position, float angle) {
  float sine = sin(angle);
  float cosine = cos(angle);
  return float2(cosine * position.x + sine * position.y, -sine * position.x + cosine * position.y);
}

// The seed as a move of the domain: a turn and a slide, applied to the pattern
// position before a generator reads it. The generators have no seed of their
// own, so this is what a fresh seed changes, and it is what keeps two tiles of
// the same generator distinct at time zero.
float3 gen_seed_shift(uint seed) {
  float value = (float)seed;
  float first = frac(sin(value * 12.9898 + 4.1) * 43758.5453);
  float second = frac(sin(value * 78.2330 + 1.7) * 43758.5453);
  float third = frac(sin(value * 39.4250 + 9.3) * 43758.5453);
  return float3((first - 0.5) * 1.5, (second - 0.5) * 1.5, third * 6.2831853);
}

float2 gen_place(float2 position, float3 shift) {
  float sine = sin(shift.z);
  float cosine = cos(shift.z);
  return float2(position.x * cosine - position.y * sine,
                position.x * sine + position.y * cosine) + shift.xy;
}

// The frame every generator but Strata and Paint Mixer works in: the canvas
// centred and measured in heights, as the reference takes it with its centre
// and scale left at their defaults.
float2 gen_centred(float2 pixel, float2 dimensions, float3 shift) {
  return gen_place((pixel - dimensions * 0.5) / dimensions.y, shift);
}

// Aurora. Detail 8, warp 100%, palette mix 70%, sheen 100%.
float3 gen_aurora(float2 pixel, float2 dimensions, GenPalette palette, float3 shift, float time) {
  float shortest = min(dimensions.x, dimensions.y);
  float2 placed = gen_place((pixel - dimensions * 0.5) * (2.0 / shortest), shift);
  // The reference winds the accumulator forward from `iTime`.
  float phase = time;
  float first = 0.0;
  float second = phase;
  for (int index = 0; index < 8; index += 1) {
    first += cos((float)index + second + first * placed.y);
    second += sin((float)index * placed.x - first);
  }
  float3 base = 0.55 + 0.45 * cos(float3(first, second, first + second) * 0.6 + float3(0.0, 2.0, 4.0));
  float3 modulation = float3(cos(0.7 * second), cos(0.7 * first), sin(0.9 * (first + second)));
  float3 iridescence = cos(base * modulation * 0.5 + 0.5);
  float sheen = saturate(dot(iridescence, float3(0.299, 0.587, 0.114)));
  float hue = 0.5 + 0.5 * sin(1.2 * placed.x + 0.8 * placed.y + 0.25 * first);
  float3 recoloured = sheen * lerp(float3(1.0, 1.0, 1.0), gen_ramp(palette, hue), 0.9);
  return lerp(iridescence, recoloured, 0.7);
}

// Gentle Gradient: a bilinear corner mix under a swirl. Swirl 100%, gloss 100%.
float3 gen_gentle(float2 pixel, float2 dimensions, GenPalette palette, float3 shift, float time) {
  float2 placed = gen_centred(pixel, dimensions, shift);
  float phase = time;
  float angle = (sin(placed.x * 2.0 + phase) + cos(placed.y * 2.3 - phase * 0.8)) * 0.6;
  float2 swirled = gen_rotate(placed, angle);
  float2 ramp = swirled * 0.5 + 0.5;
  float3 color = lerp(palette.c0, gen_palette_at(palette, 1u), saturate(ramp.x));
  color = lerp(color, gen_palette_at(palette, 2u), saturate(ramp.y));
  return saturate(color * (0.85 + color * 0.35));
}

// Ribbons. Bands 6, ripple 100%, direction 225 degrees.
float3 gen_ribbons(float2 pixel, float2 dimensions, GenPalette palette, float3 shift, float time) {
  float2 placed = gen_centred(pixel, dimensions, shift);
  float phase = time;
  float2 direction = float2(cos(3.926990817), sin(3.926990817));
  float flow = dot(placed, direction) * 3.0 + phase;
  float across = sin(placed.x * 12.0 + phase) * cos(placed.y * 8.0) * 0.3;
  float down = cos(placed.y * 10.0 - phase * 0.8) * sin(placed.x * 7.0) * 0.4;
  float ramp = frac(flow + (across + down) * 0.5);
  return gen_ramp(palette, floor(ramp * 6.0) / 5.0);
}

// Currents. Detail 5, warp 100%, turbulence 100%.
float3 gen_currents(float2 pixel, float2 dimensions, GenPalette palette, float3 shift, float time) {
  float2 placed = gen_centred(pixel, dimensions, shift);
  float phase = time;
  float turbulence = gen_noise3(float3(placed * 2.0, phase)) * 2.0 - 1.0;
  float2 warped = placed * 1.2;
  float amplitude = 0.7;
  for (int index = 0; index < 5; index += 1) {
    float frequency = (float)index + 1.0;
    float2 offset = float2(sin(frequency * warped.y + turbulence * frequency + phase),
                           cos(frequency * warped.x + turbulence - phase));
    warped -= amplitude * offset;
    amplitude *= 0.6;
  }
  float primary = 0.5 + 0.5 * sin(0.8 * (warped.x + warped.y) + turbulence);
  float secondary = 0.5 + 0.5 * cos(warped.x - warped.y);
  float3 color = lerp(gen_ramp(palette, primary), gen_ramp(palette, secondary), 0.35);
  return color * (0.75 + 0.25 * sin(warped.y + turbulence));
}

// Paint Mixer: the four colours as the corners of a mirrored bilinear mix.
// Complexity 16, warp 100%, frequency 50%.
float3 gen_paint(float2 pixel, float2 dimensions, GenPalette palette, float3 shift, float time) {
  float2 placed = gen_place((pixel - dimensions * 0.5) / dimensions, shift) + 0.5;
  float drift = time;
  for (int index = 1; index <= 16; index += 1) {
    float frequency = (float)index * 5.0 + 1.0;
    float amplitude = 1.0 / frequency;
    float phase = frequency * 0.3 + drift;
    placed.x += amplitude * sin(frequency * placed.y + phase);
    placed.y += amplitude * sin(frequency * placed.x + phase + 30.0);
  }
  float2 mirrored = abs(2.0 * frac(placed * 0.5) - 1.0);
  float3 top = lerp(palette.c0, gen_palette_at(palette, 1u), mirrored.x);
  float3 bottom = lerp(gen_palette_at(palette, 2u), gen_palette_at(palette, 3u), mirrored.x);
  return lerp(top, bottom, mirrored.y);
}

float gen_fluid_fbm(float3 start, float decay) {
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

// Fluid: domain-warped marble. Detail 50%, marble 100%, vibrance 100%.
float3 gen_fluid(float2 pixel, float2 dimensions, GenPalette palette, float3 shift, float time) {
  float2 placed = gen_centred(pixel, dimensions, shift);
  float detail = 0.5;
  // The reference walks the noise's third axis with `iTime`.
  float phase = time;
  float2 first = float2(
    gen_fluid_fbm(float3(placed, phase), detail),
    gen_fluid_fbm(float3(placed + float2(5.2, 1.3), phase), detail));
  float2 second = float2(
    gen_fluid_fbm(float3(placed + 4.0 * first + float2(1.7, 9.2), phase), detail),
    gen_fluid_fbm(float3(placed + 4.0 * first + float2(8.3, 2.8), phase), detail));
  float field = gen_fluid_fbm(float3(placed + 3.5 * second, phase), detail);
  float3 third = gen_palette_at(palette, 3u);
  float3 color = lerp(palette.c0, gen_palette_at(palette, 1u), saturate(field * field * 2.0));
  color = lerp(color, gen_palette_at(palette, 2u), saturate(length(first) * 0.5));
  color = lerp(color, third, saturate(abs(second.x) * 0.6));
  float highlight = smoothstep(0.5, 1.2, field * field * 3.0 + length(second) * 0.5);
  color += third * 0.35 * highlight;
  return pow(max(color, 0.0), 1.1);
}

float gen_silk_fbm3(float2 start) {
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

float gen_silk_fbm2(float2 start) {
  float value = 0.5 * gen_noise2(start);
  float2 turned = float2(0.8 * start.x + 0.6 * start.y, -0.6 * start.x + 0.8 * start.y) * 2.0;
  return value + 0.25 * gen_noise2(turned);
}

float2 gen_silk_warp(float2 position, float seed, bool lightweight) {
  float2 first = position * 1.2 + float2(1.7 + seed, 9.2);
  float2 second = position * 1.2 + float2(8.3, 2.8 + seed);
  if (lightweight) return float2(gen_silk_fbm2(first), gen_silk_fbm2(second));
  return float2(gen_silk_fbm3(first), gen_silk_fbm3(second));
}

// One sheet's height and its slope, which together light it.
float3 gen_silk_fold(float2 position, float seed, float frequency, bool lightweight, float time) {
  float2 warp = gen_silk_warp(position + seed * 3.7, seed, lightweight);
  float2 warped = position + warp * 0.55;
  // The reference rolls each band of the drape along with `iTime`.
  float phase = time;
  float height = 0.0;
  float2 slope = float2(0.0, 0.0);
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
  slope += cos(diagonal_phase) * float2(diagonal, diagonal) * 0.18;
  float2 detail = frequency * float2(1.8, 1.2);
  float detail_phase = dot(warped, detail) + seed * 0.7 - phase * 0.6;
  height += sin(detail_phase) * 0.08;
  slope += cos(detail_phase) * detail * 0.08;
  if (!lightweight) {
    height += gen_noise2(warped * frequency * 0.9 + seed * 10.0) * 0.12 - 0.06;
  }
  return float3(height, slope);
}

float gen_silk_specular(float2 slope, float3 light, float shine) {
  float squared = dot(slope, slope);
  if (squared < 0.0001) return 0.0;
  float2 across = float2(-slope.y, slope.x) / sqrt(squared);
  float3 tangent = normalize(float3(across, 0.0));
  float3 halfway = normalize(light + float3(0.0, 0.0, 1.0));
  float aligned = dot(tangent, halfway);
  return pow(sqrt(max(1.0 - aligned * aligned, 0.0)), shine);
}

float4 gen_silk_layer(float2 position, float seed, float frequency, float3 tone, float opacity,
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
  float3 fabric = lerp(tone * 0.14, tone * 0.55, smoothstep(0.0, 0.35, shade));
  fabric = lerp(fabric, tone * 0.95, smoothstep(0.25, 0.7, shade) * 0.5);
  float specular = gen_silk_specular(slope, primary, shine) * 0.9;
  specular += gen_silk_specular(slope, secondary, shine * 0.6) * 0.15;
  specular *= sheen;
  fabric += lerp(tone, float3(1.0, 1.0, 1.0), 0.72) * specular * specular * specular * 0.9;
  fabric += float3(0.45, 0.28, 0.15) * smoothstep(0.3, 0.9, depth) * lighting * 0.08;
  return float4(fabric, opacity * (0.65 + depth * 0.35));
}

// Silk. Folds 1, drape 100%, sheen 100%, saturation 135%, vignette 100%.
float3 gen_silk(float2 pixel, float2 dimensions, GenPalette palette, float3 shift, float time) {
  float2 placed = gen_centred(pixel, dimensions, shift);
  // `background` in the WGSL; the preview shader already has a function of
  // that name.
  float3 backdrop = palette.c0;
  float3 first = gen_palette_at(palette, 1u);
  float3 second = gen_palette_at(palette, 2u);
  float3 third = gen_palette_at(palette, 3u);
  float distance = length(placed);
  float3 color = backdrop + backdrop * 0.5 * exp(-distance * distance * 2.0);
  float4 back = gen_silk_layer(placed * 0.8 + float2(0.15, 0.0), 0.0, 2.0, first, 0.30, 26.0, 0.7, time);
  float4 middle = gen_silk_layer(placed + float2(0.0, -0.1), 1.0, 3.2, second, 0.38, 40.0, 0.9, time);
  float4 front = gen_silk_layer(placed * 1.2, 2.0, 4.5, third, 0.50, 55.0, 1.0, time);
  color = lerp(color, back.rgb, back.a);
  color += first * 0.10 * back.a * middle.a;
  color = lerp(color, middle.rgb, middle.a);
  color += second * 0.08 * middle.a * front.a;
  color = lerp(color, front.rgb, front.a);
  color += third * 0.05 * (back.a + middle.a + front.a) / 3.0;
  float vignette = 1.0 - smoothstep(0.25, 1.15, length(placed * float2(0.85, 1.0)));
  color *= 0.6 + 0.4 * vignette;
  float luminance = dot(color, float3(0.299, 0.587, 0.114));
  color = lerp(luminance.xxx, color, 1.35);
  color = max(color, 0.0);
  return color * (2.51 * color + 0.03) / (color * (2.43 * color + 0.59) + 0.14);
}

float gen_strata_fbm(float2 start) {
  float2 position = start;
  float value = 0.0;
  float amplitude = 0.5;
  for (int index = 0; index < 5; index += 1) {
    value += amplitude * gen_noise2(position);
    position = float2(0.877582562 * position.x - 0.479425539 * position.y,
                      0.479425539 * position.x + 0.877582562 * position.y) * 2.0 + float2(100.0, 100.0);
    amplitude *= 0.5;
  }
  return value;
}

// The plates pushing the bands out of true. Tectonics 60%.
float2 gen_strata_warp(float2 position, float time) {
  // The reference drifts the plates with `iTime`.
  float phase = time;
  float across = gen_strata_fbm(position * 1.5 + float2(phase, 0.0)) - 0.5;
  float down = gen_strata_fbm(position * 1.5 + float2(50.0, 30.0 + phase)) - 0.5;
  float compression = sin(position.x * 2.0 + phase) * 0.08;
  float shear = sin(position.y * 3.0 - phase * 0.7) * 0.06;
  return position + float2(across * 0.25 + shear, down * 0.18 + compression) * 0.6;
}

float gen_strata_boundary(float across, float base, float layer) {
  float first = gen_hash11(layer * 7.13);
  float second = gen_hash11(layer * 13.37);
  float third = gen_hash11(layer * 23.71);
  float fold = (0.04 + first * 0.06) * 0.6 * sin(across * (1.5 + first * 2.5) + second * 6.28318530718);
  fold += (0.015 + second * 0.025) * 0.6 * sin(across * (3.0 + second * 3.0) + third * 6.28318530718);
  fold += gen_noise2(float2(across * 4.0 + first * 100.0, layer)) * 0.6 * 0.02;
  return base + fold;
}

float gen_strata_grain(float2 position, float layer) {
  float angle = gen_hash11(layer * 41.93) * 0.3 * 3.141592654;
  float cosine = cos(angle);
  float sine = sin(angle);
  float2 turned = float2(position.x * cosine - position.y * sine,
                         position.x * sine + position.y * cosine);
  float value = gen_noise2(float2(turned.x * 120.0, turned.y * 18.0) + layer * 30.0) * 0.08;
  value += gen_noise2(float2(turned.x * 80.0, turned.y * 12.0) + layer * 50.0 + 100.0) * 0.05;
  value += gen_noise2(float2(turned.x * 15.0, turned.y * 70.0) + layer * 40.0) * 0.03;
  value += gen_noise2(position * 50.0 + layer * 25.0) * 0.025;
  return value - 0.04;
}

// Strata. Layers 12, tectonics 60%, texture 70%, saturation 82%, vignette 100%.
float3 gen_strata(float2 pixel, float2 dimensions, GenPalette palette, float3 shift, float time) {
  float2 frame = gen_place(pixel / dimensions - 0.5, shift) + 0.5;
  float2 warped = gen_strata_warp(float2(frame.x * dimensions.x / dimensions.y, frame.y), time);
  float spacing = 1.0 / 13.0;
  float layer = -1.0;
  float along = 0.0;
  float previous = -0.2;
  [loop] for (int index = 0; index < 12; index += 1) {
    float boundary = gen_strata_boundary(warped.x, (float)(index + 1) * spacing, (float)index);
    if (warped.y >= previous && warped.y < boundary) {
      layer = (float)index;
      along = (warped.y - previous) / max(boundary - previous, 0.001);
      break;
    }
    previous = boundary;
  }
  if (layer < 0.0) {
    layer = 12.0;
    along = 0.5;
  }
  float chosen = gen_hash11(layer * 17.31 + 3.7) * (float)max(palette.count, 1u);
  float3 color = gen_palette_at(palette,
    (uint)clamp(floor(chosen), 0.0, (float)(max(palette.count, 1u) - 1u)));
  color += (gen_hash11(layer * 31.17) - 0.5) * 0.06;
  color += gen_strata_grain(warped, layer) * 0.7;
  color *= 0.85 + 0.15 * smoothstep(0.0, 0.15, along) * (1.0 - smoothstep(0.85, 1.0, along));
  color *= 0.82 + 0.18 * smoothstep(0.0, 0.08, along);
  color += float3(0.04, 0.035, 0.025) * smoothstep(0.92, 1.0, along);
  color *= saturate(1.0 - 0.3 * length((frame - 0.5) * 1.5));
  color += gen_noise2(pixel * 0.15) * 0.03 * 0.7 + (gen_hash21(pixel) - 0.5) * 0.015 * 0.7;
  float luminance = dot(color, float3(0.299, 0.587, 0.114));
  return lerp(luminance.xxx, color, 0.82);
}

// The generator a canvas names, by the id the settings carry. Zero is the
// app's own mesh, which the caller handles rather than this chain. The WGSL
// switches; an if chain keeps every case a plain return under shader model 4.
//
// `time` is the canvas seconds and `speed` the generator's factor from the
// table in `screenshots/mesh_generator.rs`; the two are multiplied here, once,
// and every generator reads the product as its own `time`. Zero paints the
// still.
float3 gen_pixel(uint generator, float2 pixel, float2 dimensions, GenPalette palette, uint seed,
                 float time, float speed) {
  float3 shift = gen_seed_shift(seed);
  float t = time * speed;
  if (generator == 1u) return gen_silk(pixel, dimensions, palette, shift, t);
  if (generator == 2u) return gen_aurora(pixel, dimensions, palette, shift, t);
  if (generator == 3u) return gen_fluid(pixel, dimensions, palette, shift, t);
  if (generator == 4u) return gen_gentle(pixel, dimensions, palette, shift, t);
  if (generator == 5u) return gen_currents(pixel, dimensions, palette, shift, t);
  if (generator == 6u) return gen_paint(pixel, dimensions, palette, shift, t);
  if (generator == 7u) return gen_ribbons(pixel, dimensions, palette, shift, t);
  if (generator == 8u) return gen_strata(pixel, dimensions, palette, shift, t);
  return palette.c0;
}
