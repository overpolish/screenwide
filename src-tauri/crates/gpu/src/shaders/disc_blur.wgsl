// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The focus blur's disc, for any module that joins this file: what lies round
// `position`, in texels of the first level of `levels`, softened `radius`
// texels wide. A wider blur reads a lower level of the mip chain, so however
// wide it grows it costs the same disc of samples; two adjacent levels are
// mixed for a smooth arrival. Taps stay between `low` and `high`, in texels
// of the first level, so an edge never draws in what lies past it.

// How many taps the disc reads. A Vogel disc spreads them evenly enough that
// this few reads as smooth rather than as copies of the outline.
const DISC_BLUR_TAPS: u32 = 16u;

fn disc_blur(levels: texture_2d<f32>, blur_sampler: sampler, position: vec2<f32>, radius: f32,
             low: vec2<f32>, high: vec2<f32>) -> vec4<f32> {
  let size = vec2<f32>(textureDimensions(levels));
  let top = f32(textureNumLevels(levels) - 1u);
  let level = clamp(log2(max(radius, 1.0)), 0.0, top);
  let lower = floor(level);
  let rise = level - lower;
  let golden = 2.39996323;
  var sum = vec4<f32>(0.0);
  for (var tap = 0u; tap < DISC_BLUR_TAPS; tap++) {
    let k = f32(tap) + 0.5;
    let angle = k * golden;
    let offset = vec2<f32>(cos(angle), sin(angle)) * (sqrt(k / f32(DISC_BLUR_TAPS)) * radius);
    let uv = clamp(position + offset, low, high) / size;
    let nearer = textureSampleLevel(levels, blur_sampler, uv, lower);
    let further = textureSampleLevel(levels, blur_sampler, uv, min(lower + 1.0, top));
    sum += mix(nearer, further, rise);
  }
  return sum / f32(DISC_BLUR_TAPS);
}
