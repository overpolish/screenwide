// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// What the canvas sits on: a picture, the classic mesh, a ported generator
// or one colour. While a scene's background blooms in over the one before
// it, both are drawn: the incoming one, the canvas's own, inside a soft
// circle spreading from where the incoming screen lands, the outgoing one
// outside it, each scaled about that point as the screen's zoom pushes them.
// Part of the `preview` module, after `preview.wgsl`.

// The twin of `TurnRows` in `compositor/background_turn.rs`: the background
// giving way, in the canvas's own background rows, then the bloom.
struct BackgroundTurn {
  solid_color: vec4<f32>,
  base_color: vec4<f32>,
  mesh_points: array<vec4<f32>, 8>,
  mesh_colors: array<vec4<f32>, 4>,
  options: vec4<u32>, // seed, mesh enabled, point count, has picture
  style: vec4<u32>, // generator, palette size
  params: vec4<f32>, // warp percent, generator speed
  // Where the bloom spreads from, in canvas pixels, the farthest canvas
  // corner from there, and how far it has spread, eased.
  bloom: vec4<f32>,
  // How large the outgoing and the incoming backgrounds are drawn about
  // the bloom's centre, then whether one is giving way at all.
  scale: vec4<f32>,
}

// One background, read from the canvas's own rows or the turn's.
struct Ground {
  solid_color: vec3<f32>,
  base_color: vec3<f32>,
  mesh_points: array<vec4<f32>, 8>,
  mesh_colors: array<vec4<f32>, 4>,
  options: vec4<u32>, // seed, mesh enabled, point count, has picture
  style: vec4<u32>, // generator, palette size
  params: vec4<f32>, // warp percent, generator speed
}

fn own_ground() -> Ground {
  return Ground(canvas.solid_color.rgb, canvas.base_color.rgb, canvas.mesh_points,
                canvas.mesh_colors,
                vec4<u32>(canvas.options.xyz, canvas.background_options.x),
                vec4<u32>(canvas.background_options.yz, 0u, 0u),
                vec4<f32>(canvas.effects.z, canvas.motion.y, 0.0, 0.0));
}

fn outgoing_ground() -> Ground {
  let turn = canvas.turn;
  return Ground(turn.solid_color.rgb, turn.base_color.rgb, turn.mesh_points, turn.mesh_colors,
                turn.options, turn.style, turn.params);
}

// A chosen picture covers the canvas: it is scaled until both sides reach,
// centred, and the overflowing axis is trimmed evenly, so it never letterboxes
// and never stretches. The same framing as `cover_fit` in the CPU compose
// path, done here in UVs so one upload serves every canvas size. A picture
// drawn smaller than the canvas while it is pushed is mirrored at its edges.
fn ground_picture(outgoing: bool, solid: vec3<f32>, pixel: vec2<f32>) -> vec3<f32> {
  var picture = vec2<f32>(textureDimensions(background_image));
  if (outgoing) {
    picture = vec2<f32>(textureDimensions(turn_image));
  }
  if (picture.x == 0.0 || picture.y == 0.0) {
    return solid;
  }
  let output = canvas.output_source.xy;
  let scale = max(output.x / picture.x, output.y / picture.y);
  let covered = picture * scale;
  let origin = (output - covered) * 0.5;
  let along = (pixel - origin) / covered;
  let folded = 1.0 - abs(1.0 - (along - 2.0 * floor(along * 0.5)));
  if (outgoing) {
    return textureSample(turn_image, linear_sampler, folded).rgb;
  }
  return textureSample(background_image, linear_sampler, folded).rgb;
}

