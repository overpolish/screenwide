// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The shade a layer's spotlights cast, and the cursor as the layers see it.
// Every spotlight showing cuts its hole in one shared shade, which darkens
// what is under it by a fixed share; `composite_annotation_layers` lays it
// where the topmost spotlight sits, over the picture and the marks below,
// and darkens the cursor by it too. The arithmetic is Rust's
// `spotlight::geometry::{light, shade}`.
//
// The cursor lies on the picture: the spotlights' blur is applied to the
// source before the cursor is drawn, so the cursor is softened here by the
// same deviation and the same share.

const annotation_spotlight_kind: u32 = 6u;
// How much of the light the shade takes away. The twin of Rust's
// `spotlight::model::SPOTLIGHT_DIM`.
const annotation_spotlight_dim: f32 = 0.4;
// The flag a spotlight that blurs carries: `flags::BLUR`.
const annotation_spotlight_blur_flag: u32 = 1u << 3u;

// How much of one spotlight's light reaches `probe`: all of it well inside
// the box, none outside it, and a smooth fall over its softness in from the
// edge. `pixel` is one drawn pixel, which is all a spotlight with no softness
// fades over.
fn annotation_spotlight_light(shape: PreviewGeometry, probe: vec2<f32>, pixel: f32) -> f32 {
  let low = vec2<f32>(shape.ax, shape.ay);
  let high = vec2<f32>(shape.bx, shape.by);
  let rounding = min(shape.rounding, min(high.x - low.x, high.y - low.y) * 0.5);
  let q = abs(probe - (low + high) * 0.5) - ((high - low) * 0.5 - rounding);
  let distance = length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - rounding;
  let soft = max(shape.width, 0.0);
  let fall = saturate((distance + soft + pixel * 0.5) / (soft + pixel));
  return 1.0 - fall * fall * (3.0 - 2.0 * fall);
}

// How much of a spotlight layer reaches `probe`, from the spotlights between
// `first` and `last`: as strong as the most present spotlight that counts -
// every one for the shade, those that blur for the blur - lifted by each
// one's light as far as it is present. A spotlight's presence rides in its
// colour's alpha, which the preparation scales by its reveal. `feather` is
// half a drawn pixel, as every annotation layer is given it. The editor's
// passes blur the source with the redactions; only the live overlay, which
// has no source, asks for the blur here.
fn annotation_spotlight_cover(probe: vec2<f32>, first: u32, last: u32, feather: f32,
                              blurring: bool) -> f32 {
  let pixel = max(2.0 * feather, 1e-4);
  var layer = 0.0;
  var lit = 0.0;
  for (var index = annotation_next(probe, first, last); index < last;
       index = annotation_next(probe, index + 1u, last)) {
    let annotation = annotation_arrows[index];
    if (annotation.kind != annotation_spotlight_kind) {
      continue;
    }
    let presence = saturate(annotation.alpha);
    if (presence <= 0.0) {
      continue;
    }
    if (!blurring || (annotation.flags & annotation_spotlight_blur_flag) != 0u) {
      layer = max(layer, presence);
    }
    let shape = annotation.geometry;
    // Only a point inside the box can be lit; everywhere else is shade.
    let reach = pixel + 1.0;
    if (probe.x < shape.ax - reach || probe.y < shape.ay - reach ||
        probe.x > shape.bx + reach || probe.y > shape.by + reach) {
      continue;
    }
    lit = max(lit, presence * annotation_spotlight_light(shape, probe, pixel));
  }
  return max(layer - lit, 0.0);
}

// The spotlights' blur as the canvas draws it: its standard deviation in
// canvas pixels and how far it has arrived - as present as the most present
// spotlight that blurs - both zero where none showing blurs; and how many
// prepared annotations the spotlights lifting it are among.
struct AnnotationSpotlightBlur {
  deviation: f32,
  strength: f32,
  count: u32,
}

// How much of the pixel at `probe` the spotlights' blur takes: all of it,
// lifted by each spotlight's light as far as its presence against the
// blur's. The twin of `redact_spotlight_share`, which blurs the source.
fn annotation_spotlight_blur_share(probe: vec2<f32>, blur: AnnotationSpotlightBlur,
                                   pixel: f32) -> f32 {
  if (blur.strength <= 0.0) {
    return 0.0;
  }
  var lit = 0.0;
  for (var index = annotation_next(probe, 0u, blur.count); index < blur.count;
       index = annotation_next(probe, index + 1u, blur.count)) {
    let annotation = annotation_arrows[index];
    if (annotation.kind != annotation_spotlight_kind) {
      continue;
    }
    let presence = min(saturate(annotation.alpha) / blur.strength, 1.0);
    if (presence <= 0.0) {
      continue;
    }
    let shape = annotation.geometry;
    let reach = pixel + 1.0;
    if (probe.x < shape.ax - reach || probe.y < shape.ay - reach ||
        probe.x > shape.bx + reach || probe.y > shape.by + reach) {
      continue;
    }
    lit = max(lit, presence * annotation_spotlight_light(shape, probe, pixel));
  }
  return saturate(1.0 - lit);
}

// The cursor at `probe`, straight alpha, as the layer shows it: sharp, or
// softened by the spotlights' blur where that reaches. The soft cursor is the
// cursor alone blurred as a Gaussian into a layer by
// `compositor/mark_blur.rs`, which `annotation_cursor_blurred` reads; a pass
// with no such layer draws the cursor sharp. `pixel` is one drawn pixel,
// which a spotlight's hard edge fades over.
fn annotation_cursor_seen(probe: vec2<f32>, pixel: f32) -> vec4<f32> {
  let blur = annotation_spotlight_blur();
  var share = 0.0;
  if (annotation_cursor_blur_layer() && blur.deviation >= 0.5 && blur.strength > 0.0) {
    share = annotation_spotlight_blur_share(probe, blur, pixel);
  }
  var seen = vec4<f32>(0.0);
  if (share < 1.0 && annotation_cursor_near(probe, 1.0)) {
    let sharp = annotation_cursor_sample(probe);
    seen = vec4<f32>(sharp.rgb * sharp.a, sharp.a);
  }
  if (share > 0.0) {
    seen = mix(seen, annotation_cursor_blurred(probe), share);
  }
  if (seen.a <= 0.0) {
    return vec4<f32>(0.0);
  }
  return vec4<f32>(seen.rgb / seen.a, seen.a);
}
