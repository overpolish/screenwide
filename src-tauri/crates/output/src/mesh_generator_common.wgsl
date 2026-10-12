// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The pieces every ported generator shares: the hashes and value noises the
// reference shaders are written against, a palette small enough to pass by
// value, and the one thing this port adds, a per-seed move of the domain so
// that Randomize gives a different picture out of the same colours.
//
// Every generator takes as its `time` the canvas seconds the classic mesh
// drifts with, scaled once in `gen_pixel` by the generator's `speed` from the
// table in `mesh_generator.rs`. A still is that machinery at time zero, which
// is what thumbnails and screenshot export ask for.

struct GenPalette {
  c0: vec3<f32>,
  c1: vec3<f32>,
  c2: vec3<f32>,
  c3: vec3<f32>,
  count: u32,
}

fn gen_palette_at(palette: GenPalette, index: u32) -> vec3<f32> {
  let last = max(palette.count, 1u) - 1u;
  let wanted = min(index, last);
  if (wanted == 0u) { return palette.c0; }
  if (wanted == 1u) { return palette.c1; }
  if (wanted == 2u) { return palette.c2; }
  return palette.c3;
}

/// A palette read as a ramp: the reference generators that take a position
/// along their colours all share this walk between neighbours.
fn gen_ramp(palette: GenPalette, position: f32) -> vec3<f32> {
  let count = max(palette.count, 1u);
  if (count == 1u) { return palette.c0; }
  let scaled = clamp(position, 0.0, 1.0) * f32(count - 1u);
  let lower = u32(floor(scaled));
  return mix(
    gen_palette_at(palette, lower),
    gen_palette_at(palette, lower + 1u),
    fract(scaled),
  );
}

fn gen_hash11(value: f32) -> f32 {
  var carried = fract(value * 0.1031);
  carried = carried * (carried + 33.33);
  carried = carried * (carried + carried);
  return fract(carried);
}

fn gen_hash21(position: vec2<f32>) -> f32 {
  var carried = fract(vec3<f32>(position.x, position.y, position.x) * 0.1031);
  carried += dot(carried, carried.yzx + 33.33);
  return fract((carried.x + carried.y) * carried.z);
}

fn gen_hash31(position: vec3<f32>) -> f32 {
  var carried = fract(position * 0.1031);
  carried += dot(carried, carried.zyx + 31.32);
  return fract((carried.x + carried.y) * carried.z);
}

fn gen_noise2(position: vec2<f32>) -> f32 {
  let cell = floor(position);
  let local = fract(position);
  let eased = local * local * (3.0 - 2.0 * local);
  return mix(
    mix(gen_hash21(cell), gen_hash21(cell + vec2<f32>(1.0, 0.0)), eased.x),
    mix(gen_hash21(cell + vec2<f32>(0.0, 1.0)), gen_hash21(cell + vec2<f32>(1.0, 1.0)), eased.x),
    eased.y,
  );
}

fn gen_noise3(position: vec3<f32>) -> f32 {
  let cell = floor(position);
  let local = fract(position);
  let eased = local * local * (3.0 - 2.0 * local);
  let lower = mix(
    mix(gen_hash31(cell), gen_hash31(cell + vec3<f32>(1.0, 0.0, 0.0)), eased.x),
    mix(gen_hash31(cell + vec3<f32>(0.0, 1.0, 0.0)), gen_hash31(cell + vec3<f32>(1.0, 1.0, 0.0)), eased.x),
    eased.y,
  );
  let upper = mix(
    mix(gen_hash31(cell + vec3<f32>(0.0, 0.0, 1.0)), gen_hash31(cell + vec3<f32>(1.0, 0.0, 1.0)), eased.x),
    mix(gen_hash31(cell + vec3<f32>(0.0, 1.0, 1.0)), gen_hash31(cell + vec3<f32>(1.0, 1.0, 1.0)), eased.x),
    eased.y,
  );
  return mix(lower, upper, eased.z);
}

/// The reference shaders are written with GLSL's column-major `mat2`, where
/// `mat2(a, b, c, d) * v` is `(a * v.x + c * v.y, b * v.x + d * v.y)`. Writing
/// the two rotations out avoids carrying that convention into three shading
/// languages.
fn gen_rotate(position: vec2<f32>, angle: f32) -> vec2<f32> {
  let sine = sin(angle);
  let cosine = cos(angle);
  return vec2<f32>(cosine * position.x + sine * position.y, -sine * position.x + cosine * position.y);
}

/// The seed as a move of the domain: a turn and a slide, applied to the
/// pattern position before a generator reads it. The generators have no seed
/// of their own, so this is what a fresh seed changes, and it is what keeps
/// two tiles of the same generator distinct at time zero.
fn gen_seed_shift(seed: u32) -> vec3<f32> {
  let value = f32(seed);
  let first = fract(sin(value * 12.9898 + 4.1) * 43758.5453);
  let second = fract(sin(value * 78.2330 + 1.7) * 43758.5453);
  let third = fract(sin(value * 39.4250 + 9.3) * 43758.5453);
  return vec3<f32>((first - 0.5) * 1.5, (second - 0.5) * 1.5, third * 6.2831853);
}

fn gen_place(position: vec2<f32>, shift: vec3<f32>) -> vec2<f32> {
  let sine = sin(shift.z);
  let cosine = cos(shift.z);
  return vec2<f32>(
    position.x * cosine - position.y * sine,
    position.x * sine + position.y * cosine,
  ) + shift.xy;
}

/// The frame every generator but Strata and Paint Mixer works in: the canvas
/// centred and measured in heights, as the reference takes it with its centre
/// and scale left at their defaults.
fn gen_centred(pixel: vec2<f32>, dimensions: vec2<f32>, shift: vec3<f32>) -> vec2<f32> {
  return gen_place((pixel - dimensions * 0.5) / dimensions.y, shift);
}
