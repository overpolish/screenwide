// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#include <math.h>
#include <stdint.h>

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

/// Restrict each head to its own half of the curve, leaving a visible shaft.
static inline float annotation_trim(AnnotationVector a, AnnotationVector b,
                                    AnnotationVector c, float length, int at_end) {
  float low = at_end ? 0.5f : 0, high = at_end ? 1 : 0.5f;
  AnnotationVector tip = at_end ? c : a;
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
static inline AnnotationArrowGeometry annotation_prepare_arrow(AnnotationVector a,
    AnnotationVector b, AnnotationVector c, float width, uint32_t head) {
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
  float length = result.width * 4, half_base = result.width * 2;
  result.rounding = result.width * 0.35f;
  if (head != 0) {
    result.high = annotation_trim(a, b, c, length, 1);
    result.end_head = annotation_prepare_head(c, annotation_bezier(a, b, c, result.high),
                                              half_base, result.rounding);
  }
  if (head == 2) {
    result.low = annotation_trim(a, b, c, length, 0);
    result.start_head = annotation_prepare_head(a, annotation_bezier(a, b, c, result.low),
                                                half_base, result.rounding);
  }
  return result;
}
