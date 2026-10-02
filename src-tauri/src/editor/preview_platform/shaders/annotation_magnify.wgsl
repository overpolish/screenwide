// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The magnifier: a loupe showing a zoom area enlarged, the line joining the
// two, the loupe's rim and its shadow. Every number it reads was prepared by
// `magnify::geometry::prepare_magnify`, which documents where the record
// keeps each part.
//
// The loupe reads the picture through `annotation_magnify_placement` and
// `annotation_magnify_fetch`, which each shader that draws annotations
// defines over its own source: the preview's is its source texture after the
// redactions and the spotlights' blur were applied to it, and the live
// overlay's, which offers no magnifier, the desktop its highlights recolour.
// Highlights read them too, to find each glyph's full ink. Each texel is drawn
// as a crisp square, with only the one drawn pixel across its edge blended.
// The cursor lies on the picture, under the loupe, so the loupe shows it too,
// enlarged, through `annotation_cursor_seen`. In the editor a loupe shows
// every annotation below it as well, enlarged: `annotation_layers.wgsl`
// composes what it shows and hands it to `annotation_magnify_draw`.

const annotation_magnify_kind: u32 = 8u;
// The flag a loupe that casts a shadow carries: `flags::SHADOW`.
const annotation_magnify_shadow_flag: u32 = 1u << 7u;

// Where the source lies on the canvas: the placed image's rectangle, the
// texels the canvas shows of it, first and last inclusive, and its size.
struct AnnotationMagnifyPlacement {
  image: vec4<f32>,
  texels: vec4<f32>,
  size: vec2<f32>,
}

// The picture at canvas point `q`, where one drawn pixel spans `span` canvas
// pixels there.
fn annotation_magnify_sample(q: vec2<f32>, span: vec2<f32>) -> vec4<f32> {
  let at = annotation_magnify_placement();
  if (any(at.image.zw <= vec2<f32>(0.0)) || any(at.size < vec2<f32>(1.0))) {
    return vec4<f32>(0.0);
  }
  let per = at.size / at.image.zw;
  let centred = (q - at.image.xy) * per - 0.5;
  let base = floor(centred);
  // Enlarged, a texel spans many drawn pixels, and only the one across its
  // edge is blended; reduced, this is plain bilinear filtering.
  let footprint = min(max(span * per, vec2<f32>(1e-4)), vec2<f32>(1.0));
  let blend = saturate((centred - base - 0.5) / footprint + 0.5);
  // Past what the canvas shows, the edge the crop left carries on.
  let first = vec2<i32>(clamp(base, at.texels.xy, at.texels.zw));
  let second = vec2<i32>(clamp(base + 1.0, at.texels.xy, at.texels.zw));
  let top = mix(annotation_magnify_fetch(first),
                annotation_magnify_fetch(vec2<i32>(second.x, first.y)), blend.x);
  let bottom = mix(annotation_magnify_fetch(vec2<i32>(first.x, second.y)),
                   annotation_magnify_fetch(second), blend.x);
  return mix(top, bottom, blend.y);
}

// A rounded box's signed distance, by its centre and half size.
fn annotation_magnify_box(probe: vec2<f32>, centre: vec2<f32>, half_size: vec2<f32>,
                          radius: f32) -> f32 {
  let rounding = min(radius, min(half_size.x, half_size.y));
  let q = abs(probe - centre) - (half_size - rounding);
  return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - rounding;
}

fn annotation_magnify_segment(probe: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
  let along = b - a;
  let t = saturate(dot(probe - a, along) / max(dot(along, along), 1e-6));
  return distance(probe, a + along * t);
}

fn annotation_magnify_over(rgba: vec4<f32>, color: vec3<f32>, alpha: f32) -> vec4<f32> {
  if (alpha <= 0.0) {
    return rgba;
  }
  return annotation_over(rgba, color, alpha);
}

