// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

/// The shape's own half of the annotation shader: an outline round a rounded
/// box, walked clockwise from the top side's left end - four sides and four
/// corners - and the pen's stroke along a window of that walk with a round
/// pen at each end. Each piece of the walk is measured on its own and the
/// nearest kept, so wherever the pen went is drawn. Every number it reads was
/// prepared by Rust's `prepare_shape`, over `screenwide_annotation_prepare`,
/// which also decides a hand-drawn stroke's wander; this only evaluates. The
/// per-pixel twin of `shape_distance` in `annotations/outline/geometry.rs`,
/// which picks the same stroke.
#define GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_SHAPE @R"METAL(
constant float annotation_shape_quarter = 1.5707963268;
constant float annotation_shape_turn = 6.2831853072;

static float annotation_shape_perimeter(float2 half_size, float rounding) {
  return 4.0 * (half_size.x + half_size.y - 2.0 * rounding) + annotation_shape_turn * rounding;
}

/// How far the stroke sits outside the outline `along` into it in `x`, and
/// how fast that changes along it in `y`: a bow all the way round, a lead-in
/// easing off from the start and a trail-off easing in towards the end.
/// Nowhere for a clean stroke, whose wander is all zero. The twin of
/// `wander` and `slope` in `annotations/outline/wander.rs`.
static float2 annotation_shape_wander(const device AnnotationArrowGeometry &shape, float along) {
  float2 bow = float2(shape.start_head.a);
  float2 first = float2(shape.start_head.b);
  float2 second = float2(shape.start_head.c);
  float trail = float2(shape.end_head.a).x;
  float2 ease = max(float2(shape.end_head.b), 1e-6);
  // The lead-in settles in an S from its start; the trail-off flicks off,
  // steepest at the tip. Each is gone past its own span.
  float lead_x = saturate(along / ease.x);
  float trail_x = saturate((float2(shape.c).y - along) / ease.y);
  float first_at = first.x * along + first.y;
  float second_at = second.x * along + second.y;
  return float2(
      bow.x * (0.85 * sin(first_at) + 0.15 * sin(second_at)) +
          bow.y * (1.0 - lead_x * lead_x * (3.0 - 2.0 * lead_x)) +
          trail * (1.0 - trail_x) * (1.0 - trail_x),
      bow.x * (0.85 * first.x * cos(first_at) + 0.15 * second.x * cos(second_at)) -
          bow.y * 6.0 * lead_x * (1.0 - lead_x) / ease.x +
          trail * 2.0 * (1.0 - trail_x) / ease.y);
}

/// Where the stroke begins on the walk's piece `start` into it, before any
/// whole laps are added.
static float annotation_shape_base(const device AnnotationArrowGeometry &shape, float start,
                                   float lap) {
  float base = start - float2(shape.c).x;
  return base - lap * floor(base / lap);
}

/// How far `local` is from the pen's line along one side, from its `from`
/// end along `toward` for `run`, `normal` pointing out. `window` is the part
/// of the stroke drawn; the stroke may pass over the side on three laps.
/// Square to the side, the distance is scaled back by the line's tilt, so a
/// line leaving the outline keeps the pen's width rather than thinning.
static float annotation_shape_side(
    float2 local, float2 from, float2 toward, float2 normal, float start, float run,
    const device AnnotationArrowGeometry &shape, float lap, float2 window) {
  float2 relative = local - from;
  float along = dot(relative, toward);
  float off = dot(relative, normal);
  float base = annotation_shape_base(shape, start, lap);
  float nearest = 1e20;
  for (int turn = -1; turn <= 1; ++turn) {
    float offset = base + float(turn) * lap;
    float low = max(window.x - offset, 0.0);
    float high = min(window.y - offset, run);
    if (low > high) continue;
    float at = clamp(along, low, high);
    float2 aside = annotation_shape_wander(shape, offset + at);
    nearest = min(nearest, along == at
        ? abs(off - aside.x) * rsqrt(1.0 + aside.y * aside.y)
        : length(relative - (toward * at + normal * aside.x)));
  }
  return nearest;
}

/// How far `local` is from the pen's line round one corner: a quarter turn
/// about `centre` from `from_angle`. A square corner is passed at one moment,
/// and faces every way between its sides.
static float annotation_shape_corner(
    float2 local, float2 centre, float from_angle, float rounding, float start,
    const device AnnotationArrowGeometry &shape, float lap, float2 window) {
  float2 relative = local - centre;
  float turned = atan2(relative.y, relative.x) - from_angle;
  // Held to the half turn centred on the corner's own, so a point behind it
  // lands on the nearer of its two ends.
  turned -= annotation_shape_turn *
            floor((turned + 1.5 * annotation_shape_quarter) / annotation_shape_turn);
  float base = annotation_shape_base(shape, start, lap);
  float nearest = 1e20;
  for (int turn = -1; turn <= 1; ++turn) {
    float offset = base + float(turn) * lap;
    float low = max(window.x - offset, 0.0);
    float high = min(window.y - offset, annotation_shape_quarter * rounding);
    if (low > high) continue;
    float2 span = rounding > 0.0 ? float2(low, high) / rounding
                                 : float2(0.0, annotation_shape_quarter);
    float at = clamp(turned, span.x, span.y);
    float2 aside = annotation_shape_wander(shape, offset + at * rounding);
    // A stroke pulled inside a corner further than it is rounded needs no
    // join: its two sides already cross there.
    if (rounding + aside.x < 0.0) continue;
    if (rounding > 0.0 && turned == at) {
      // Measured out from the corner's centre, tilted as a side is.
      float tilt = aside.y * rounding / max(rounding + aside.x, 1e-6);
      nearest = min(nearest, abs(length(relative) - rounding - aside.x) *
                                 rsqrt(1.0 + tilt * tilt));
      continue;
    }
    float angle = from_angle + at;
    nearest = min(nearest,
                  length(relative - float2(cos(angle), sin(angle)) * (rounding + aside.x)));
  }
  return nearest;
}