fn ground_color(ground: Ground, outgoing: bool, pixel: vec2<f32>) -> vec3<f32> {
  if (feature_kept(feature_background_picture) && ground.options.w != 0u) {
    return ground_picture(outgoing, ground.solid_color, pixel);
  }
  if (ground.options.y == 0u) {
    return ground.solid_color;
  }
  // A ported generator reads its palette from the front of the mesh colours,
  // the seed, and the same timeline seconds the classic mesh drifts with,
  // which `gen_pixel_shifted` scales by the generator's own speed.
  if (feature_kept(feature_background_generator) && ground.style.x != 0u) {
    let palette = GenPalette(ground.mesh_colors[0].rgb, ground.mesh_colors[1].rgb,
                             ground.mesh_colors[2].rgb, ground.mesh_colors[3].rgb,
                             ground.style.y);
    // Ported generators reuse the classic mesh point slot for the seed's
    // domain shift, resolved on the CPU unless its last word asks for the
    // shader's own hash; only the classic generator reads point geometry.
    var shift = ground.mesh_points[0].xyz;
    if (ground.mesh_points[0].w != 0.0) {
      shift = gen_seed_shift(ground.options.x);
    }
    return gen_pixel_shifted(ground.style.x, pixel, canvas.output_source.xy, palette, shift,
                             canvas.motion.x, ground.params.y);
  }
  if (!feature_kept(feature_background_mesh)) {
    return ground.solid_color;
  }
  let shortest = min(canvas.output_source.x, canvas.output_source.y);
  let dimensions = canvas.output_source.xy;
  let aspect = dimensions / shortest;
  let frequency = 3.5 / shortest;
  let phase = canvas.motion.x * 0.28;
  let drift = vec2<f32>(sin(phase), cos(phase * 0.83)) * shortest * 0.012;
  let warped_pixel = pixel + drift;
  let warp_scale = shortest * ground.params.x / 100.0;
  let seed = ground.options.x;
  let warp = vec2<f32>(
      fractal_noise(warped_pixel * frequency + phase * 0.035, seed),
      fractal_noise(warped_pixel * frequency + vec2<f32>(19.7, -7.3) - phase * 0.03,
                    seed ^ 0xa511e9b3u)) * warp_scale;
  var weighted = ground.base_color * 0.18;
  var total = 0.18;
  for (var index = 0u; index < ground.options.z; index++) {
    let first = ground.mesh_points[index * 2u];
    let second = ground.mesh_points[index * 2u + 1u];
    let local_phase = phase + f32(index) * 1.73;
    let animated_center = first.xy + vec2<f32>(sin(local_phase), cos(local_phase * 0.91)) * 0.012;
    let delta = (pixel + warp) / shortest - animated_center * aspect;
    let rotated = vec2<f32>(delta.x * second.x + delta.y * second.y,
                            -delta.x * second.y + delta.y * second.x);
    let reach = length(rotated / max(first.zw, vec2<f32>(0.01)));
    let weight = 1.0 / (pow(max(reach, 0.025), 3.5) + 0.012);
    weighted += ground.mesh_colors[index].rgb * weight;
    total += weight;
  }
  let result = weighted / total;
  let depth = fractal_noise((pixel + drift) * frequency * 0.7, seed ^ 0xd1b54a35u) * 13.0 / 255.0;
  return saturate(result + depth);
}

// The two backgrounds are mixed in linear light, as light adds up, so the
// bloom's soft edge keeps its brightness rather than dipping through grey.
fn ground_to_linear(color: vec3<f32>) -> vec3<f32> {
  return select(pow((color + 0.055) / 1.055, vec3<f32>(2.4)), color / 12.92,
                color <= vec3<f32>(0.04045));
}

fn ground_from_linear(color: vec3<f32>) -> vec3<f32> {
  let light = saturate(color);
  return select(1.055 * pow(light, vec3<f32>(1.0 / 2.4)) - 0.055, light * 12.92,
                light <= vec3<f32>(0.0031308));
}

fn background(pixel: vec2<f32>) -> vec3<f32> {
  if (!feature_kept(feature_background_turn) || canvas.turn.scale.z == 0.0) {
    return ground_color(own_ground(), false, pixel);
  }
  let turn = canvas.turn;
  let centre = turn.bloom.xy;
  // The edge is soft over most of the canvas, so the new background reads as
  // light spreading rather than a circle growing.
  let feather = turn.bloom.z * 0.6;
  let radius = turn.bloom.w * (turn.bloom.z + feather);
  let inside = 1.0 - smoothstep(radius - feather, radius, distance(pixel, centre));
  let incoming = ground_color(own_ground(), false, centre + (pixel - centre) / turn.scale.y);
  let outgoing = ground_color(outgoing_ground(), true, centre + (pixel - centre) / turn.scale.x);
  return ground_from_linear(mix(ground_to_linear(outgoing), ground_to_linear(incoming), inside));
}
