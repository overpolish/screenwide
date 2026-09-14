// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The two layered generators, and the switch every backend calls.
//
// Silk stacks three lit sheets of fabric; Strata cuts the canvas into folded
// bands. Both are the reference `mainImage` with every knob but the colours at
// its reference default. Silk's first colour is the reference's separate
// background swatch, which is why it carries four.
//
// `time` is the canvas seconds already scaled by the generator's `speed`, the
// factor tabled in `mesh_generator.rs`; `gen_pixel` applies it once, so no
// generator carries a pace of its own.

fn gen_silk_fbm3(start: vec2<f32>) -> f32 {
  var position = start;
  var value = 0.0;
  var amplitude = 0.5;
  for (var index = 0; index < 3; index += 1) {
    value += amplitude * gen_noise2(position);
    position = vec2<f32>(0.8 * position.x + 0.6 * position.y, -0.6 * position.x + 0.8 * position.y) * 2.0;
    amplitude *= 0.5;
  }
  return value;
}

fn gen_silk_fbm2(start: vec2<f32>) -> f32 {
  let value = 0.5 * gen_noise2(start);
  let turned = vec2<f32>(0.8 * start.x + 0.6 * start.y, -0.6 * start.x + 0.8 * start.y) * 2.0;
  return value + 0.25 * gen_noise2(turned);
}

fn gen_silk_warp(position: vec2<f32>, seed: f32, lightweight: bool) -> vec2<f32> {
  let first = position * 1.2 + vec2<f32>(1.7 + seed, 9.2);
  let second = position * 1.2 + vec2<f32>(8.3, 2.8 + seed);
  if (lightweight) {
    return vec2<f32>(gen_silk_fbm2(first), gen_silk_fbm2(second));
  }
  return vec2<f32>(gen_silk_fbm3(first), gen_silk_fbm3(second));
}

/// One sheet's height and its slope, which together light it.
fn gen_silk_fold(position: vec2<f32>, seed: f32, frequency: f32, lightweight: bool, time: f32) -> vec3<f32> {
  let warp = gen_silk_warp(position + seed * 3.7, seed, lightweight);
  let warped = position + warp * 0.55;
  // The reference rolls each band of the drape along with `iTime`.
  let phase = time;
  var height = 0.0;
  var slope = vec2<f32>(0.0);
  let first = frequency * vec2<f32>(0.7, 0.4);
  let first_phase = dot(warped, first) + seed * 2.1 + phase;
  height += sin(first_phase) * 0.35;
  slope += cos(first_phase) * first * 0.35;
  let second = frequency * vec2<f32>(-0.3, 0.9);
  let second_phase = dot(warped, second) + seed * 1.3 - phase * 0.8;
  height += sin(second_phase) * 0.25;
  slope += cos(second_phase) * second * 0.25;
  let diagonal = frequency * 0.6;
  let diagonal_phase = (warped.x + warped.y) * diagonal + seed * 4.5 + phase * 1.3;
  height += sin(diagonal_phase) * 0.18;
  slope += cos(diagonal_phase) * vec2<f32>(diagonal) * 0.18;
  let detail = frequency * vec2<f32>(1.8, 1.2);
  let detail_phase = dot(warped, detail) + seed * 0.7 - phase * 0.6;
  height += sin(detail_phase) * 0.08;
  slope += cos(detail_phase) * detail * 0.08;
  if (!lightweight) {
    height += gen_noise2(warped * frequency * 0.9 + seed * 10.0) * 0.12 - 0.06;
  }
  return vec3<f32>(height, slope);
}

fn gen_silk_specular(slope: vec2<f32>, light: vec3<f32>, shine: f32) -> f32 {
  let squared = dot(slope, slope);
  if (squared < 0.0001) {
    return 0.0;
  }
  let across = vec2<f32>(-slope.y, slope.x) / sqrt(squared);
  let tangent = normalize(vec3<f32>(across, 0.0));
  let halfway = normalize(light + vec3<f32>(0.0, 0.0, 1.0));
  let aligned = dot(tangent, halfway);
  return pow(sqrt(max(1.0 - aligned * aligned, 0.0)), shine);
}

