// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The camera's effects, drawn into a picture the size of the camera frame
// before the canvas samples it. The matte says how much of each pixel is the
// person; what shows where they are not is nothing, the room blurred, or a
// picture. Enhance sharpens the recorded camera with FSR 1's RCAS
// (`fsr1.wgsl`) before the rest, so the room behind and a picture are left
// as they are, and corrects the colour of the person and the room blurred
// behind them (`colour_correction.wgsl`).
//
// The blurred room arrives blurred whole by the Gaussian a blur redaction
// draws, so the two look alike.

struct Effects {
  // x: the background, 0 kept, 1 nothing, 2 blurred, 3 a picture.
  // y: 1 where the colour is corrected, which waits for its measure. z: 1
  // where the camera is sharpened.
  mode: vec4<u32>,
  // The colour correction's gains, in linear light.
  gain: vec4<f32>,
  // The picture's part of its texture the frame covers: scale, then offset.
  picture: vec4<f32>,
  // The size of the target drawn into, in pixels.
  target_size: vec4<f32>,
}

@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var matte: texture_2d<f32>;
@group(0) @binding(2) var backdrop: texture_2d<f32>;
@group(0) @binding(3) var linear_sampler: sampler;
@group(0) @binding(4) var<uniform> effects: Effects;

@vertex
fn vs_main(@builtin(vertex_index) id: u32) -> @builtin(position) vec4<f32> {
  let corner = vec2<f32>(f32((id << 1u) & 2u), f32(id & 2u));
  return vec4<f32>(corner * 2.0 - 1.0, 0.0, 1.0);
}

fn uv_of(position: vec4<f32>) -> vec2<f32> {
  return position.xy / effects.target_size.xy;
}

fn corrected(rgb: vec3<f32>) -> vec3<f32> {
  if (effects.mode.y == 0u) {
    return rgb;
  }
  return colour_correction(rgb, effects.gain.rgb);
}

// The colour of the person around `uv`, from the pixels the matte holds
// solid within about ten pixels of a 1080-line frame, or nothing where none
// is. A soft edge pixel mixes the person with the room behind them; drawn
// over a new background the room shows as a pale rim, so the soft edge
// takes its colour from the person instead.
fn inner_colour(uv: vec2<f32>) -> vec4<f32> {
  let frame = vec2<f32>(textureDimensions(source));
  let reach = frame.y / 1080.0 / frame;
  var sum = vec3<f32>(0.0);
  var weights = 0.0;
  for (var ring = 1; ring <= 3; ring++) {
    let radius = f32(ring) * 3.5;
    let fall = exp(-radius * radius / 50.0);
    for (var turn = 0; turn < 8; turn++) {
      let angle = (f32(turn) + 0.5 * f32(ring % 2)) * 0.7853982;
      let at = uv + vec2<f32>(cos(angle), sin(angle)) * radius * reach;
      let solid = step(0.97, textureSampleLevel(matte, linear_sampler, at, 0.0).r);
      sum += textureSampleLevel(source, linear_sampler, at, 0.0).rgb * solid * fall;
      weights += solid * fall;
    }
  }
  if (weights <= 0.0) {
    return vec4<f32>(0.0);
  }
  return vec4<f32>(sum / weights, 1.0);
}

// The recorded camera at `position`, sharpened where Enhance is on. The
// target is the frame's own size, so each fragment is one of its texels.
fn recorded(position: vec4<f32>, uv: vec2<f32>) -> vec3<f32> {
  if (effects.mode.z == 0u) {
    return textureSampleLevel(source, linear_sampler, uv, 0.0).rgb;
  }
  return fsr_rcas(source, vec2<i32>(position.xy));
}

// The person over their background. Where there is none the picture keeps
// its alpha straight, as the canvas reads a camera's.
@fragment
fn fs_effects(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
  let uv = uv_of(position);
  var person = recorded(position, uv);
  let background = effects.mode.x;
  if (background == 0u) {
    return vec4<f32>(corrected(person), 1.0);
  }
  let alpha = textureSampleLevel(matte, linear_sampler, uv, 0.0).r;
  if (alpha > 0.0 && alpha < 0.97) {
    let inner = inner_colour(uv);
    let share = saturate((0.97 - alpha) / 0.5) * inner.a;
    person = mix(person, inner.rgb, share);
  }
  person = corrected(person);
  if (background == 1u) {
    return vec4<f32>(person, alpha);
  }
  var behind: vec3<f32>;
  if (background == 2u) {
    behind = corrected(textureSampleLevel(backdrop, linear_sampler, uv, 0.0).rgb);
  } else {
    let at = uv * effects.picture.xy + effects.picture.zw;
    behind = textureSampleLevel(backdrop, linear_sampler, at, 0.0).rgb;
  }
  return vec4<f32>(mix(behind, person, alpha), 1.0);
}