/// How far `point` falls outside the drawn stroke: zero on its edge and
/// negative inside it.
static float annotation_shape_distance(float2 point,
                                       const device AnnotationArrowGeometry &shape) {
  float2 stroke = float2(shape.c);
  if (shape.high <= shape.low || !(stroke.y > 0.0)) return 1e20;
  float2 low = float2(shape.a);
  float2 high = float2(shape.b);
  float2 half_size = (high - low) * 0.5;
  float rounding = max(min(shape.rounding, min(half_size.x, half_size.y)), 0.0);
  float lap = annotation_shape_perimeter(half_size, rounding);
  float2 local = point - (low + high) * 0.5;
  float2 window = float2(shape.low, shape.high) * stroke.y;
  float2 inner = half_size - rounding;
  float across = 2.0 * inner.x;
  float down = 2.0 * inner.y;
  float bend = annotation_shape_quarter * rounding;
  float q = annotation_shape_quarter;
  // Clockwise from the top side's left end: each side, then the corner
  // after it.
  float start = 0.0;
  float nearest = annotation_shape_side(local, float2(-inner.x, -half_size.y), float2(1.0, 0.0),
                                        float2(0.0, -1.0), start, across, shape, lap, window);
  start += across;
  nearest = min(nearest, annotation_shape_corner(local, float2(inner.x, -inner.y), -q, rounding,
                                                 start, shape, lap, window));
  start += bend;
  nearest = min(nearest, annotation_shape_side(local, float2(half_size.x, -inner.y),
                                               float2(0.0, 1.0), float2(1.0, 0.0), start, down,
                                               shape, lap, window));
  start += down;
  nearest = min(nearest, annotation_shape_corner(local, inner, 0.0, rounding, start, shape, lap,
                                                 window));
  start += bend;
  nearest = min(nearest, annotation_shape_side(local, float2(inner.x, half_size.y),
                                               float2(-1.0, 0.0), float2(0.0, 1.0), start, across,
                                               shape, lap, window));
  start += across;
  nearest = min(nearest, annotation_shape_corner(local, float2(-inner.x, inner.y), q, rounding,
                                                 start, shape, lap, window));
  start += bend;
  nearest = min(nearest, annotation_shape_side(local, float2(-half_size.x, inner.y),
                                               float2(0.0, -1.0), float2(-1.0, 0.0), start, down,
                                               shape, lap, window));
  start += down;
  nearest = min(nearest, annotation_shape_corner(local, -inner, 2.0 * q, rounding, start, shape,
                                                 lap, window));
  return nearest - shape.width * 0.5;
}

/// Accumulated exposure coverage for a shape: the stroke is drawn at every
/// prepared sample between the shutter start and now, so an arriving one
/// smears along the outline it covered. Each sample carries its own opacity.
static float annotation_shape_exposure(
    float2 point, const device AnnotationUniforms &annotation,
    const device AnnotationSample *samples, float feather) {
  float total = 0.0;
  for (uint tap = 0; tap < annotation.sample_count; ++tap) {
    const device AnnotationSample &sample = samples[annotation.sample_offset + tap];
    total += annotation_edge(annotation_shape_distance(point, sample.arrow), feather) *
             sample.opacity;
  }
  return total / float(annotation.sample_count);
}

/// One shape: its stroke in the annotation's own colour.
static float4 annotation_shape_layer(
    float4 rgba, const device AnnotationUniforms &annotation, float4 color,
    float2 canvas_point, float feather, float halo,
    const device AnnotationSample *samples) {
  const device AnnotationArrowGeometry &shape = annotation.arrow;
  // The stroke stays within the pen and its wander of the clean outline.
  // Every exposure sample shares the box and the wander, so these tests
  // cover all of them.
  float2 wobble = float2(shape.start_head.a);
  float trail = float2(shape.end_head.a).x;
  float reach = shape.width * 0.5 + abs(wobble.x) + max(abs(wobble.y), abs(trail)) + halo +
                feather + 1.0;
  float2 low = float2(shape.a);
  float2 high = float2(shape.b);
  if (any(canvas_point < low - reach) || any(canvas_point > high + reach)) return rgba;
  float2 half_size = (high - low) * 0.5;
  float rounding = max(min(shape.rounding, min(half_size.x, half_size.y)), 0.0);
  float2 q = abs(canvas_point - (low + high) * 0.5) - half_size + rounding;
  if (abs(length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - rounding) > reach) return rgba;
  float distance = annotation_shape_distance(canvas_point, shape);
  if (halo > 0.0) {
    // The halo hugs the stroke from its edge outwards, the ruler's way.
    float band = (1.0 - annotation_edge(distance, feather)) *
        annotation_edge(distance - halo, feather);
    float alpha = band * color.a * annotation_hover_alpha;
    if (alpha > 0.0) {
      rgba.rgb = color.rgb * alpha + rgba.rgb * (1.0 - alpha);
      rgba.a = alpha + rgba.a * (1.0 - alpha);
    }
  }
  // A still frame draws the prepared stroke directly, its opacity already
  // folded into the colour; a moving one averages it over the exposure.
  float coverage = annotation.sample_count == 0u
      ? annotation_edge(distance, feather)
      : annotation_shape_exposure(canvas_point, annotation, samples, feather);
  if (coverage <= 0.0) return rgba;
  float alpha = coverage * color.a;
  rgba.rgb = color.rgb * alpha + rgba.rgb * (1.0 - alpha);
  rgba.a = alpha + rgba.a * (1.0 - alpha);
  return rgba;
}

)METAL"
