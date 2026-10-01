// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

/// The draw tool's half of the annotation shader: a line drawn freehand, as
/// the chain of quadratic curves Rust fitted it to. The chain rides in the
/// side buffer, from the corner of the stroke's box in source pixels, placed
/// here by the record's `a` (where that corner lands) and `b` (one source
/// pixel's reach); `c` is the box's far corner, which turns away a pixel far
/// from the stroke. After the chain comes one entry a curve, where along the
/// stroke it starts and ends, so a stroke drawing itself in draws of each
/// curve only what its `low` to `high` covers.
///
/// A stroke drawing itself in is smeared over the exposure as every other
/// kind is, each sample at the window it had then. Measuring the whole chain
/// again for every sample would cost the pixel dozens of passes, so the
/// curves near the pixel are measured once, each over its whole length, and
/// each sample only asks whether that nearest point lies inside its window -
/// and where it does not, how far the window's end on that curve is. The
/// twin of `annotation_draw.wgsl`, and of
/// `freehand::geometry::freehand_distance`, which picks the same line.
#define GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_DRAW @R"METAL(
constant uint annotation_draw_kind = 7u;

/// The most curves near one pixel that are kept. A stroke that runs past
/// the same pixel more often than this keeps the nearest of its passes.
constant uint annotation_draw_near_most = 16u;

/// One curve near the pixel, measured over its whole length: the side
/// buffer's index of its first point, and the parameter and the distance of
/// the pixel's nearest point on it.
struct AnnotationDrawNear {
  uint first;
  float t;
  float distance;
};

/// Collects the curves that come within `reach` of `point` into `near`, and
/// answers how many.
static uint annotation_draw_near(float2 point, const device AnnotationUniforms &annotation,
                                 const device packed_float2 *points, float reach,
                                 thread AnnotationDrawNear *near) {
  float2 origin = float2(annotation.arrow.a);
  float2 unit = float2(annotation.arrow.b);
  uint found = 0u;
  for (uint at = 0u; at + 2u < annotation.data_count; at += 2u) {
    uint first = annotation.data_offset + at;
    float2 a = origin + float2(points[first]) * unit;
    float2 b = origin + float2(points[first + 1u]) * unit;
    float2 c = origin + float2(points[first + 2u]) * unit;
    if (any(point < min(a, min(b, c)) - reach) || any(point > max(a, max(b, c)) + reach))
      continue;
    float2 nearest = annotation_curve_nearest(point, a, b, c, 0.0, 1.0);
    AnnotationDrawNear entry = {first, nearest.y, nearest.x};
    if (found < annotation_draw_near_most) {
      near[found++] = entry;
      continue;
    }
    uint farthest = 0u;
    for (uint index = 1u; index < found; ++index)
      if (near[index].distance > near[farthest].distance) farthest = index;
    if (entry.distance < near[farthest].distance) near[farthest] = entry;
  }
  return found;
}

/// How far `point` falls outside the stretch of the line from `low` to
/// `high` along it: zero on its edge and negative inside it. Measured from
/// the curves `near` it: a nearest point inside a curve's stretch is the
/// distance, and one outside it leaves the stretch's ends on that curve.
static float annotation_draw_window(float2 point, const device AnnotationUniforms &annotation,
                                    const device packed_float2 *points,
                                    thread const AnnotationDrawNear *near, uint found,
                                    float low, float high) {
  if (high <= low) return 1e20;
  float2 origin = float2(annotation.arrow.a);
  float2 unit = float2(annotation.arrow.b);
  uint count = annotation.data_count;
  float nearest = 1e20;
  for (uint index = 0u; index < found; ++index) {
    uint first = near[index].first;
    // Where this curve lies along the stroke, and so the part of it, by its
    // own parameter, that the window shows.
    float2 lies = float2(points[annotation.data_offset + count + (first - annotation.data_offset) / 2u]);
    if (lies.y < low || lies.x > high) continue;
    float span = max(lies.y - lies.x, 1e-6);
    float from = saturate((low - lies.x) / span);
    float to = saturate((high - lies.x) / span);
    if (near[index].t >= from && near[index].t <= to) {
      nearest = min(nearest, near[index].distance);
      continue;
    }
    float2 a = origin + float2(points[first]) * unit;
    float2 b = origin + float2(points[first + 1u]) * unit;
    float2 c = origin + float2(points[first + 2u]) * unit;
    float2 leg = b - a;
    float2 bend = a - 2.0 * b + c;
    nearest = min(nearest, min(distance(point, annotation_curve_point(a, leg, bend, from)),
                               distance(point, annotation_curve_point(a, leg, bend, to))));
  }
  return nearest - annotation.arrow.width * 0.5;
}

/// One stroke: its line in the annotation's own colour, haloed while the
/// pointer rests on it, and smeared over the exposure while it draws itself
/// in or out.
static float4 annotation_draw_layer(
    float4 rgba, const device AnnotationUniforms &annotation, float4 color,
    float2 canvas_point, float feather, float halo, const device AnnotationSample *samples,
    const device packed_float2 *points) {
  float reach = annotation.arrow.width * 0.5 + halo + feather + 1.0;
  float2 low = float2(annotation.arrow.a);
  float2 high = float2(annotation.arrow.c);
  if (any(canvas_point < low - reach) || any(canvas_point > high + reach)) return rgba;
  AnnotationDrawNear near[annotation_draw_near_most];
  uint found = annotation_draw_near(canvas_point, annotation, points, reach, near);
  if (found == 0u) return rgba;
  float distance = annotation_draw_window(canvas_point, annotation, points, near, found,
                                          annotation.arrow.low, annotation.arrow.high);
  if (halo > 0.0) {
    // The halo hugs the line from its edge outwards, the ruler's way.
    float band = (1.0 - annotation_edge(distance, feather)) *
        annotation_edge(distance - halo, feather);
    float alpha = band * color.a * annotation_hover_alpha;
    if (alpha > 0.0) {
      rgba.rgb = color.rgb * alpha + rgba.rgb * (1.0 - alpha);
      rgba.a = alpha + rgba.a * (1.0 - alpha);
    }
  }
  float coverage = annotation_edge(distance, feather);
  if (annotation.sample_count > 0u) {
    float total = 0.0;
    for (uint tap = 0u; tap < annotation.sample_count; ++tap) {
      const device AnnotationSample &sample = samples[annotation.sample_offset + tap];
      float sampled = annotation_draw_window(canvas_point, annotation, points, near, found,
                                             sample.arrow.low, sample.arrow.high);
      total += annotation_edge(sampled, feather) * sample.opacity;
    }
    coverage = total / float(annotation.sample_count);
  }
  float alpha = coverage * color.a;
  if (alpha <= 0.0) return rgba;
  rgba.rgb = color.rgb * alpha + rgba.rgb * (1.0 - alpha);
  rgba.a = alpha + rgba.a * (1.0 - alpha);
  return rgba;
}
)METAL"