// Draws one magnifier, as `shape` places it and in `color`, over `rgba`: the
// loupe's shadow, the zoom area's outline and the line to the loupe in a pen
// half the rim's, `content` in the loupe, and the rim. `content` is what the
// loupe shows at `probe`, premultiplied: what lies under the magnifier at the
// matching point of its zoom area, enlarged, as the caller composed it. It
// is solid throughout: a loupe setting off from its zoom area shows exactly
// what that covers, at its own size, so it lifts off the picture rather than
// fading in over it. The shadow is a clip's, and comes with the magnifier's
// presence; the rest comes with its colour, which carries the presence
// already. The hover halo hugs the zoom area, the box the chrome holds.
fn annotation_magnify_draw(rgba_in: vec4<f32>, shape: PreviewGeometry, color: vec4<f32>,
                           flags: u32, probe: vec2<f32>, feather: f32, halo: f32,
                           content: vec4<f32>) -> vec4<f32> {
  let centre = vec2<f32>(shape.ax, shape.ay);
  let half_size = vec2<f32>(shape.bx, shape.by);
  let area = vec2<f32>(shape.cx, shape.cy);
  let area_half = vec2<f32>(shape.start_head_cx, shape.start_head_cy);
  if (any(half_size <= vec2<f32>(0.0)) || any(area_half <= vec2<f32>(0.0))) {
    return rgba_in;
  }
  var rgba = rgba_in;
  let presence = saturate(shape.low);
  let stroke = max(shape.width, 0.0);
  let pen = max(stroke * 0.5, feather * 2.0);
  let shadowed = (flags & annotation_magnify_shadow_flag) != 0u;
  // A clip's own shadow: the same spread for its size, lifted the same way.
  let sigma = clamp(min(half_size.x, half_size.y) * 0.11, 2.0, 110.0);
  let lift = sigma * 0.35;
  let reach = stroke + feather * 2.0 + select(0.0, sigma * 4.0 + lift, shadowed);
  let near_loupe = all(abs(probe - centre) <= half_size + reach);
  let near_area = all(abs(probe - area) <= area_half + pen + halo + feather * 2.0);
  let line_start = vec2<f32>(shape.start_head_ax, shape.start_head_ay);
  let line_end = vec2<f32>(shape.start_head_bx, shape.start_head_by);
  let near_line = shape.head != 0u &&
      all(probe >= min(line_start, line_end) - pen - feather * 2.0) &&
      all(probe <= max(line_start, line_end) + pen + feather * 2.0);
  if (!near_loupe && !near_area && !near_line) {
    return rgba;
  }
  let loupe = annotation_magnify_box(probe, centre, half_size, shape.rounding);
  let inside = annotation_edge(loupe, feather);
  if (shadowed && near_loupe) {
    let fall = max(annotation_magnify_box(probe - vec2<f32>(0.0, lift), centre, half_size,
                                          shape.rounding), 0.0);
    let shadow = 0.14 * presence * exp(-0.5 * fall * fall / (sigma * sigma));
    rgba = vec4<f32>(rgba.rgb * (1.0 - shadow * (1.0 - inside)), rgba.a);
  }
  var outline = 1e20;
  if (near_area) {
    outline = annotation_magnify_box(probe, area, area_half, shape.high);
  }
  if (halo > 0.0 && near_area) {
    let outer = outline - pen * 0.5;
    let band = (1.0 - annotation_edge(outer, feather)) * annotation_edge(outer - halo, feather);
    rgba = annotation_magnify_over(rgba, color.rgb, band * color.a * annotation_hover_alpha);
  }
  var lines = abs(outline) - pen * 0.5;
  if (near_line) {
    lines = min(lines, annotation_magnify_segment(probe, line_start, line_end) - pen * 0.5);
  }
  rgba = annotation_magnify_over(rgba, color.rgb, annotation_edge(lines, feather) * color.a);
  if (inside > 0.0) {
    rgba = content * inside + rgba * (1.0 - inside * content.a);
  }
  if (stroke > 0.0 && near_loupe) {
    rgba = annotation_magnify_over(rgba, color.rgb,
                                   annotation_edge(abs(loupe) - stroke * 0.5, feather) * color.a);
  }
  return rgba;
}

// How many canvas pixels of the zoom area one loupe pixel shows, on each
// axis.
fn annotation_magnify_shrink(shape: PreviewGeometry) -> vec2<f32> {
  return vec2<f32>(shape.start_head_cx, shape.start_head_cy) / vec2<f32>(shape.bx, shape.by);
}

// The point of the zoom area the loupe shows at `probe`.
fn annotation_magnify_seen_at(shape: PreviewGeometry, probe: vec2<f32>) -> vec2<f32> {
  return vec2<f32>(shape.cx, shape.cy) +
      (probe - vec2<f32>(shape.ax, shape.ay)) * annotation_magnify_shrink(shape);
}

// What a loupe shows at `probe` of the picture alone, premultiplied, and of
// the cursor lying on it where `cursor` says the layer carries it; nothing
// outside the loupe.
fn annotation_magnify_picture(shape: PreviewGeometry, probe: vec2<f32>, feather: f32,
                              cursor: bool) -> vec4<f32> {
  if (annotation_magnify_inside(shape, probe, feather) <= 0.0) {
    return vec4<f32>(0.0);
  }
  let shrink = annotation_magnify_shrink(shape);
  let seen_at = annotation_magnify_seen_at(shape, probe);
  let picture = annotation_magnify_sample(seen_at, shrink * feather * 2.0);
  var content = vec4<f32>(picture.rgb * picture.a, picture.a);
  if (cursor) {
    let pointer = annotation_cursor_seen(seen_at, feather * 2.0 * min(shrink.x, shrink.y));
    content = annotation_over(content, pointer.rgb, pointer.a);
  }
  return content;
}

