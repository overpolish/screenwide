// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The sticker: a picture laid over the canvas, turned, sized, mirrored and
// rounded, with the shadow it may cast. Every number it reads was prepared by
// `sticker::geometry::prepare_sticker`, which documents where the record
// keeps each part; where its picture sits in the sticker atlas rides in
// `start_head`, and its shadow's in `end_head`, as `sticker_artwork` wrote
// them. The atlas holds premultiplied colour, so the picture is laid over the
// canvas as it is, not tinted.

const annotation_sticker_kind: u32 = 9u;
// The flag a sticker that casts a shadow carries: `flags::SHADOW`.
const annotation_sticker_shadow_flag: u32 = 1u << 7u;
// The halo a hovered sticker wears: the system blue, since a sticker has no
// colour of its own to wear one in.
const annotation_sticker_halo_color: vec4<f32> = vec4<f32>(0.0, 0.48, 1.0, 1.0);
// What a sticker whose picture cannot be found shows instead: a quiet grey
// card, so the annotation can still be found, chosen and given a picture.
const annotation_sticker_placeholder: vec4<f32> = vec4<f32>(0.5, 0.5, 0.5, 0.35);
// How far a shadow spreads, as a share of half the picture's shorter side,
// and how far below the picture it falls, as a share of that spread: the
// magnifier's own. The twin of `SHADOW_SPREAD` in `stickers/shadow.rs`.
const annotation_sticker_spread: f32 = 0.11;
const annotation_sticker_lift: f32 = 0.35;
// How dark a shadow is where its picture is solid: twice the magnifier's,
// so it is as dark as the loupe's along the picture's edge, where the
// blurred silhouette is half.
const annotation_sticker_shadow_strength: f32 = 0.28;

/// The stickers' pictures, each drawn at about the size it is shown.
@group(0) @binding(17) var annotation_sticker_pictures: texture_2d<f32>;

/// A sticker read out of its prepared record: its middle, its picture's own
/// right and down as unit vectors, its half width and height, its corner
/// radius, where its picture and its shadow sit in the atlas, how far in from
/// the shadow's cell its picture's box starts, and whether it is mirrored.
struct PreviewSticker {
  center: vec2<f32>,
  across: vec2<f32>,
  down: vec2<f32>,
  half_size: vec2<f32>,
  rounding: f32,
  cell_origin: vec2<f32>,
  cell_size: vec2<f32>,
  shadow_origin: vec2<f32>,
  shadow_size: vec2<f32>,
  shadow_inset: f32,
  mirrored: bool,
}

fn annotation_sticker(prepared: PreviewGeometry) -> PreviewSticker {
  var sticker: PreviewSticker;
  sticker.center = vec2<f32>(prepared.ax, prepared.ay);
  let across = vec2<f32>(prepared.bx, prepared.by) - sticker.center;
  let down = vec2<f32>(prepared.cx, prepared.cy) - sticker.center;
  sticker.half_size = vec2<f32>(length(across), length(down));
  sticker.across = across / max(sticker.half_size.x, 1e-6);
  sticker.down = down / max(sticker.half_size.y, 1e-6);
  sticker.rounding = prepared.rounding;
  sticker.cell_origin = vec2<f32>(prepared.start_head_ax, prepared.start_head_ay);
  sticker.cell_size = vec2<f32>(prepared.start_head_bx, prepared.start_head_by);
  sticker.shadow_origin = vec2<f32>(prepared.end_head_ax, prepared.end_head_ay);
  sticker.shadow_size = vec2<f32>(prepared.end_head_bx, prepared.end_head_by);
  sticker.shadow_inset = prepared.end_head_cx;
  sticker.mirrored = prepared.head != 0u;
  return sticker;
}

/// `probe` in the picture's own frame: across and down from its middle.
fn annotation_sticker_local(probe: vec2<f32>, sticker: PreviewSticker) -> vec2<f32> {
  let offset = probe - sticker.center;
  return vec2<f32>(dot(offset, sticker.across), dot(offset, sticker.down));
}

/// How far `probe` falls outside the picture's rounded box: negative inside.
/// The per-pixel twin of `sticker::geometry::sticker_distance`.
fn annotation_sticker_distance(probe: vec2<f32>, sticker: PreviewSticker) -> f32 {
  let local = annotation_sticker_local(probe, sticker);
  let rounding = min(sticker.rounding, min(sticker.half_size.x, sticker.half_size.y));
  let q = abs(local) - (sticker.half_size - rounding);
  return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - rounding;
}

