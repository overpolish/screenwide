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
// how far the window's end on that curve is. The HLSL twin of
// `gpu_compositor_macos_shader_source_annotation_draw.h`, and of
// `freehand::geometry::freehand_distance`, which picks the same line.
// Included by `annotations.hlsl`, after the pieces every kind shares.

static const uint annotation_draw_kind = 7u;

// The most curves near one pixel that are kept. A stroke that runs past the
// same pixel more often than this keeps the nearest of its passes.
static const uint annotation_draw_near_most = 16u;

// One curve near the pixel, measured over its whole length: the side
// buffer's index of its first point, and the parameter and the distance of
// the pixel's nearest point on it.
struct AnnotationDrawNear {
  uint first;
  float t;
  float distance;
};

// Collects the curves that come within `reach` of `probe` into `near`, and
// answers how many.
uint annotation_draw_near(float2 probe, PreviewArrow annotation, float reach,
                          out AnnotationDrawNear near[annotation_draw_near_most]) {
  PreviewGeometry stroke = annotation.geometry;
  float2 origin = float2(stroke.ax, stroke.ay);
  float2 unit = float2(stroke.bx, stroke.by);
  uint found = 0u;
  // Every slot is written before any is read; only the first `found` count.
  [unroll] for (uint slot = 0u; slot < annotation_draw_near_most; ++slot) {
    near[slot].first = 0u;
    near[slot].t = 0.0;
    near[slot].distance = 1e20;
  }
  [loop] for (uint at = 0u; at + 2u < annotation.data_count; at += 2u) {
    uint first = annotation.data_offset + at;
    float2 a = origin + annotation_points[first] * unit;
    float2 b = origin + annotation_points[first + 1u] * unit;
    float2 c = origin + annotation_points[first + 2u] * unit;
    if (any(probe < min(a, min(b, c)) - reach) || any(probe > max(a, max(b, c)) + reach))
      continue;
    float2 nearest = annotation_curve_nearest(probe, a, b, c, 0.0, 1.0);
    AnnotationDrawNear entry;
    entry.first = first;
    entry.t = nearest.y;
    entry.distance = nearest.x;
    if (found < annotation_draw_near_most) {
      near[found] = entry;
      found += 1u;
      continue;
    }
    uint farthest = 0u;
    [loop] for (uint index = 1u; index < found; ++index)
      if (near[index].distance > near[farthest].distance) farthest = index;
    if (entry.distance < near[farthest].distance) near[farthest] = entry;
  }
  return found;
}

// How far `probe` falls outside the stretch of the line from `low` to `high`
// along it: zero on its edge and negative inside it. Measured from the
// curves `near` it: a nearest point inside a curve's stretch is the
// distance, and one outside it leaves the stretch's ends on that curve.
float annotation_draw_window(float2 probe, PreviewArrow annotation,
                             AnnotationDrawNear near[annotation_draw_near_most], uint found,
                             float low, float high) {
  if (high <= low) return 1e20;
  PreviewGeometry stroke = annotation.geometry;
  float2 origin = float2(stroke.ax, stroke.ay);
  float2 unit = float2(stroke.bx, stroke.by);
  uint count = annotation.data_count;
  float nearest = 1e20;
  [loop] for (uint index = 0u; index < found; ++index) {
    uint first = near[index].first;
    // Where this curve lies along the stroke, and so the part of it, by its
    // own parameter, that the window shows.
    float2 lies = annotation_points[annotation.data_offset + count +
                                    (first - annotation.data_offset) / 2u];
    if (lies.y < low || lies.x > high) continue;
    float span = max(lies.y - lies.x, 1e-6);
    float from = saturate((low - lies.x) / span);
    float to = saturate((high - lies.x) / span);
    if (near[index].t >= from && near[index].t <= to) {
      nearest = min(nearest, near[index].distance);
      continue;
    }
    float2 a = origin + annotation_points[first] * unit;
    float2 b = origin + annotation_points[first + 1u] * unit;
    float2 c = origin + annotation_points[first + 2u] * unit;
    float2 leg = b - a;
    float2 bend = a - 2.0 * b + c;
    nearest = min(nearest, min(distance(probe, annotation_curve_point(a, leg, bend, from)),
                               distance(probe, annotation_curve_point(a, leg, bend, to))));
  }
  return nearest - stroke.width * 0.5;
}

// One stroke: its line in the annotation's own colour, haloed while the
// pointer rests on it, and smeared over the exposure while it draws itself
// in or out.
float4 annotation_draw_layer(
    float4 rgba, PreviewArrow annotation, float4 color, float2 canvas_point,
    float feather, float halo) {
  PreviewGeometry stroke = annotation.geometry;
  float reach = stroke.width * 0.5 + halo + feather + 1.0;
  float2 low = float2(stroke.ax, stroke.ay);
  float2 high = float2(stroke.cx, stroke.cy);
  if (any(canvas_point < low - reach) || any(canvas_point > high + reach)) return rgba;
  AnnotationDrawNear near[annotation_draw_near_most];
  uint found = annotation_draw_near(canvas_point, annotation, reach, near);
  if (found == 0u) return rgba;
  float edge_distance =
      annotation_draw_window(canvas_point, annotation, near, found, stroke.low, stroke.high);
  if (halo > 0.0) {
    // The halo hugs the line from its edge outwards, the ruler's way.
    float band = (1.0 - annotation_edge(edge_distance, feather)) *
        annotation_edge(edge_distance - halo, feather);
    float halo_alpha = band * color.a * annotation_hover_alpha;
    if (halo_alpha > 0.0) {
      rgba.rgb = color.rgb * halo_alpha + rgba.rgb * (1.0 - halo_alpha);
      rgba.a = halo_alpha + rgba.a * (1.0 - halo_alpha);
    }
  }
  float coverage = annotation_edge(edge_distance, feather);
  if (annotation.sample_count > 0u) {
    float total = 0.0;
    [loop] for (uint tap = 0u; tap < annotation.sample_count; ++tap) {
      PreviewSample sample = annotation_samples[annotation.sample_first + tap];
      float sampled = annotation_draw_window(canvas_point, annotation, near, found,
                                             sample.geometry.low, sample.geometry.high);
      total += annotation_edge(sampled, feather) * sample.opacity;
    }
    coverage = total / (float)annotation.sample_count;
  }
  float alpha = coverage * color.a;
  if (alpha <= 0.0) return rgba;
  rgba.rgb = color.rgb * alpha + rgba.rgb * (1.0 - alpha);
  rgba.a = alpha + rgba.a * (1.0 - alpha);
  return rgba;
}
