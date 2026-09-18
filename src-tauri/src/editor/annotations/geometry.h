// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#include <math.h>
#include <stdint.h>
#include "reveal.h"

/// Prepared in the caller's pixel space. Plain float pairs keep the same
/// four-byte alignment in C and Metal's packed_float2.
typedef struct { float x, y; } AnnotationVector;
typedef struct { AnnotationVector a, b, c; } AnnotationTriangle;
typedef struct {
  AnnotationVector a, b, c;
  float width, low, high;
  AnnotationTriangle start_head, end_head;
  float rounding;
  uint32_t head;
} AnnotationArrowGeometry;
_Static_assert(sizeof(AnnotationArrowGeometry) == 92, "Prepared arrow ABI");

static inline AnnotationVector annotation_vector(float x, float y) {
  return (AnnotationVector){x, y};
}
static inline AnnotationVector annotation_add(AnnotationVector a, AnnotationVector b) {
  return annotation_vector(a.x + b.x, a.y + b.y);
}
static inline AnnotationVector annotation_subtract(AnnotationVector a, AnnotationVector b) {
  return annotation_vector(a.x - b.x, a.y - b.y);
}
static inline AnnotationVector annotation_scale(AnnotationVector a, float scale) {
  return annotation_vector(a.x * scale, a.y * scale);
}
static inline float annotation_length(AnnotationVector a) {
  return sqrtf(a.x * a.x + a.y * a.y);
}
static inline float annotation_distance(AnnotationVector a, AnnotationVector b) {
  return annotation_length(annotation_subtract(a, b));
}
static inline AnnotationVector annotation_bezier(AnnotationVector a, AnnotationVector b,
                                                 AnnotationVector c, float t) {
  AnnotationVector leg = annotation_subtract(b, a);
  AnnotationVector bend = annotation_add(annotation_subtract(a, annotation_scale(b, 2)), c);
  return annotation_add(a, annotation_scale(annotation_add(annotation_scale(leg, 2),
                                                           annotation_scale(bend, t)), t));
}

/// Where the shaft has to stop for a head of `length` to meet it: the
/// parameter in `[low, high]` at which the curve leaves `tip` by that much.
/// A static arrow searches its own half of the curve, so a head can never eat
/// the whole shaft; a revealing one searches the window it is showing, where
/// a head legitimately fills everything there is.
static inline float annotation_trim(AnnotationVector a, AnnotationVector b,
                                    AnnotationVector c, AnnotationVector tip,
                                    float length, int at_end, float low, float high) {
  float limit = length * length;
  for (int step = 0; step < 10; step++) {
    float t = (low + high) * 0.5f;
    AnnotationVector offset = annotation_subtract(annotation_bezier(a, b, c, t), tip);
    if (((offset.x * offset.x + offset.y * offset.y) <= limit) == at_end) high = t;
    else low = t;
  }
  return at_end ? high : low;
}

/// Shrink the rounded head to its inner triangle. Distance to these vertices
/// minus rounding is the final outline; both picking and Metal use it.
/// The apex extension compensates for rounding so the visible tip stays put.
static inline AnnotationTriangle annotation_prepare_head(AnnotationVector tip,
    AnnotationVector join, float half_base, float rounding) {
  AnnotationVector delta = annotation_subtract(tip, join);
  float length = annotation_length(delta);
  AnnotationVector direction = length > 1e-6f ? annotation_scale(delta, 1 / length)
                                               : annotation_vector(1, 0);
  float k = fminf(fmaxf(rounding / fmaxf(half_base, 1e-4f), 0), 0.9f);
  float span = length - rounding, complement = 1 - k * k;
  float height = (span + k * sqrtf(span * span + half_base * half_base * complement)) /
      fmaxf(complement, 1e-6f);
  AnnotationVector apex = annotation_add(tip, annotation_scale(direction, fmaxf(height - length, 0)));
  AnnotationVector across = annotation_vector(-direction.y * half_base, direction.x * half_base);
  AnnotationVector left = annotation_add(join, across), right = annotation_subtract(join, across);
  float side_apex = annotation_distance(left, right);
  float side_left = annotation_distance(apex, right), side_right = annotation_distance(apex, left);
  float perimeter = fmaxf(side_apex + side_left + side_right, 1e-6f);
  AnnotationVector incentre = annotation_scale(annotation_add(annotation_add(
      annotation_scale(apex, side_apex), annotation_scale(left, side_left)),
      annotation_scale(right, side_right)), 1 / perimeter);
  float area = 0.5f * fabsf((left.x - apex.x) * (right.y - apex.y) -
                           (right.x - apex.x) * (left.y - apex.y));
  float inradius = 2 * area / perimeter;
  float shrink = fmaxf(inradius - rounding, 0) / fmaxf(inradius, 1e-6f);
  return (AnnotationTriangle){
    annotation_add(incentre, annotation_scale(annotation_subtract(apex, incentre), shrink)),
    annotation_add(incentre, annotation_scale(annotation_subtract(left, incentre), shrink)),
    annotation_add(incentre, annotation_scale(annotation_subtract(right, incentre), shrink))};
}

