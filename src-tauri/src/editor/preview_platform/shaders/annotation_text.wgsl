// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The text box: the box and its pointer read out of the slots an arrow fills
// with its curve, the distance to them, its type sampled from the atlas, and
// the layer that draws them. The twin of
// `gpu_compositor_macos_shader_source_annotation_text.h`.

/// A text box read out of the slots an arrow fills with its curve. Prepared
/// once per annotation by Rust's `text::geometry::prepare_text`, which
/// documents every slot.
struct PreviewTextBox {
  low: vec2<f32>,
  high: vec2<f32>,
  radius: f32,
  base: vec2<f32>,
  tip: vec2<f32>,
  base_radius: f32,
  tip_radius: f32,
  blend: f32,
  text_origin: vec2<f32>,
  text_size: vec2<f32>,
  text_centre: vec2<f32>,
}

fn annotation_text_box(prepared: PreviewGeometry) -> PreviewTextBox {
  var box: PreviewTextBox;
  box.low = vec2<f32>(prepared.ax, prepared.ay);
  box.high = vec2<f32>(prepared.bx, prepared.by);
  box.radius = prepared.rounding;
  box.base = vec2<f32>(prepared.end_head_ax, prepared.end_head_ay);
  box.tip = vec2<f32>(prepared.end_head_bx, prepared.end_head_by);
  box.base_radius = prepared.high;
  box.tip_radius = prepared.low;
  box.blend = prepared.end_head_cx;
  box.text_origin = vec2<f32>(prepared.start_head_ax, prepared.start_head_ay);
  box.text_size = vec2<f32>(prepared.start_head_bx, prepared.start_head_by);
  box.text_centre = vec2<f32>(prepared.start_head_cx, prepared.start_head_cy);
  return box;
}

fn annotation_rounded_box_distance(probe: vec2<f32>, low: vec2<f32>, high: vec2<f32>,
                                   radius: f32) -> f32 {
  let half_size = (high - low) * 0.5;
  let q = abs(probe - (low + high) * 0.5) - half_size + radius;
  return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - radius;
}

/// A stroke tapering from a circle at `base` to a smaller one at `tip`, its
/// sides tangent to both. The per-pixel twin of `tapered_distance` in
/// `annotations/text/geometry.rs`.
fn annotation_tapered_distance(probe: vec2<f32>, base: vec2<f32>, tip: vec2<f32>,
                               base_radius: f32, tip_radius: f32) -> f32 {
  let axis = tip - base;
  let span = dot(axis, axis);
  let taper = base_radius - tip_radius;
  let local = probe - base;
  if (span <= taper * taper) {
    return min(length(local) - base_radius, length(probe - tip) - tip_radius);
  }
  let q = vec2<f32>(abs(local.x * axis.y - local.y * axis.x), dot(local, axis)) / span;
  let side = vec2<f32>(sqrt(span - taper * taper), taper);
  let cross_value = side.x * q.y - side.y * q.x;
  if (cross_value < 0.0) {
    return sqrt(span * dot(q, q)) - base_radius;
  }
  if (cross_value > side.x) {
    return sqrt(span * (dot(q, q) + 1.0 - 2.0 * q.y)) - tip_radius;
  }
  return dot(side, q) - base_radius;
}

/// How far a point falls outside a text box, pointer included: the box and
/// the pointer blended over `blend`, so the pointer leaves the edge in a
/// curve. The per-pixel twin of `text_distance`.
fn annotation_text_distance(probe: vec2<f32>, box: PreviewTextBox) -> f32 {
  let body = annotation_rounded_box_distance(probe, box.low, box.high, box.radius);
  if (box.base_radius <= 0.0) {
    return body;
  }
  let pointer = annotation_tapered_distance(probe, box.base, box.tip, box.base_radius,
                                            box.tip_radius);
  if (box.blend <= 0.0) {
    return min(body, pointer);
  }
  let mix_amount = max(box.blend - abs(body - pointer), 0.0) / box.blend;
  return min(body, pointer) - mix_amount * mix_amount * box.blend * 0.25;
}