fn gen_silk_layer(
  position: vec2<f32>,
  seed: f32,
  frequency: f32,
  tone: vec3<f32>,
  opacity: f32,
  shine: f32,
  sheen: f32,
  time: f32,
) -> vec4<f32> {
  let primary = normalize(vec3<f32>(0.4, 1.05, 0.8));
  let secondary = normalize(vec3<f32>(-0.5, -0.3, 0.6));
  let lightweight = opacity < 0.35;
  let fold = gen_silk_fold(position, seed, frequency, lightweight, time);
  let slope = fold.yz;
  let normal = normalize(vec3<f32>(-slope * 1.8, 1.0));
  let lighting = max(dot(normal, primary), 0.0) * 0.75 + max(dot(normal, secondary), 0.0) * 0.12;
  let depth = smoothstep(-0.8, 0.4, fold.x);
  let shade = lighting * depth;
  var fabric = mix(tone * 0.14, tone * 0.55, smoothstep(0.0, 0.35, shade));
  fabric = mix(fabric, tone * 0.95, smoothstep(0.25, 0.7, shade) * 0.5);
  var specular = gen_silk_specular(slope, primary, shine) * 0.9;
  specular += gen_silk_specular(slope, secondary, shine * 0.6) * 0.15;
  specular *= sheen;
  fabric += mix(tone, vec3<f32>(1.0), 0.72) * specular * specular * specular * 0.9;
  fabric += vec3<f32>(0.45, 0.28, 0.15) * smoothstep(0.3, 0.9, depth) * lighting * 0.08;
  return vec4<f32>(fabric, opacity * (0.65 + depth * 0.35));
}

/// Silk. Folds 1, drape 100%, sheen 100%, saturation 135%, vignette 100%.
fn gen_silk(pixel: vec2<f32>, dimensions: vec2<f32>, palette: GenPalette, shift: vec3<f32>, time: f32) -> vec3<f32> {
  let point = gen_centred(pixel, dimensions, shift);
  let background = palette.c0;
  let first = gen_palette_at(palette, 1u);
  let second = gen_palette_at(palette, 2u);
  let third = gen_palette_at(palette, 3u);
  let distance = length(point);
  var color = background + background * 0.5 * exp(-distance * distance * 2.0);
  let back = gen_silk_layer(point * 0.8 + vec2<f32>(0.15, 0.0), 0.0, 2.0, first, 0.30, 26.0, 0.7, time);
  let middle = gen_silk_layer(point + vec2<f32>(0.0, -0.1), 1.0, 3.2, second, 0.38, 40.0, 0.9, time);
  let front = gen_silk_layer(point * 1.2, 2.0, 4.5, third, 0.50, 55.0, 1.0, time);
  color = mix(color, back.rgb, back.a);
  color += first * 0.10 * back.a * middle.a;
  color = mix(color, middle.rgb, middle.a);
  color += second * 0.08 * middle.a * front.a;
  color = mix(color, front.rgb, front.a);
  color += third * 0.05 * (back.a + middle.a + front.a) / 3.0;
  let vignette = 1.0 - smoothstep(0.25, 1.15, length(point * vec2<f32>(0.85, 1.0)));
  color *= 0.6 + 0.4 * vignette;
  color = mix(vec3<f32>(dot(color, vec3<f32>(0.299, 0.587, 0.114))), color, 1.35);
  color = max(color, vec3<f32>(0.0));
  return color * (2.51 * color + 0.03) / (color * (2.43 * color + 0.59) + 0.14);
}

fn gen_strata_fbm(start: vec2<f32>) -> f32 {
  var position = start;
  var value = 0.0;
  var amplitude = 0.5;
  for (var index = 0; index < 5; index += 1) {
    value += amplitude * gen_noise2(position);
    position = vec2<f32>(
      0.877582562 * position.x - 0.479425539 * position.y,
      0.479425539 * position.x + 0.877582562 * position.y,
    ) * 2.0 + vec2<f32>(100.0);
    amplitude *= 0.5;
  }
  return value;
}

/// The plates pushing the bands out of true. Tectonics 60%.
fn gen_strata_warp(position: vec2<f32>, time: f32) -> vec2<f32> {
  // The reference drifts the plates with `iTime`.
  let phase = time;
  let across = gen_strata_fbm(position * 1.5 + vec2<f32>(phase, 0.0)) - 0.5;
  let down = gen_strata_fbm(position * 1.5 + vec2<f32>(50.0, 30.0 + phase)) - 0.5;
  let compression = sin(position.x * 2.0 + phase) * 0.08;
  let shear = sin(position.y * 3.0 - phase * 0.7) * 0.06;
  return position + vec2<f32>(across * 0.25 + shear, down * 0.18 + compression) * 0.6;
}

fn gen_strata_boundary(across: f32, base: f32, layer: f32) -> f32 {
  let first = gen_hash11(layer * 7.13);
  let second = gen_hash11(layer * 13.37);
  let third = gen_hash11(layer * 23.71);
  var fold = (0.04 + first * 0.06) * 0.6 * sin(across * (1.5 + first * 2.5) + second * 6.28318530718);
  fold += (0.015 + second * 0.025) * 0.6 * sin(across * (3.0 + second * 3.0) + third * 6.28318530718);
  fold += gen_noise2(vec2<f32>(across * 4.0 + first * 100.0, layer)) * 0.6 * 0.02;
  return base + fold;
}