/// One preparation per mark before drawing or picking, never per pixel.
/// A short curve scales the stroke and heads together. Sampling each arm
/// also preserves folded curves whose midpoint happens to coincide with a tip.
///
/// `reveal` is how much of the mark this frame draws. The whole path at full
/// size prepares exactly what it always has: the head sits on the end point,
/// facing the way the curve leaves the control, at full size. A mark part way
/// through its clip wears each head on its end of the shaft, ahead of it -
/// riding the end that is moving, growing out of its base as the mark sets
/// off and back into it as the mark leaves.
static inline AnnotationArrowGeometry annotation_prepare_arrow(AnnotationVector a,
    AnnotationVector b, AnnotationVector c, float width, uint32_t head,
    AnnotationReveal reveal) {
  AnnotationArrowGeometry result = {.a = a, .b = b, .c = c, .width = fmaxf(width, 0),
                                    .low = 0, .high = 1, .head = head};
  AnnotationVector middle = annotation_bezier(a, b, c, 0.5f);
  if (head != 0) {
    float room = fmaxf(annotation_distance(middle, c),
        annotation_distance(annotation_bezier(a, b, c, 0.75f), c));
    result.width = fminf(result.width, room * 0.8f / 4);
  }
  if (head == 2) {
    float room = fmaxf(annotation_distance(middle, a),
        annotation_distance(annotation_bezier(a, b, c, 0.25f), a));
    result.width = fminf(result.width, room * 0.8f / 4);
  }
  // Coincident handles draw a round dot, avoiding a degenerate triangle SDF.
  if (result.width <= 1e-4f) {
    result.width = fmaxf(width, 0);
    result.head = 0;
    return result;
  }
  // Geometry is prepared once per exposure sample, not per pixel.
  AnnotationRevealGeometry travel;
  screenwide_annotation_reveal_geometry(a.x, a.y, b.x, b.y, c.x, c.y,
                                        result.width, (float)head, reveal, &travel);
  int revealing = travel.low > 0 || travel.high < 1 || travel.scale < 1;
  float scale = travel.scale;
  float length = result.width * 4 * scale, half_base = result.width * 2 * scale;
  result.rounding = result.width * 0.35f * scale;
  // Stroke, heads and rounding are one mark and scale together. A shaft
  // shorter than the stroke is wide draws its own round cap, so a full-width
  // stroke on a mark two pixels long appears as a disc the width of the mark:
  // weight has to arrive with the rest of it.
  result.width *= scale;
  result.low = travel.low;
  result.high = travel.high;
  // A head the reveal has not grown to half a pixel yet has no triangle worth
  // building: its three vertices collapse onto each other, and a triangle
  // with no winding reads as inside everywhere.
  if (revealing && length <= 0.5f) {
    result.head = 0;
    return result;
  }
  // A growing head narrows fast, so while a mark is revealing, the shaft
  // runs on a little way under the head rather than stopping at its base:
  // met exactly, a bend pokes out through the head's narrowing sides. The
  // head itself is built the same way throughout, so nothing jumps when the
  // reveal holds.
  float pullback = 0.88f;
  if (head != 0) {
    AnnotationVector tip = revealing ? annotation_bezier(a, b, c, travel.end_tip) : c;
    float join = revealing ? travel.high
                           : annotation_trim(a, b, c, tip, length, 1, 0.5f, 1);
    result.end_head = annotation_prepare_head(tip, annotation_bezier(a, b, c, join),
                                              half_base, result.rounding);
    result.high = revealing
        ? annotation_trim(a, b, c, tip, length * pullback, 1, travel.high, travel.end_tip)
        : join;
  }
  if (head == 2) {
    AnnotationVector tip = revealing ? annotation_bezier(a, b, c, travel.start_tip) : a;
    float join = revealing ? travel.low
                           : annotation_trim(a, b, c, tip, length, 0, 0, 0.5f);
    result.start_head = annotation_prepare_head(tip, annotation_bezier(a, b, c, join),
                                                half_base, result.rounding);
    result.low = revealing
        ? annotation_trim(a, b, c, tip, length * pullback, 0, travel.start_tip, travel.low)
        : join;
  }
  return result;
}

/// How far a counter's tail reaches from the disc's centre, in radii, and how
/// round its tip is as a share of the radius. The twins of
/// `COUNTER_TAIL_REACH` and `COUNTER_TIP_SHARE` in `counter.rs`.
#define ANNOTATION_COUNTER_TAIL_REACH 1.5f
#define ANNOTATION_COUNTER_TIP_SHARE 0.125f

/// One counter prepared for drawing and picking, read out of the slots an
/// arrow fills with its curve. `a` is the disc's centre and `b` the tail's
/// tip; `rounding` is the disc's radius, `low` the radius the tip is rounded
/// to, and `width` the disc's diameter, so the bounding box an arrow's stroke
/// drives is also a counter's. Where the number is rasterised - its origin
/// and size in the text atlas - rides in `start_head`.
typedef struct {
  AnnotationVector center, tip;
  float radius, tip_radius;
  AnnotationVector text_origin, text_size;
} AnnotationCounterGeometry;