/// The atlas's colour at `texel`, bilinearly filtered and held inside the
/// cell from `low` to `high`, so a read at the picture's edge takes its own
/// transparent margin rather than the next cell over.
fn annotation_sticker_bilinear(texel: vec2<f32>, low: vec2<f32>, high: vec2<f32>) -> vec4<f32> {
  let base = texel - 0.5;
  let first = floor(base);
  let blend = base - first;
  let near = vec2<i32>(clamp(first, low, high));
  let far = vec2<i32>(clamp(first + 1.0, low, high));
  let top = mix(textureLoad(annotation_sticker_pictures, near, 0),
                textureLoad(annotation_sticker_pictures, vec2<i32>(far.x, near.y), 0), blend.x);
  let bottom = mix(textureLoad(annotation_sticker_pictures, vec2<i32>(near.x, far.y), 0),
                   textureLoad(annotation_sticker_pictures, far, 0), blend.x);
  return mix(top, bottom, blend.y);
}

/// Where in the atlas the picture's point `local` is drawn: its inside runs
/// from one pixel in from the cell's corner to one pixel in from the far one.
fn annotation_sticker_texel(local: vec2<f32>, sticker: PreviewSticker) -> vec2<f32> {
  var share = local / max(sticker.half_size, vec2<f32>(1e-6)) * 0.5 + 0.5;
  if (sticker.mirrored) {
    share.x = 1.0 - share.x;
  }
  return sticker.cell_origin + 1.0 + share * (sticker.cell_size - 2.0);
}

/// The picture's premultiplied colour over one drawn pixel at `probe`, spread
/// `feather` canvas pixels either way: four filtered taps over the one or two
/// atlas pixels the drawn pixel spans. Clear outside the picture.
fn annotation_sticker_colour(probe: vec2<f32>, sticker: PreviewSticker,
                             feather: f32) -> vec4<f32> {
  if (sticker.cell_size.x <= 2.0 || sticker.cell_size.y <= 2.0) {
    return annotation_sticker_placeholder * vec4<f32>(annotation_sticker_placeholder.aaa, 1.0);
  }
  let atlas = vec2<f32>(textureDimensions(annotation_sticker_pictures));
  let low = sticker.cell_origin;
  let high = min(sticker.cell_origin + sticker.cell_size, atlas) - 1.0;
  let per_canvas = (sticker.cell_size.x - 2.0) / max(2.0 * sticker.half_size.x, 1e-6);
  let spread = 2.0 * feather * per_canvas * 0.25;
  let local = annotation_sticker_local(probe, sticker);
  let texel = annotation_sticker_texel(local, sticker);
  var total = vec4<f32>(0.0);
  for (var tap = 0u; tap < 4u; tap++) {
    let offset = vec2<f32>(select(-spread, spread, (tap & 1u) != 0u),
                           select(-spread, spread, (tap >> 1u) != 0u));
    total += annotation_sticker_bilinear(texel + offset, low, high);
  }
  return total * 0.25;
}

/// How far a sticker's shadow spreads, in canvas pixels.
fn annotation_sticker_spread_of(sticker: PreviewSticker) -> f32 {
  return annotation_sticker_spread * min(sticker.half_size.x, sticker.half_size.y);
}

/// How much a sticker's shadow darkens what is under it at `probe`: its
/// silhouette, blurred in the atlas, lowered by its lift and turned and
/// mirrored with its picture.
fn annotation_sticker_shadow(probe: vec2<f32>, sticker: PreviewSticker) -> f32 {
  if (sticker.shadow_size.x <= 0.0 || sticker.shadow_size.y <= 0.0) {
    return 0.0;
  }
  let spread = annotation_sticker_spread_of(sticker);
  let lowered = probe - vec2<f32>(0.0, spread * annotation_sticker_lift);
  var share = annotation_sticker_local(lowered, sticker) /
      max(sticker.half_size, vec2<f32>(1e-6)) * 0.5 + 0.5;
  if (sticker.mirrored) {
    share.x = 1.0 - share.x;
  }
  let inner = sticker.shadow_size - 2.0 * sticker.shadow_inset;
  let texel = sticker.shadow_origin + sticker.shadow_inset + share * inner;
  let atlas = vec2<f32>(textureDimensions(annotation_sticker_pictures));
  let low = sticker.shadow_origin;
  let high = min(sticker.shadow_origin + sticker.shadow_size, atlas) - 1.0;
  if (any(texel < low) || any(texel > high + 1.0)) {
    return 0.0;
  }
  return annotation_sticker_shadow_strength * annotation_sticker_bilinear(texel, low, high).a;
}

