// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The counter: a disc with a tail and its number. Every number it reads was
// prepared by `counter::geometry::prepare_counter`.

/// A counter read out of the slots an arrow fills with its curve: the disc's
/// centre and radius, the tail's tip and the radius it is rounded to, and where
/// its number was rasterised in the text atlas.
struct PreviewCounter {
  center: vec2<f32>,
  tip: vec2<f32>,
  radius: f32,
  tip_radius: f32,
  text_origin: vec2<f32>,
  text_size: vec2<f32>,
}

fn annotation_counter(prepared: PreviewGeometry) -> PreviewCounter {
  var counter: PreviewCounter;
  counter.center = vec2<f32>(prepared.ax, prepared.ay);
  counter.tip = vec2<f32>(prepared.bx, prepared.by);
  counter.radius = prepared.rounding;
  counter.tip_radius = prepared.low;
  counter.text_origin = vec2<f32>(prepared.start_head_ax, prepared.start_head_ay);
  counter.text_size = vec2<f32>(prepared.start_head_bx, prepared.start_head_by);
  return counter;
}

/// How far a point falls outside a counter's silhouette: the disc, unioned
/// with the tail - the overlap of two circles, one either side of the axis,
/// each tangent to the disc and to the little circle the tip is rounded to,
/// their centres a radius behind the disc's own.
///
/// The side circles are solved here from the disc, the tip and the reach.
/// Their overlap is measured the way any lens is - to the near circle inside
/// its span, to the shared corner past it - and that corner is the tip's
/// centre, so taking the tip's radius off rounds the point without moving it.
/// Each side circle is tangent to the disc on the line through both centres,
/// so behind that line the nearest edge is the disc's: there the disc alone is
/// measured. The per-pixel twin of `counter_silhouette_distance` in
/// `annotations/counter/silhouette.rs`.
fn annotation_counter_distance(probe: vec2<f32>, counter: PreviewCounter) -> f32 {
  let local = probe - counter.center;
  let reach = counter.tip - counter.center;
  let length_reach = length(reach);
  if (counter.radius <= 0.0) {
    return length(local);
  }
  if (length_reach <= 0.0) {
    return length(local) - counter.radius;
  }
  let axis = reach / length_reach;
  let along = dot(local, axis);
  let across = abs(local.x * -axis.y + local.y * axis.x);
  let disc = length(local) - counter.radius;
  let gap = counter.radius - counter.tip_radius;
  if (gap <= 0.0) {
    return disc;
  }
  let tip = length_reach - counter.tip_radius;
  let side = (tip * tip + 2.0 * tip * counter.radius + gap * gap) / (2.0 * gap);
  let apart = side + counter.tip_radius - counter.radius;
  var offset = apart * apart - counter.radius * counter.radius;
  if (offset <= 0.0) {
    return disc;
  }
  offset = sqrt(offset);
  var lens: f32;
  if ((along - tip) * offset > across * (tip + counter.radius)) {
    lens = length(vec2<f32>(across, along - tip));
  } else {
    lens = length(vec2<f32>(across + offset, along + counter.radius)) - side;
  }
  if (along * offset < across * counter.radius) {
    return disc;
  }
  return min(disc, lens - counter.tip_radius);
}

/// How much of the number covers this pixel, from the atlas the numbers were
/// rasterised into at `atlas.scale` pixels to the canvas pixel. `feather` is
/// half a drawn pixel, in canvas pixels, which is how far the atlas read
/// spreads. The number is centred on the disc and carried by the disc's own
/// radius, so it grows and shrinks with the annotation.
fn annotation_number_coverage(probe: vec2<f32>, counter: PreviewCounter,
                              atlas: AnnotationTextAtlas, feather: f32) -> f32 {
  if (atlas.size.x == 0u || atlas.size.y == 0u || atlas.scale <= 0.0 ||
      counter.text_size.x <= 0.0 || counter.text_size.y <= 0.0) {
    return 0.0;
  }
  let drawn = counter.text_size / atlas.scale;
  let local = probe - counter.center + drawn * 0.5;
  if (any(local < vec2<f32>(0.0)) || any(local > drawn)) {
    return 0.0;
  }
  // The atlas rows run top-down from its first pixel, which is the space the
  // rasteriser reports its rectangles in.
  return annotation_atlas_coverage(atlas, counter.text_origin, counter.text_size,
                                   counter.text_origin + local * atlas.scale,
                                   2.0 * feather * atlas.scale);
}

/// Accumulated exposure coverage for a counter: the disc is drawn at every
/// prepared sample between the shutter start and now, so one that grew
/// through the frame smears over the sizes it covered rather than jumping
/// between them. Each sample carries its own opacity, which is what fades a
/// counter in and out of an exported frame.
fn annotation_counter_exposure(probe: vec2<f32>, annotation: PreviewArrow, feather: f32) -> f32 {
  var total = 0.0;
  for (var tap = 0u; tap < annotation.sample_count; tap++) {
    let sample = annotation_samples[annotation.sample_first + tap];
    let distance = annotation_counter_distance(probe, annotation_counter(sample.geometry));
    total += annotation_edge(distance, feather) * sample.opacity;
  }
  return total / f32(annotation.sample_count);
}

/// Where a counter can reach. The disc and tail fit within twice the radius
/// of the centre. The exposure samples run steadily from last frame's reveal
/// to this one's, so the first and last hold the largest radius any of them
/// is drawn at.
fn annotation_counter_bounds(annotation: PreviewArrow, halo: f32, feather: f32) -> vec4<f32> {
  let counter = annotation_counter(annotation.geometry);
  var radius = counter.radius;
  if (annotation.sample_count > 0u) {
    let last = annotation.sample_first + annotation.sample_count - 1u;
    radius = max(radius, max(
        annotation_counter(annotation_samples[annotation.sample_first].geometry).radius,
        annotation_counter(annotation_samples[last].geometry).radius));
  }
  let reach = radius * 2.0 + halo + feather + 1.0;
  return vec4<f32>(counter.center - reach, counter.center + reach);
}

/// One counter: its silhouette in the annotation's own colour, and its number
/// in whichever of black or white reads on that colour. The number is clipped
/// to the silhouette's own coverage, so the two share one antialiased edge.
fn annotation_counter_layer(rgba_in: vec4<f32>, annotation: PreviewArrow, color: vec4<f32>,
                            canvas_point: vec2<f32>, feather: f32, halo: f32,
                            atlas: AnnotationTextAtlas) -> vec4<f32> {
  if (annotation_outside(canvas_point, annotation_counter_bounds(annotation, halo, feather))) {
    return rgba_in;
  }
  let counter = annotation_counter(annotation.geometry);
  let distance = annotation_counter_distance(canvas_point, counter);
  var rgba = annotation_halo(rgba_in, color, distance, halo, feather);
  // A still frame draws the prepared disc directly, its opacity already
  // folded into the colour; a moving one averages the disc over the
  // exposure, where each sample carries the opacity it had.
  var coverage: f32;
  if (annotation.sample_count == 0u) {
    coverage = annotation_edge(distance, feather);
  } else {
    coverage = annotation_counter_exposure(canvas_point, annotation, feather);
  }
  if (coverage <= 0.0) {
    return rgba;
  }
  let alpha = coverage * color.a;
  rgba = annotation_over(rgba, color.rgb, alpha);
  let ink = annotation_number_coverage(canvas_point, counter, atlas, feather) * alpha;
  if (ink <= 0.0) {
    return rgba;
  }
  return annotation_over(rgba, annotation_label_tint(color.rgb), ink);
}