static inline AnnotationCounterGeometry annotation_counter_geometry(
    AnnotationArrowGeometry prepared) {
  return (AnnotationCounterGeometry){
      .center = prepared.a,
      .tip = prepared.b,
      .radius = prepared.rounding,
      .tip_radius = prepared.low,
      .text_origin = prepared.start_head.a,
      .text_size = prepared.start_head.b,
  };
}

/// One preparation per counter before drawing or picking, never per pixel.
///
/// The silhouette is the disc unioned with the tail: the overlap of two
/// circles, one either side of the axis, each tangent to the disc and to the
/// little circle the tip is rounded to, with their centres a radius behind the
/// disc's own. That is the outline of `MapPinPlusInside` to within a
/// hundredth of the radius.
///
/// `reveal.scale` is the size the mark is drawn at this frame: the disc, its
/// tail and its number are one mark and grow together, so the scale is
/// applied here, once, and the number follows the radius it lands on.
static inline AnnotationArrowGeometry annotation_prepare_counter(
    AnnotationVector center, float diameter, float angle, AnnotationReveal reveal) {
  float scale = fmaxf(fminf(reveal.scale, 1.0f), 0.0f);
  float radius = fmaxf(diameter, 0) * 0.5f * scale;
  AnnotationArrowGeometry result = {.a = center, .b = center, .c = center,
                                    .width = radius * 2, .low = 0, .high = 1,
                                    .rounding = radius, .head = 0};
  if (radius <= 0) return result;
  result.low = radius * ANNOTATION_COUNTER_TIP_SHARE;
  float reach = radius * ANNOTATION_COUNTER_TAIL_REACH;
  AnnotationVector direction = annotation_vector(cosf(angle), sinf(angle));
  result.b = annotation_add(center, annotation_scale(direction, reach));
  return result;
}

/// The far end of a prepared counter's tail: the point its grip is drawn on,
/// which the preparation already placed at the tail's full reach.
static inline AnnotationVector annotation_counter_tail_point(
    AnnotationArrowGeometry prepared) {
  return prepared.b;
}

/// How far a point falls outside the silhouette, measured `along` the tail and
/// `across` it from the disc's centre, with `across` already folded to the
/// near side. The twin of `counter_silhouette_distance` in
/// `counter_silhouette.rs`.
///
/// The side circles are solved rather than chosen: a radius behind the disc's
/// centre, tangent to the disc and tangent to the tip's own circle, leaves one
/// radius and one distance across the axis to find. Their overlap is measured
/// the way any lens is - to the near circle inside its span, to the shared
/// corner past it - and that corner is the tip's centre, so taking the tip's
/// radius off the whole thing rounds the point without moving it.
///
/// The overlap runs back behind the disc as well, and is wider than the disc
/// where it does, so it is cut at the plane where the circles touch the disc.
/// That cut is a chord of the disc, inside the union, and so never shows.
static inline double annotation_counter_silhouette_distance(
    double across, double along, double radius, double tip_radius, double reach) {
  double disc = hypot(across, along) - radius;
  double gap = radius - tip_radius;
  if (gap <= 0) return disc;
  // Where the tip's circle sits, and the side circles that reach it.
  double tip = reach - tip_radius;
  double side = (tip * tip + 2 * tip * radius + gap * gap) / (2 * gap);
  double apart = side + tip_radius - radius;
  double offset = apart * apart - radius * radius;
  if (offset <= 0) return disc;
  offset = sqrt(offset);
  double lens = (along - tip) * offset > across * (tip + radius)
                    ? hypot(across, along - tip)
                    : hypot(across + offset, along + radius) - side;
  double touch = radius * radius / apart;
  return fmin(disc, fmax(lens - tip_radius, touch - along));
}

/// How far `point` falls outside a prepared counter's silhouette, in the
/// space it was prepared in. Negative inside the mark, which is what picks
/// it. The twin of `counter_distance` in `counter.rs`.
static inline double annotation_counter_distance(AnnotationVector point,
                                                 AnnotationArrowGeometry prepared) {
  AnnotationCounterGeometry counter = annotation_counter_geometry(prepared);
  double local_x = point.x - counter.center.x;
  double local_y = point.y - counter.center.y;
  if (counter.radius <= 0) return hypot(local_x, local_y);
  double reach = hypot(counter.tip.x - counter.center.x, counter.tip.y - counter.center.y);
  if (reach <= 0) return hypot(local_x, local_y) - counter.radius;
  double axis_x = (counter.tip.x - counter.center.x) / reach;
  double axis_y = (counter.tip.y - counter.center.y) / reach;
  double along = local_x * axis_x + local_y * axis_y;
  double across = fabs(local_x * -axis_y + local_y * axis_x);
  return annotation_counter_silhouette_distance(across, along, counter.radius,
                                                counter.tip_radius, reach);
}
