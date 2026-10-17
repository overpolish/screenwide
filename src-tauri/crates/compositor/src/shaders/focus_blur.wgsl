// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The focus blur: what lies under the topmost focus, already drawn into the
// first level of a chain the size of the target, softened everywhere outside
// the focus boxes. A pixel on the screen softens by how far the point of the
// upright screen it shows lies past a box's rounded edge and, on a screen
// turned in depth, by how far past the box the screen sits; the background
// lies beyond the screen and takes the whole blur. A wider blur reads a lower
// level of the chain, so however wide it grows it costs the same disc of
// samples; two adjacent levels are mixed for a smooth arrival.

struct FocusRegion {
  // The box's top-left and bottom-right corners, in source pixels.
  box: vec4<f32>,
  // The corner radius in source pixels, the strength from one to five, how
  // present the box is this frame, and the inverse turn's weight at the
  // box's middle, which the depth term measures from.
  radius: f32,
  strength: f32,
  presence: f32,
  reference: f32,
}

struct FocusConstants {
  // The canvas's size, then the source's.
  output_source: vec4<f32>,
  // Where the upright screen's picture and its visible box sit, in canvas
  // pixels.
  image_rect: vec4<f32>,
  crop_rect: vec4<f32>,
  // The target's corner and how many canvas pixels one target pixel covers.
  placement: vec4<f32>,
  // The inverse turn's rows, which carry a canvas pixel back to the point of
  // the upright screen it shows; the identity on an upright screen.
  inverse: array<vec4<f32>, 3>,
  // How far past a box the blur takes to arrive, the widest blur the middle
  // strength reaches, how wide the depth term reaches, all in canvas pixels,
  // and how many boxes there are.
  shape: vec4<f32>,
  // How far past the screen's edge the background takes to reach the whole
  // blur, in canvas pixels; the canvas's corner radius, which the frame is
  // clipped to after the blur; whether the canvas is opaque to its edge, a
  // whole canvas rather than a layer laid over others; and how opaque what
  // lies past its edge is laid, sharp as the canvas pass drew it, if at all.
  edge: vec4<f32>,
}

@group(0) @binding(0) var chain: texture_2d<f32>;
@group(0) @binding(1) var linear: sampler;
@group(0) @binding(2) var<uniform> constants: FocusConstants;
@group(0) @binding(3) var<storage, read> regions: array<FocusRegion>;

// How many taps the blur disc reads. A Vogel disc spreads them evenly enough
// that this few reads as smooth rather than as copies of the outline.
const TAPS: u32 = 16u;

// How far past a rounded box a point lies: negative inside it.
fn box_distance(point: vec2<f32>, region: FocusRegion) -> f32 {
  let half_size = (region.box.zw - region.box.xy) * 0.5;
  let q = abs(point - (region.box.xy + region.box.zw) * 0.5) - half_size + region.radius;
  return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - region.radius;
}

// How wide the blur is at canvas pixel `pixel`, in canvas pixels: the least
// any box showing asks for, so a pixel any box keeps sharp stays sharp.
fn circle_of_confusion(pixel: vec2<f32>) -> f32 {
  let at = vec3<f32>(pixel, 1.0);
  let h = vec3<f32>(dot(constants.inverse[0].xyz, at), dot(constants.inverse[1].xyz, at),
                    dot(constants.inverse[2].xyz, at));
  let ahead = h.z > 1e-6;
  let point = select(vec2<f32>(-1e9), h.xy / h.z, ahead);
  // The background lies beyond the screen, so it takes the whole blur; but
  // it arrives at it from the blur the screen has at its edge, as it would
  // through a lens. Jumping straight there would leave the screen's edge a
  // seam where the blur's width changes, which reads as a hard line.
  let crop = constants.crop_rect;
  let outside = length(max(max(crop.xy - point, point - (crop.xy + crop.zw)), vec2<f32>(0.0)));
  let past = clamp(outside / max(constants.edge.x, 1.0), 0.0, 1.0);
  let beyond = select(1.0, past * past * (3.0 - 2.0 * past), ahead);
  let per = constants.output_source.zw / max(constants.image_rect.zw, vec2<f32>(1.0));
  let source_point = (point - constants.image_rect.xy) * per;
  let reach = max(constants.shape.x, 1.0);
  let widest = constants.shape.y;
  var coc = widest * 4.0;
  let count = u32(constants.shape.w);
  for (var index = 0u; index < count; index++) {
    let region = regions[index];
    // Measured on the upright screen, so the box stays where its grips are
    // however the screen is turned, then carried into canvas pixels.
    let distance = box_distance(source_point, region) / max(per.x, 1e-6);
    let fall = clamp(distance / reach, 0.0, 1.0);
    let flat = fall * fall * (3.0 - 2.0 * fall);
    // The inverse turn's weight goes as the reciprocal of a point's depth,
    // so its change from the box's middle is how far the screen there sits
    // in front of or behind the box. An upright screen has none.
    let depth = step(0.0, distance) *
        clamp(abs(h.z - region.reference) / max(abs(region.reference), 1e-3) *
              constants.shape.z / max(widest, 1e-3), 0.0, 1.0);
    let amount = max(max(flat, depth), beyond);
    coc = min(coc, widest * (region.strength / 3.0) * region.presence * amount);
  }
  return select(0.0, coc, count > 0u);
}

