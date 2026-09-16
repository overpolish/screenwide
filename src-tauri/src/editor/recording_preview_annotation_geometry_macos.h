// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#include "annotations/geometry.h"

/// The control point behind a reported middle handle, so the shaft can be
/// sampled without asking Rust for the curve again.
static NSPoint annotation_control_point(NSPoint start, NSPoint middle,
                                        NSPoint end) {
  return NSMakePoint(2.0 * middle.x - (start.x + end.x) / 2.0,
                     2.0 * middle.y - (start.y + end.y) / 2.0);
}

static NSPoint annotation_curve_point(NSPoint start, NSPoint control,
                                      NSPoint end, double t) {
  double inverse = 1.0 - t;
  return NSMakePoint(
      inverse * inverse * start.x + 2.0 * inverse * t * control.x + t * t * end.x,
      inverse * inverse * start.y + 2.0 * inverse * t * control.y + t * t * end.y);
}

static double annotation_segment_distance(NSPoint point, NSPoint start,
                                          NSPoint end) {
  double dx = end.x - start.x;
  double dy = end.y - start.y;
  double length = dx * dx + dy * dy;
  double t = length <= 0.0
      ? 0.0
      : ((point.x - start.x) * dx + (point.y - start.y) * dy) / length;
  t = fmax(0.0, fmin(1.0, t));
  double nearestX = start.x + dx * t;
  double nearestY = start.y + dy * t;
  return hypot(point.x - nearestX, point.y - nearestY);
}

/// How far a point is from a triangle, in display points. Zero inside it.
static double annotation_triangle_distance(NSPoint point, NSPoint a, NSPoint b,
                                           NSPoint c) {
  double first = (b.x - a.x) * (point.y - a.y) - (b.y - a.y) * (point.x - a.x);
  double second = (c.x - b.x) * (point.y - b.y) - (c.y - b.y) * (point.x - b.x);
  double third = (a.x - c.x) * (point.y - c.y) - (a.y - c.y) * (point.x - c.x);
  if ((first >= 0.0 && second >= 0.0 && third >= 0.0) ||
      (first <= 0.0 && second <= 0.0 && third <= 0.0))
    return 0.0;
  return fmin(annotation_segment_distance(point, a, b),
              fmin(annotation_segment_distance(point, b, c),
                   annotation_segment_distance(point, c, a)));
}

static NSPoint annotation_native_point(AnnotationVector point) {
  return NSMakePoint(point.x, point.y);
}

/// The same rounded triangle the compositor draws, in display points.
static double annotation_prepared_head_distance(NSPoint point,
    AnnotationTriangle triangle, float rounding) {
  return fmax(0, annotation_triangle_distance(point,
      annotation_native_point(triangle.a), annotation_native_point(triangle.b),
      annotation_native_point(triangle.c)) - rounding);
}

/// How far a point is from one arrow, in display points: its shaft, and the
/// heads on it. A press on a head is a press on the arrow - it is the part of
/// it the hand aims at. A mark half-way through drawing itself in is still
/// picked by the whole of what it will be: the hand aims at the arrow, not at
/// the frame of it that happens to be showing.
static double annotation_shaft_distance(NSRect image,
                                        ScreenwidePreviewAnnotation item,
                                        NSPoint point) {
  NSPoint start = annotation_display_point(image, item.start_x, item.start_y);
  NSPoint middle = annotation_display_point(image, item.middle_x, item.middle_y);
  NSPoint end = annotation_display_point(image, item.end_x, item.end_y);
  NSPoint control = annotation_control_point(start, middle, end);
  double best = INFINITY;
  NSPoint previous = start;
  for (NSUInteger sample = 1; sample <= kAnnotationShaftSamples; sample++) {
    NSPoint next = annotation_curve_point(
        start, control, end, (double)sample / (double)kAnnotationShaftSamples);
    best = fmin(best, annotation_segment_distance(point, previous, next));
    previous = next;
  }
  float width = fmax(item.start_head, item.end_head) * image.size.width / 4.0;
  uint32_t heads = item.start_head > 0 ? 2 : item.end_head > 0 ? 1 : 0;
  AnnotationArrowGeometry geometry = annotation_prepare_arrow(
      annotation_vector(start.x, start.y), annotation_vector(control.x, control.y),
      annotation_vector(end.x, end.y), width, heads, annotation_reveal_whole());
  if (geometry.head != 0)
    best = fmin(best, annotation_prepared_head_distance(point, geometry.end_head, geometry.rounding));
  if (geometry.head == 2)
    best = fmin(best, annotation_prepared_head_distance(point, geometry.start_head, geometry.rounding));
  return best;
}

