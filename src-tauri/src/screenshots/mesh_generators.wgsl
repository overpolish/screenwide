// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Six of the eight ported generators. Each is the reference `mainImage` with
// every knob but the colours pinned to the value the reference declares as its
// default, so what the picker offers is the picture the reference library
// shows as that generator's preview.
//
// `time` is the canvas seconds already scaled by the generator's `speed`, the
// factor tabled in `mesh_generator.rs`; `gen_pixel` applies it once, so no
// generator carries a pace of its own. Zero is the still a thumbnail and a
// screenshot export ask for.

/// Aurora. Detail 8, warp 100%, palette mix 70%, sheen 100%.
fn gen_aurora(pixel: vec2<f32>, dimensions: vec2<f32>, palette: GenPalette, shift: vec3<f32>, time: f32) -> vec3<f32> {
  let shortest = min(dimensions.x, dimensions.y);
  let point = gen_place((pixel - dimensions * 0.5) * (2.0 / shortest), shift);
  // The reference winds the accumulator forward from `iTime`.
  let phase = time;
  var first = 0.0;
  var second = phase;
  for (var index = 0; index < 8; index += 1) {
    first += cos(f32(index) + second + first * point.y);
    second += sin(f32(index) * point.x - first);
  }
  let base = 0.55 + 0.45 * cos(vec3<f32>(first, second, first + second) * 0.6 + vec3<f32>(0.0, 2.0, 4.0));
  let modulation = vec3<f32>(cos(0.7 * second), cos(0.7 * first), sin(0.9 * (first + second)));
  let iridescence = cos(base * modulation * 0.5 + 0.5);
  let sheen = clamp(dot(iridescence, vec3<f32>(0.299, 0.587, 0.114)), 0.0, 1.0);
  let hue = 0.5 + 0.5 * sin(1.2 * point.x + 0.8 * point.y + 0.25 * first);
  let recoloured = sheen * mix(vec3<f32>(1.0), gen_ramp(palette, hue), 0.9);
  return mix(iridescence, recoloured, 0.7);
}

/// Gentle Gradient: a bilinear corner mix under a swirl. Swirl 100%, gloss 100%.
fn gen_gentle(pixel: vec2<f32>, dimensions: vec2<f32>, palette: GenPalette, shift: vec3<f32>, time: f32) -> vec3<f32> {
  let point = gen_centred(pixel, dimensions, shift);
  let phase = time;
  let angle = (sin(point.x * 2.0 + phase) + cos(point.y * 2.3 - phase * 0.8)) * 0.6;
  let swirled = gen_rotate(point, angle);
  let ramp = swirled * 0.5 + 0.5;
  var color = mix(palette.c0, gen_palette_at(palette, 1u), clamp(ramp.x, 0.0, 1.0));
  color = mix(color, gen_palette_at(palette, 2u), clamp(ramp.y, 0.0, 1.0));
  return clamp(color * (vec3<f32>(0.85) + color * 0.35), vec3<f32>(0.0), vec3<f32>(1.0));
}

/// Ribbons. Bands 6, ripple 100%, direction 225 degrees.
fn gen_ribbon_phase(point: vec2<f32>, time: f32) -> f32 {
  let direction = vec2<f32>(cos(3.926990817), sin(3.926990817));
  let flow = dot(point, direction) * 3.0 + time;
  let across = sin(point.x * 12.0 + time) * cos(point.y * 8.0) * 0.3;
  let down = cos(point.y * 10.0 - time * 0.8) * sin(point.x * 7.0) * 0.4;
  return flow + (across + down) * 0.5;
}

fn gen_ribbons(pixel: vec2<f32>, dimensions: vec2<f32>, palette: GenPalette, shift: vec3<f32>, time: f32) -> vec3<f32> {
  let point = gen_centred(pixel, dimensions, shift);
  let ramp = fract(gen_ribbon_phase(point, time));
  let point_xp = gen_centred(pixel + vec2<f32>(0.5, 0.0), dimensions, shift);
  let point_xm = gen_centred(pixel - vec2<f32>(0.5, 0.0), dimensions, shift);
  let point_yp = gen_centred(pixel + vec2<f32>(0.0, 0.5), dimensions, shift);
  let point_ym = gen_centred(pixel - vec2<f32>(0.0, 0.5), dimensions, shift);
  let footprint = 0.5 * (abs(gen_ribbon_phase(point_xp, time) - gen_ribbon_phase(point_xm, time)) +
                         abs(gen_ribbon_phase(point_yp, time) - gen_ribbon_phase(point_ym, time)));
  let band = floor(ramp * 6.0);
  let fraction = ramp * 6.0 - band;
  let width = min(max(footprint * 6.0, 0.0001), 0.5);
  let color = gen_ramp(palette, band / 5.0);
  let previous = select(5.0, band - 1.0, band > 0.0);
  let next = select(0.0, band + 1.0, band < 5.0);
  let blended = mix(gen_ramp(palette, previous / 5.0), color, smoothstep(-width, width, fraction));
  return mix(blended, gen_ramp(palette, next / 5.0), smoothstep(-width, width, fraction - 1.0));
}