// How much of the canvas's own rounded frame covers canvas pixel `pixel`, as
// the canvas pass clips it: the twin of `rounded_coverage` in `preview.wgsl`.
fn frame_coverage(pixel: vec2<f32>) -> f32 {
  let radius = constants.edge.y;
  let half_size = constants.output_source.xy * 0.5;
  let local = abs(pixel - half_size) - (half_size - radius);
  let distance = length(max(local, vec2<f32>(0.0))) + min(max(local.x, local.y), 0.0) - radius;
  if (radius <= 0.0) {
    return select(0.0, 1.0, distance < 0.0);
  }
  return 1.0 - smoothstep(-0.75, 0.75, distance);
}

@vertex
fn vs_main(@builtin(vertex_index) id: u32) -> @builtin(position) vec4<f32> {
  let corner = vec2<f32>(f32((id << 1u) & 2u), f32(id & 2u));
  return vec4<f32>(corner * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
  let size = vec2<f32>(textureDimensions(chain));
  let scale = max(constants.placement.z, 1e-6);
  let pixel = (position.xy - constants.placement.xy) * constants.placement.zw;
  let coverage = frame_coverage(pixel);
  // Nothing past the canvas's edge is exported, so the focus leaves it be,
  // faded as the canvas pass fades it.
  let sharp = textureLoad(chain, vec2<i32>(position.xy), 0) * constants.edge.w;
  if (coverage <= 0.0) {
    return sharp;
  }
  // The blur in target pixels, which the chain is laid in.
  let radius = circle_of_confusion(pixel) / scale;
  let top = f32(textureNumLevels(chain) - 1u);
  let level = clamp(log2(max(radius, 1.0)), 0.0, top);
  let lower = floor(level);
  let rise = level - lower;
  // Taps stay on the canvas, so its edge never draws in what lies past it.
  let low = max(constants.placement.xy, vec2<f32>(0.0)) + 0.5;
  let high = min(constants.placement.xy + constants.output_source.xy / scale, size) - 0.5;
  let golden = 2.39996323;
  var sum = vec4<f32>(0.0);
  for (var tap = 0u; tap < TAPS; tap++) {
    let k = f32(tap) + 0.5;
    let angle = k * golden;
    let offset = vec2<f32>(cos(angle), sin(angle)) * (sqrt(k / f32(TAPS)) * radius);
    let uv = clamp(position.xy + offset, low, max(low, high)) / size;
    let nearer = textureSampleLevel(chain, linear, uv, lower);
    let further = textureSampleLevel(chain, linear, uv, min(lower + 1.0, top));
    sum += mix(nearer, further, rise);
  }
  var blurred = sum / f32(TAPS);
  // A whole canvas is opaque to its edge. Where a coarse level averages in
  // what lies past the edge, its colour is read back at full strength, so
  // the edge neither darkens nor thins; then the frame is clipped, sharp.
  if (constants.edge.z != 0.0 && blurred.a > 1e-4) {
    blurred = vec4<f32>(blurred.rgb / blurred.a, 1.0);
  }
  return blurred * coverage + sharp * (1.0 - coverage);
}
