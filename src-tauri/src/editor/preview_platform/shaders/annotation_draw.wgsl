// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The draw tool's stroke: a line drawn freehand, as the chain of quadratic
// curves Rust fitted it to. The chain rides in `annotation_points`, from the
// corner of the stroke's box in source pixels, placed by the prepared
// record's `a` (where that corner lands) and `b` (one source pixel's reach);
// `c` is the box's far corner, which turns away a pixel far from the stroke.
// After the chain comes one entry a curve, where along the stroke it starts
// and ends, so a stroke drawing itself in draws of each curve only what its
// `low` to `high` covers.
//
// A stroke drawing itself in is smeared over the exposure as every other
// kind is, each sample at the window it had then. The curves near the pixel
// are measured once, each over its whole length, and each sample only asks
// whether that nearest point lies inside its window - and where it does not,
// how far the window's end on that curve is. The twin of
// `gpu_compositor_macos_shader_source_annotation_draw.h`, and of
// `freehand::geometry::freehand_distance`, which picks the same line.

const annotation_draw_kind: u32 = 7u;

// The most curves near one pixel that are kept. A stroke that runs past the
// same pixel more often than this keeps the nearest of its passes.
const annotation_draw_near_most: u32 = 16u;

// One curve near the pixel, measured over its whole length: the side
// buffer's index of its first point, and the parameter and the distance of
// the pixel's nearest point on it.
struct AnnotationDrawNear {
  first: u32,
  t: f32,
  distance: f32,
}

alias AnnotationDrawNearList = array<AnnotationDrawNear, annotation_draw_near_most>;

// Collects the curves that come within `reach` of `probe` into `near`, and
// answers how many.
fn annotation_draw_near(probe: vec2<f32>, annotation: PreviewArrow, reach: f32,
                        near: ptr<function, AnnotationDrawNearList>) -> u32 {
  let stroke = annotation.geometry;
  let origin = vec2<f32>(stroke.ax, stroke.ay);
  let unit = vec2<f32>(stroke.bx, stroke.by);
  var found = 0u;
  for (var at = 0u; at + 2u < annotation.data_count; at += 2u) {
    let first = annotation.data_offset + at;
    let a = origin + annotation_points[first] * unit;
    let b = origin + annotation_points[first + 1u] * unit;
    let c = origin + annotation_points[first + 2u] * unit;
    if (any(probe < min(a, min(b, c)) - reach) || any(probe > max(a, max(b, c)) + reach)) {
      continue;
    }
    let nearest = annotation_curve_nearest(probe, a, b, c, 0.0, 1.0);
    let entry = AnnotationDrawNear(first, nearest.y, nearest.x);
    if (found < annotation_draw_near_most) {
      (*near)[found] = entry;
      found += 1u;
      continue;
    }
    var farthest = 0u;
    for (var index = 1u; index < found; index++) {
      if ((*near)[index].distance > (*near)[farthest].distance) {
        farthest = index;
      }
    }
    if (entry.distance < (*near)[farthest].distance) {
      (*near)[farthest] = entry;
    }
  }
  return found;
}

// How far `probe` falls outside the stretch of the line from `low` to `high`
// along it: zero on its edge and negative inside it. Measured from the
// curves `near` it: a nearest point inside a curve's stretch is the
// distance, and one outside it leaves the stretch's ends on that curve.
fn annotation_draw_window(probe: vec2<f32>, annotation: PreviewArrow,
                          near: ptr<function, AnnotationDrawNearList>, found: u32, low: f32,
                          high: f32) -> f32 {
  if (high <= low) {
    return 1e20;
  }
  let stroke = annotation.geometry;
  let origin = vec2<f32>(stroke.ax, stroke.ay);
  let unit = vec2<f32>(stroke.bx, stroke.by);
  let count = annotation.data_count;
  var nearest = 1e20;
  for (var index = 0u; index < found; index++) {
    let first = (*near)[index].first;
    // Where this curve lies along the stroke, and so the part of it, by its
    // own parameter, that the window shows.
    let lies = annotation_points[annotation.data_offset + count +
                                 (first - annotation.data_offset) / 2u];
    if (lies.y < low || lies.x > high) {
      continue;
    }
    let span = max(lies.y - lies.x, 1e-6);
    let start = saturate((low - lies.x) / span);
    let end = saturate((high - lies.x) / span);
    if ((*near)[index].t >= start && (*near)[index].t <= end) {
      nearest = min(nearest, (*near)[index].distance);
      continue;
    }
    let a = origin + annotation_points[first] * unit;
    let b = origin + annotation_points[first + 1u] * unit;
    let c = origin + annotation_points[first + 2u] * unit;
    let leg = b - a;
    let bend = a - 2.0 * b + c;
    nearest = min(nearest, min(distance(probe, annotation_curve_point(a, leg, bend, start)),
                               distance(probe, annotation_curve_point(a, leg, bend, end))));
  }
  return nearest - stroke.width * 0.5;
}

// One stroke: its line in the annotation's own colour, haloed while the
// pointer rests on it, and smeared over the exposure while it draws itself
// in or out.
fn annotation_draw_layer(rgba_in: vec4<f32>, annotation: PreviewArrow, color: vec4<f32>,
                         canvas_point: vec2<f32>, feather: f32, halo: f32) -> vec4<f32> {
  let stroke = annotation.geometry;
  let reach = stroke.width * 0.5 + halo + feather + 1.0;
  let low = vec2<f32>(stroke.ax, stroke.ay);
  let high = vec2<f32>(stroke.cx, stroke.cy);
  if (any(canvas_point < low - reach) || any(canvas_point > high + reach)) {
    return rgba_in;
  }
  var near: AnnotationDrawNearList;
  let found = annotation_draw_near(canvas_point, annotation, reach, &near);
  if (found == 0u) {
    return rgba_in;
  }
  let edge_distance = annotation_draw_window(canvas_point, annotation, &near, found, stroke.low,
                                             stroke.high);
  let rgba = annotation_halo(rgba_in, color, edge_distance, halo, feather);
  var coverage = annotation_edge(edge_distance, feather);
  if (annotation.sample_count > 0u) {
    var total = 0.0;
    for (var tap = 0u; tap < annotation.sample_count; tap++) {
      let sample = annotation_samples[annotation.sample_first + tap];
      let sampled = annotation_draw_window(canvas_point, annotation, &near, found,
                                           sample.geometry.low, sample.geometry.high);
      total += annotation_edge(sampled, feather) * sample.opacity;
    }
    coverage = total / f32(annotation.sample_count);
  }
  let alpha = coverage * color.a;
  if (alpha <= 0.0) {
    return rgba;
  }
  return annotation_over(rgba, color.rgb, alpha);
}