/// Where a sticker can reach: its turned picture, at its largest across the
/// exposure, and the shadow and halo round it.
fn annotation_sticker_bounds(annotation: PreviewArrow, halo: f32, feather: f32) -> vec4<f32> {
  var sticker = annotation_sticker(annotation.geometry);
  if (annotation.sample_count > 0u) {
    let last = annotation.sample_first + annotation.sample_count - 1u;
    let first = annotation_sticker(annotation_samples[annotation.sample_first].geometry);
    let latest = annotation_sticker(annotation_samples[last].geometry);
    sticker.half_size = max(sticker.half_size, max(first.half_size, latest.half_size));
  }
  let reach_x = abs(sticker.across.x) * sticker.half_size.x + abs(sticker.down.x) * sticker.half_size.y;
  let reach_y = abs(sticker.across.y) * sticker.half_size.x + abs(sticker.down.y) * sticker.half_size.y;
  var spill = halo + feather + 1.0;
  if ((annotation.flags & annotation_sticker_shadow_flag) != 0u) {
    spill += annotation_sticker_spread_of(sticker) * (3.0 + annotation_sticker_lift);
  }
  let reach = vec2<f32>(reach_x, reach_y) + spill;
  return vec4<f32>(sticker.center - reach, sticker.center + reach);
}

/// One sticker over `rgba_in`, premultiplied: its shadow, then its picture,
/// cut to its rounded box. A still frame draws the prepared sticker, its
/// presence already folded into its colour's alpha; a moving one averages
/// the sticker over the exposure, each sample at the presence it had.
fn annotation_sticker_layer(rgba_in: vec4<f32>, annotation: PreviewArrow, canvas_point: vec2<f32>,
                            feather: f32) -> vec4<f32> {
  let halo = max(annotation.hover, 0.0);
  if (annotation_outside(canvas_point, annotation_sticker_bounds(annotation, halo, feather))) {
    return rgba_in;
  }
  let shadowed = (annotation.flags & annotation_sticker_shadow_flag) != 0u;
  let taps = max(annotation.sample_count, 1u);
  var picture = vec4<f32>(0.0);
  var shadow = 0.0;
  for (var tap = 0u; tap < taps; tap++) {
    var geometry = annotation.geometry;
    var presence = annotation.alpha;
    if (annotation.sample_count > 0u) {
      let sample = annotation_samples[annotation.sample_first + tap];
      geometry = sample.geometry;
      presence = sample.opacity;
    }
    // Every sample reads the picture and its shadow from where the frame's
    // own record says they were drawn.
    geometry.start_head_ax = annotation.geometry.start_head_ax;
    geometry.start_head_ay = annotation.geometry.start_head_ay;
    geometry.start_head_bx = annotation.geometry.start_head_bx;
    geometry.start_head_by = annotation.geometry.start_head_by;
    geometry.end_head_ax = annotation.geometry.end_head_ax;
    geometry.end_head_ay = annotation.geometry.end_head_ay;
    geometry.end_head_bx = annotation.geometry.end_head_bx;
    geometry.end_head_by = annotation.geometry.end_head_by;
    geometry.end_head_cx = annotation.geometry.end_head_cx;
    let sticker = annotation_sticker(geometry);
    if (shadowed) {
      shadow += annotation_sticker_shadow(canvas_point, sticker) * presence;
    }
    let inside = annotation_edge(annotation_sticker_distance(canvas_point, sticker), feather);
    if (inside > 0.0) {
      picture += annotation_sticker_colour(canvas_point, sticker, feather) * inside * presence;
    }
  }
  picture /= f32(taps);
  shadow /= f32(taps);
  var rgba = rgba_in;
  if (halo > 0.0) {
    let sticker = annotation_sticker(annotation.geometry);
    rgba = annotation_halo(rgba, annotation_sticker_halo_color,
                           annotation_sticker_distance(canvas_point, sticker), halo, feather);
  }
  // The shadow darkens what lies under it, as the magnifier's does, and the
  // picture covers it.
  if (shadow > 0.0) {
    rgba = vec4<f32>(rgba.rgb * (1.0 - shadow), rgba.a);
  }
  return picture + rgba * (1.0 - picture.a);
}