// Draws one magnifier over `rgba`, its loupe showing the picture alone. One
// that moved while the shutter was open is drawn whole at every exposure
// sample, each at the opacity it had then, and the results averaged: what an
// open shutter records of a loupe flying out of its zoom area, picture and
// all.
fn annotation_magnify_layer(rgba: vec4<f32>, magnifier: PreviewArrow, probe: vec2<f32>,
                            feather: f32, halo: f32, cursor: bool) -> vec4<f32> {
  let color = annotation_color(magnifier);
  if (magnifier.sample_count == 0u) {
    return annotation_magnify_draw(rgba, magnifier.geometry, color, magnifier.flags, probe,
                                   feather, halo,
                                   annotation_magnify_picture(magnifier.geometry, probe, feather,
                                                              cursor));
  }
  var total = vec4<f32>(0.0);
  for (var tap = 0u; tap < magnifier.sample_count; tap++) {
    let sample = annotation_samples[magnifier.sample_first + tap];
    total += annotation_magnify_draw(rgba, sample.geometry,
                                     vec4<f32>(color.rgb, color.a * sample.opacity),
                                     magnifier.flags, probe, feather, halo,
                                     annotation_magnify_picture(sample.geometry, probe, feather,
                                                                cursor));
  }
  return total / f32(magnifier.sample_count);
}

// How much of `probe` one prepared loupe covers.
fn annotation_magnify_inside(shape: PreviewGeometry, probe: vec2<f32>, feather: f32) -> f32 {
  let centre = vec2<f32>(shape.ax, shape.ay);
  let half_size = vec2<f32>(shape.bx, shape.by);
  if (any(half_size <= vec2<f32>(0.0)) || shape.start_head_cx <= 0.0 ||
      shape.start_head_cy <= 0.0 || any(abs(probe - centre) > half_size + feather * 2.0)) {
    return 0.0;
  }
  return annotation_edge(annotation_magnify_box(probe, centre, half_size, shape.rounding),
                         feather);
}

// How much of `probe` a magnifier's loupe covers, over its exposure as
// `annotation_magnify_layer` draws it: what a cursor lying under the loupe is
// hidden by.
fn annotation_magnify_cover(magnifier: PreviewArrow, probe: vec2<f32>, feather: f32) -> f32 {
  if (magnifier.sample_count == 0u) {
    return annotation_magnify_inside(magnifier.geometry, probe, feather);
  }
  var total = 0.0;
  for (var tap = 0u; tap < magnifier.sample_count; tap++) {
    total += annotation_magnify_inside(annotation_samples[magnifier.sample_first + tap].geometry,
                                       probe, feather);
  }
  return total / f32(magnifier.sample_count);
}

/// Where one prepared magnifier can reach: its loupe and the shadow under
/// it, its zoom area's outline and halo, and the line between them.
fn annotation_magnify_reach(shape: PreviewGeometry, flags: u32, feather: f32,
                            halo: f32) -> vec4<f32> {
  let centre = vec2<f32>(shape.ax, shape.ay);
  let half_size = vec2<f32>(shape.bx, shape.by);
  let area = vec2<f32>(shape.cx, shape.cy);
  let area_half = vec2<f32>(shape.start_head_cx, shape.start_head_cy);
  if (any(half_size <= vec2<f32>(0.0)) || any(area_half <= vec2<f32>(0.0))) {
    return annotation_no_bounds;
  }
  let stroke = max(shape.width, 0.0);
  let pen = max(stroke * 0.5, feather * 2.0);
  let sigma = clamp(min(half_size.x, half_size.y) * 0.11, 2.0, 110.0);
  let shadowed = (flags & annotation_magnify_shadow_flag) != 0u;
  let reach = stroke + feather * 2.0 + select(0.0, sigma * 4.0 + sigma * 0.35, shadowed);
  var bounds = vec4<f32>(centre - half_size - reach, centre + half_size + reach);
  let outline = area_half + pen + halo + feather * 2.0;
  bounds = annotation_union(bounds, vec4<f32>(area - outline, area + outline));
  if (shape.head != 0u) {
    let start = vec2<f32>(shape.start_head_ax, shape.start_head_ay);
    let end = vec2<f32>(shape.start_head_bx, shape.start_head_by);
    let line = pen + feather * 2.0;
    bounds = annotation_union(bounds, vec4<f32>(min(start, end) - line, max(start, end) + line));
  }
  return bounds;
}

/// Where a magnifier can reach, over its exposure as it is drawn.
fn annotation_magnify_bounds(magnifier: PreviewArrow, feather: f32) -> vec4<f32> {
  let halo = max(magnifier.hover, 0.0);
  var bounds = annotation_magnify_reach(magnifier.geometry, magnifier.flags, feather, halo);
  for (var tap = 0u; tap < magnifier.sample_count; tap++) {
    bounds = annotation_union(bounds, annotation_magnify_reach(
        annotation_samples[magnifier.sample_first + tap].geometry, magnifier.flags, feather, halo));
  }
  return bounds;
}