fn gen_strata_grain(position: vec2<f32>, layer: f32) -> f32 {
  let angle = gen_hash11(layer * 41.93) * 0.3 * 3.141592654;
  let cosine = cos(angle);
  let sine = sin(angle);
  let turned = vec2<f32>(
    position.x * cosine - position.y * sine,
    position.x * sine + position.y * cosine,
  );
  var value = gen_noise2(vec2<f32>(turned.x * 120.0, turned.y * 18.0) + layer * 30.0) * 0.08;
  value += gen_noise2(vec2<f32>(turned.x * 80.0, turned.y * 12.0) + layer * 50.0 + 100.0) * 0.05;
  value += gen_noise2(vec2<f32>(turned.x * 15.0, turned.y * 70.0) + layer * 40.0) * 0.03;
  value += gen_noise2(position * 50.0 + layer * 25.0) * 0.025;
  return value - 0.04;
}

/// Strata. Layers 12, tectonics 60%, texture 70%, saturation 82%, vignette 100%.
fn gen_strata_sample(pixel: vec2<f32>, dimensions: vec2<f32>, palette: GenPalette, shift: vec3<f32>, time: f32) -> vec3<f32> {
  let frame = gen_place(pixel / dimensions - 0.5, shift) + 0.5;
  let warped = gen_strata_warp(vec2<f32>(frame.x * dimensions.x / dimensions.y, frame.y), time);
  let spacing = 1.0 / 13.0;
  var layer = -1.0;
  var along = 0.0;
  var previous = -0.2;
  for (var index = 0; index < 12; index += 1) {
    let boundary = gen_strata_boundary(warped.x, f32(index + 1) * spacing, f32(index));
    if (warped.y >= previous && warped.y < boundary) {
      layer = f32(index);
      along = (warped.y - previous) / max(boundary - previous, 0.001);
      break;
    }
    previous = boundary;
  }
  if (layer < 0.0) {
    layer = 12.0;
    along = 0.5;
  }
  let chosen = gen_hash11(layer * 17.31 + 3.7) * f32(max(palette.count, 1u));
  var color = gen_palette_at(palette, u32(clamp(floor(chosen), 0.0, f32(max(palette.count, 1u) - 1u))));
  color += (gen_hash11(layer * 31.17) - 0.5) * 0.06;
  color += gen_strata_grain(warped, layer) * 0.7;
  color *= 0.85 + 0.15 * smoothstep(0.0, 0.15, along) * (1.0 - smoothstep(0.85, 1.0, along));
  color *= 0.82 + 0.18 * smoothstep(0.0, 0.08, along);
  color += vec3<f32>(0.04, 0.035, 0.025) * smoothstep(0.92, 1.0, along);
  color *= clamp(1.0 - 0.3 * length((frame - 0.5) * 1.5), 0.0, 1.0);
  color += gen_noise2(pixel * 0.15) * 0.03 * 0.7 + (gen_hash21(pixel) - 0.5) * 0.015 * 0.7;
  return mix(vec3<f32>(dot(color, vec3<f32>(0.299, 0.587, 0.114))), color, 0.82);
}

fn gen_strata(pixel: vec2<f32>, dimensions: vec2<f32>, palette: GenPalette, shift: vec3<f32>, time: f32) -> vec3<f32> {
  let quarter = vec2<f32>(0.25, 0.25);
  return (gen_strata_sample(pixel - quarter, dimensions, palette, shift, time) +
          gen_strata_sample(pixel + vec2<f32>(quarter.x, -quarter.y), dimensions, palette, shift, time) +
          gen_strata_sample(pixel + vec2<f32>(-quarter.x, quarter.y), dimensions, palette, shift, time) +
          gen_strata_sample(pixel + quarter, dimensions, palette, shift, time)) * 0.25;
}

/// The generator a canvas names, by the id the settings carry. Zero is the
/// app's own mesh, which the caller handles rather than this switch.
///
/// `time` is the canvas seconds and `speed` the generator's factor from the
/// table in `mesh_generator.rs`; the two are multiplied here, once, and every
/// generator below reads the product as its own `time`. Zero paints the still
/// a thumbnail and a screenshot export ask for.
fn gen_pixel(
  generator: u32,
  pixel: vec2<f32>,
  dimensions: vec2<f32>,
  palette: GenPalette,
  seed: u32,
  time: f32,
  speed: f32,
) -> vec3<f32> {
  let shift = gen_seed_shift(seed);
  let t = time * speed;
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