/// Currents. Detail 5, warp 100%, turbulence 100%.
fn gen_currents(pixel: vec2<f32>, dimensions: vec2<f32>, palette: GenPalette, shift: vec3<f32>, time: f32) -> vec3<f32> {
  let point = gen_centred(pixel, dimensions, shift);
  let phase = time;
  let turbulence = gen_noise3(vec3<f32>(point * 2.0, phase)) * 2.0 - 1.0;
  var warped = point * 1.2;
  var amplitude = 0.7;
  for (var index = 0; index < 5; index += 1) {
    let frequency = f32(index) + 1.0;
    let offset = vec2<f32>(
      sin(frequency * warped.y + turbulence * frequency + phase),
      cos(frequency * warped.x + turbulence - phase),
    );
    warped -= amplitude * offset;
    amplitude *= 0.6;
  }
  let primary = 0.5 + 0.5 * sin(0.8 * (warped.x + warped.y) + turbulence);
  let secondary = 0.5 + 0.5 * cos(warped.x - warped.y);
  let color = mix(gen_ramp(palette, primary), gen_ramp(palette, secondary), 0.35);
  return color * (0.75 + 0.25 * sin(warped.y + turbulence));
}

/// Paint Mixer: the four colours as the corners of a mirrored bilinear mix.
/// Complexity 16, warp 100%, frequency 50%.
fn gen_paint(pixel: vec2<f32>, dimensions: vec2<f32>, palette: GenPalette, shift: vec3<f32>, time: f32) -> vec3<f32> {
  var point = gen_place((pixel - dimensions * 0.5) / dimensions, shift) + 0.5;
  let drift = time;
  for (var index = 1; index <= 16; index += 1) {
    let frequency = f32(index) * 5.0 + 1.0;
    let amplitude = 1.0 / frequency;
    let phase = frequency * 0.3 + drift;
    point.x += amplitude * sin(frequency * point.y + phase);
    point.y += amplitude * sin(frequency * point.x + phase + 30.0);
  }
  let mirrored = abs(2.0 * fract(point * 0.5) - 1.0);
  let top = mix(palette.c0, gen_palette_at(palette, 1u), mirrored.x);
  let bottom = mix(gen_palette_at(palette, 2u), gen_palette_at(palette, 3u), mirrored.x);
  return mix(top, bottom, mirrored.y);
}

fn gen_fluid_fbm(start: vec3<f32>, decay: f32) -> f32 {
  var position = start;
  var value = 0.0;
  var amplitude = 0.5;
  for (var index = 0; index < 5; index += 1) {
    value += amplitude * (gen_noise3(position) * 2.0 - 1.0);
    let rotated = gen_rotate(position.xy, 0.7) * 2.03 + vec2<f32>(1.7, 9.2);
    position = vec3<f32>(rotated, position.z + 2.3);
    amplitude *= decay;
  }
  return value;
}

/// Fluid: domain-warped marble. Detail 50%, marble 100%, vibrance 100%.
fn gen_fluid(pixel: vec2<f32>, dimensions: vec2<f32>, palette: GenPalette, shift: vec3<f32>, time: f32) -> vec3<f32> {
  let point = gen_centred(pixel, dimensions, shift);
  let detail = 0.5;
  // The reference walks the noise's third axis with `iTime`.
  let phase = time;
  let first = vec2<f32>(
    gen_fluid_fbm(vec3<f32>(point, phase), detail),
    gen_fluid_fbm(vec3<f32>(point + vec2<f32>(5.2, 1.3), phase), detail),
  );
  let second = vec2<f32>(
    gen_fluid_fbm(vec3<f32>(point + 4.0 * first + vec2<f32>(1.7, 9.2), phase), detail),
    gen_fluid_fbm(vec3<f32>(point + 4.0 * first + vec2<f32>(8.3, 2.8), phase), detail),
  );
  let field = gen_fluid_fbm(vec3<f32>(point + 3.5 * second, phase), detail);
  let third = gen_palette_at(palette, 3u);
  var color = mix(palette.c0, gen_palette_at(palette, 1u), clamp(field * field * 2.0, 0.0, 1.0));
  color = mix(color, gen_palette_at(palette, 2u), clamp(length(first) * 0.5, 0.0, 1.0));
  color = mix(color, third, clamp(abs(second.x) * 0.6, 0.0, 1.0));
  let highlight = smoothstep(0.5, 1.2, field * field * 3.0 + length(second) * 0.5);
  color += third * 0.35 * highlight;
  return pow(max(color, vec3<f32>(0.0)), vec3<f32>(1.1));
}