/// How much of the type covers this pixel, from the atlas it was rasterised
/// into at `atlas.scale` pixels to the canvas pixel, centred on the text
/// block. `feather` is half a drawn pixel, in canvas pixels, which is how far
/// the atlas read spreads.
fn annotation_text_coverage(probe: vec2<f32>, box: PreviewTextBox, atlas: AnnotationTextAtlas,
                            feather: f32) -> f32 {
  if (atlas.size.x == 0u || atlas.size.y == 0u || atlas.scale <= 0.0 ||
      box.text_size.x <= 0.0 || box.text_size.y <= 0.0) {
    return 0.0;
  }
  let drawn = box.text_size / atlas.scale;
  let local = probe - box.text_centre + drawn * 0.5;
  if (any(local < vec2<f32>(0.0)) || any(local > drawn)) {
    return 0.0;
  }
  return annotation_atlas_coverage(atlas, box.text_origin, box.text_size,
                                   box.text_origin + local * atlas.scale,
                                   2.0 * feather * atlas.scale);
}

/// A text box averaged over its exposure samples, as a counter is while it
/// grows into place.
fn annotation_text_exposure(probe: vec2<f32>, annotation: PreviewArrow, feather: f32) -> f32 {
  var total = 0.0;
  for (var tap = 0u; tap < annotation.sample_count; tap++) {
    let sample = annotation_samples[annotation.sample_first + tap];
    let distance = annotation_text_distance(probe, annotation_text_box(sample.geometry));
    total += annotation_edge(distance, feather) * sample.opacity;
  }
  return total / f32(annotation.sample_count);
}

/// `bounds` (low in `xy`, high in `zw`) grown to hold a text box and its
/// pointer, `reach` beyond.
fn annotation_text_bounds(box: PreviewTextBox, reach: f32, bounds: vec4<f32>) -> vec4<f32> {
  var low = min(bounds.xy, box.low - reach);
  var high = max(bounds.zw, box.high + reach);
  if (box.base_radius > 0.0) {
    low = min(low, box.tip - box.tip_radius - reach);
    high = max(high, box.tip + box.tip_radius + reach);
  }
  return vec4<f32>(low, high);
}

/// One text box: its box and pointer in the annotation's own colour, and its
/// type in whichever of black or white reads on that colour, clipped to the
/// box's own coverage so the two share one antialiased edge.
fn annotation_text_layer(rgba_in: vec4<f32>, annotation: PreviewArrow, color: vec4<f32>,
                         canvas_point: vec2<f32>, feather: f32, halo: f32,
                         atlas: AnnotationTextAtlas) -> vec4<f32> {
  let box = annotation_text_box(annotation.geometry);
  let reach = halo + feather + 1.0;
  var bounds = annotation_text_bounds(box, reach, vec4<f32>(1e20, 1e20, -1e20, -1e20));
  if (annotation.sample_count > 0u) {
    // The exposure samples run steadily from last frame's reveal to this
    // one's, so the first and last hold every one between: a pointer drawing
    // back in trails past where its tip is now.
    let last = annotation.sample_first + annotation.sample_count - 1u;
    bounds = annotation_text_bounds(
        annotation_text_box(annotation_samples[annotation.sample_first].geometry), reach, bounds);
    bounds = annotation_text_bounds(annotation_text_box(annotation_samples[last].geometry), reach,
                                    bounds);
  }
  if (any(canvas_point < bounds.xy) || any(canvas_point > bounds.zw)) {
    return rgba_in;
  }
  let distance = annotation_text_distance(canvas_point, box);
  var rgba = annotation_halo(rgba_in, color, distance, halo, feather);
  var coverage: f32;
  if (annotation.sample_count == 0u) {
    coverage = annotation_edge(distance, feather);
  } else {
    coverage = annotation_text_exposure(canvas_point, annotation, feather);
  }
  if (coverage <= 0.0) {
    return rgba;
  }
  let alpha = coverage * color.a;
  rgba = annotation_over(rgba, color.rgb, alpha);
  let ink = annotation_text_coverage(canvas_point, box, atlas, feather) * alpha;
  if (ink <= 0.0) {
    return rgba;
  }
  // The counter's rule, so a text box and a counter in one colour carry the
  // same ink.
  return annotation_over(rgba, annotation_label_tint(color.rgb), ink);
}
