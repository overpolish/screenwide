// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The paint pass: one pixel of the box, drawn over a copy of what the last
// pass left. The twin of `redact_source_rgba`.

// One cell's averaged colour, the nearest cell standing in past the box's
// edge.
fn redact_cell(cell: vec2<i32>) -> vec3<f32> {
  let last = vec2<i32>(redact.grid) - 1;
  return textureLoad(redact_cells, clamp(cell, vec2<i32>(0), last), 0).rgb;
}

// The blur's second pass at the box's pixel `gid`: the Gaussian down its
// column of what the rows pass wrote, within the box, back out of
// premultiplied colour.
fn redact_blur(gid: vec2<u32>) -> vec3<f32> {
  let tall = i32(redact.bounds.w - redact.bounds.y);
  let reach = redact_reach(redact.size);
  var sum = vec4<f32>(0.0);
  var total = 0.0;
  let row = i32(gid.y);
  let last = min(row + reach, tall - 1);
  for (var y = max(row - reach, 0); y <= last; y++) {
    let weight = redact_weight(y - row, redact.size);
    sum += weight * textureLoad(redact_rows, vec2<i32>(i32(gid.x), y), 0);
    total += weight;
  }
  let average = sum / total;
  if (average.a <= 0.0) {
    return vec3<f32>(0.0);
  }
  return average.rgb / average.a;
}

// How much of the pixel centred at `local` the box takes: all of every pixel
// its rounded outline touches, since a pixel's farthest point is half its
// diagonal from its centre, then a soft edge one pixel wide beyond it. So
// only a pixel wholly outside the outline is ever blended, and what shows
// through it is picture the box never covered.
fn redact_cover(local: vec2<f32>, size: vec2<f32>) -> f32 {
  if (redact.radius <= 0.0) {
    return 1.0;
  }
  let q = abs(local - size * 0.5) - (size * 0.5 - redact.radius);
  let distance = length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - redact.radius;
  return saturate(1.70710678 - distance);
}

// The colour the box's pixel `gid` is painted with.
fn redact_colour(gid: vec2<u32>) -> vec3<f32> {
  if (redact.mode == 1u) {
    return redact_block(vec2<u32>(vec2<f32>(gid) / max(redact.size, 1.0)));
  }
  if (redact.mode == 2u || redact.mode == 4u) {
    return redact_blur(gid);
  }
  if (redact.mode == 3u) {
    return redact_cell(vec2<i32>(redact_cell_of(gid)));
  }
  return redact.color.rgb;
}

// How much of the pixel centred at `probe` the spotlights' blur takes: as
// much as the record's cover, which rides in the colour's alpha, lifted by
// each spotlight's light as far as it is present. The zones are the holes,
// four each: the box's corners, its rounding and fade, and its presence
// beside the blur's, in source pixels. The twin of `redact_spotlight_share`.
fn redact_spotlight_share(probe: vec2<f32>) -> f32 {
  var lit = 0.0;
  for (var at = 0u; at + 3u < redact.entry_count; at += 4u) {
    let first = redact.zone_first + at;
    let low = redact_zones[first];
    let high = redact_zones[first + 1u];
    let shape = redact_zones[first + 2u];
    let presence = saturate(redact_zones[first + 3u].x);
    let rounding = min(shape.x, min(high.x - low.x, high.y - low.y) * 0.5);
    let q = abs(probe - (low + high) * 0.5) - ((high - low) * 0.5 - rounding);
    let distance = length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - rounding;
    let soft = max(shape.y, 0.0);
    let fall = saturate((distance + soft + 0.5) / (soft + 1.0));
    lit = max(lit, presence * (1.0 - fall * fall * (3.0 - 2.0 * fall)));
  }
  return max(saturate(redact.color.a) - lit, 0.0);
}

// The viewport is the box, so every pixel drawn is one of its own. A pixel
// the box does not take is discarded, which leaves the copy's own there.
@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
  let pixel = vec2<u32>(position.xy);
  if (any(pixel < redact.bounds.xy) || any(pixel >= redact.bounds.zw)) {
    discard;
  }
  let gid = pixel - redact.bounds.xy;
  // Its rounded outline's cover, and of that only the share an arriving fill
  // has reached, which rides in the colour's alpha. The spotlights' blur
  // takes what its holes leave.
  var cover: f32;
  if (redact.mode == 4u) {
    cover = redact_spotlight_share(vec2<f32>(pixel) + 0.5);
  } else {
    cover = redact_cover(vec2<f32>(gid) + 0.5, vec2<f32>(redact.bounds.zw - redact.bounds.xy)) *
        saturate(redact.color.a);
  }
  if (cover <= 0.0) {
    discard;
  }
  let original = textureLoad(redact_source, vec2<i32>(pixel), 0);
  // What the spotlights' blur softens keeps its own alpha: it is the same
  // picture, only less sharp.
  var painted = vec4<f32>(round(saturate(redact_colour(gid)) * 255.0) / 255.0,
                          select(1.0, original.a, redact.mode == 4u));
  if (cover < 1.0) {
    painted = mix(original, painted, cover);
  }
  return painted;
}
