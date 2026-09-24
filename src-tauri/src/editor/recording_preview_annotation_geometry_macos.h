// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#include "annotations/geometry.h"

/// One annotation prepared in display points, from the normalised record the
/// chrome was given. The shape is Rust's, so the grips and the halo cannot
/// drift from what the compositor draws.
static inline AnnotationArrowGeometry annotation_prepared(
    NSRect image, ScreenwidePreviewAnnotation item) {
  NSPoint start = annotation_display_point(image, item.start_x, item.start_y);
  NSPoint end = annotation_display_point(image, item.end_x, item.end_y);
  AnnotationArrowGeometry prepared;
  if (item.kind == ScreenwideAnnotationKindText) {
    // The pointer rides in `end` held against the box, never placed; the text
    // block's size rides in `middle` as shares of the drawn width and the
    // alignment in `start_head`. See the kind's own reading.
    screenwide_annotation_prepare(item.kind, start.x, start.y, item.end_x, item.end_y,
                                  item.middle_x * image.size.width,
                                  item.middle_y * image.size.width,
                                  item.width * image.size.width,
                                  (uint32_t)item.start_head & 3u,
                                  annotation_reveal_whole(), &prepared);
    return prepared;
  }
  // A middle handle is reported rather than the curve's control point, so the
  // control is taken back out of it here, in display points. A counter aims
  // its tail with `start_head`, an angle, which rides in the same slot.
  NSPoint middle = annotation_display_point(image, item.middle_x, item.middle_y);
  NSPoint control = item.kind == ScreenwideAnnotationKindCounter
      ? NSMakePoint(item.start_head, 0)
      : NSMakePoint(2.0 * middle.x - (start.x + end.x) / 2.0,
                    2.0 * middle.y - (start.y + end.y) / 2.0);
  uint32_t heads = item.start_head > 0 ? 2 : item.end_head > 0 ? 1 : 0;
  screenwide_annotation_prepare(item.kind, start.x, start.y, control.x, control.y,
                                end.x, end.y, item.width * image.size.width, heads,
                                annotation_reveal_whole(), &prepared);
  return prepared;
}

/// A counter's one grip: the far end of its tail, where the preparation
/// already placed it.
static inline NSPoint annotation_counter_tail(NSRect image,
                                       ScreenwidePreviewAnnotation item) {
  AnnotationArrowGeometry prepared = annotation_prepared(image, item);
  return NSMakePoint(prepared.b.x, prepared.b.y);
}

/// A text box's one grip: its pointer's tip, drawn out of the box or tucked
/// into it where it was left.
static inline NSPoint annotation_text_grip(NSRect image, ScreenwidePreviewAnnotation item) {
  AnnotationArrowGeometry prepared = annotation_prepared(image, item);
  return NSMakePoint(prepared.c.x, prepared.c.y);
}

/// Where a text box's text block sits on screen, and the type size it is set
/// at, in display points: what the text view the box is typed in covers.
static inline NSRect annotation_text_block(NSRect image, ScreenwidePreviewAnnotation item,
                                    CGFloat *font) {
  AnnotationArrowGeometry prepared = annotation_prepared(image, item);
  CGFloat width = item.middle_x * image.size.width;
  CGFloat height = item.middle_y * image.size.width;
  *font = prepared.width;
  return NSMakeRect(prepared.start_head.c.x - width * 0.5,
                    prepared.start_head.c.y - height * 0.5, width, height);
}

/// How far a point is from one annotation's drawn shape, in display points.
/// Zero anywhere the annotation is actually painted, because the tolerance is
/// measured from the stroke's edge rather than its centreline. A press on a
/// head is a press on the arrow - it is the part of it the hand aims at - and a
/// press on a counter's tail or a text box's pointer is a press on it. An
/// annotation half-way through drawing itself in is still picked by the whole
/// of what it will be: the hand aims at the annotation, not at the frame of it
/// that happens to be showing.
static inline double annotation_shaft_distance(NSRect image,
                                        ScreenwidePreviewAnnotation item,
                                        NSPoint point) {
  AnnotationArrowGeometry prepared = annotation_prepared(image, item);
  return screenwide_annotation_distance(item.kind, point.x, point.y, &prepared);
}
