// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#include "annotations/geometry.h"

/// One annotation prepared in display points, from the normalised record the
/// chrome was given. The shape is Rust's, so the grips and the halo cannot
/// drift from what the compositor draws.
static AnnotationArrowGeometry annotation_prepared(
    NSRect image, ScreenwidePreviewAnnotation item) {
  NSPoint start = annotation_display_point(image, item.start_x, item.start_y);
  NSPoint end = annotation_display_point(image, item.end_x, item.end_y);
  // A middle handle is reported rather than the curve's control point, so the
  // control is taken back out of it here, in display points. A counter aims
  // its tail with `start_head`, an angle, which rides in the same slot.
  NSPoint middle = annotation_display_point(image, item.middle_x, item.middle_y);
  NSPoint control = item.kind == ScreenwideAnnotationKindCounter
      ? NSMakePoint(item.start_head, 0)
      : NSMakePoint(2.0 * middle.x - (start.x + end.x) / 2.0,
                    2.0 * middle.y - (start.y + end.y) / 2.0);
  uint32_t heads = item.start_head > 0 ? 2 : item.end_head > 0 ? 1 : 0;
  AnnotationArrowGeometry prepared;
  screenwide_annotation_prepare(item.kind, start.x, start.y, control.x, control.y,
                                end.x, end.y, item.width * image.size.width, heads,
                                annotation_reveal_whole(), &prepared);
  return prepared;
}

/// A counter's one grip: the far end of its tail, where the preparation
/// already placed it.
static NSPoint annotation_counter_tail(NSRect image,
                                       ScreenwidePreviewAnnotation item) {
  AnnotationArrowGeometry prepared = annotation_prepared(image, item);
  return NSMakePoint(prepared.b.x, prepared.b.y);
}

/// How far a point is from one annotation's drawn shape, in display points.
/// Zero anywhere the annotation is actually painted, because the tolerance is
/// measured from the stroke's edge rather than its centreline. A press on a
/// head is a press on the arrow - it is the part of it the hand aims at - and a
/// press on a counter's tail is a press on the counter. An annotation half-way
/// through drawing itself in is still picked by the whole of what it will be:
/// the hand aims at the annotation, not at the frame of it that happens to be
/// showing.
static double annotation_shaft_distance(NSRect image,
                                        ScreenwidePreviewAnnotation item,
                                        NSPoint point) {
  AnnotationArrowGeometry prepared = annotation_prepared(image, item);
  return screenwide_annotation_distance(item.kind, point.x, point.y, &prepared);
}
