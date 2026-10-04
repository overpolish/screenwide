// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a press lands on: the chosen annotation's grips, the topmost
//! annotation drawn under the pointer, and which part of it a press takes
//! hold of.

#import "recording_preview_surface_macos_private.h"
#include <math.h>
#include "recording_preview_annotation_layers_macos.h"

/// The grip's hit radius, matching the selection handles'.
static const CGFloat kAnnotationHandleHit = 8.0;

static NSPoint annotation_display_point(NSRect image, double x, double y) {
  return NSMakePoint(NSMinX(image) + image.size.width * x,
                     NSMinY(image) + image.size.height * y);
}

#include "recording_preview_annotation_geometry_macos.h"

/// Which grip a press on the body of the annotation at `index` takes hold
/// of: a magnifier's loupe is carried on its own, and everything else, a
/// magnifier's zoom area included, whole.
SCREENWIDE_PREVIEW_PRIVATE uint32_t annotation_body_handle(ScreenwidePreviewSurface *surface,
                                                           NSInteger index, NSPoint point) {
  NSUInteger count = 0;
  const ScreenwidePreviewAnnotation *items = annotation_items(surface, &count);
  if (items == NULL || index < 0 || (NSUInteger)index >= count ||
      items[index].kind != ScreenwideAnnotationKindMagnify)
    return ScreenwideAnnotationHandleBody;
  NSRect image = annotation_layer_image(surface, items[index].layer_id);
  AnnotationArrowGeometry prepared = annotation_prepared(image, items[index]);
  return screenwide_magnify_part(point.x, point.y, &prepared) == 1
             ? ScreenwideAnnotationHandleMiddle
             : ScreenwideAnnotationHandleBody;
}

/// Where a magnifier's loupe grip sits: on its rim towards its bottom-right
/// corner, where Rust's preparation placed it.
SCREENWIDE_PREVIEW_PRIVATE NSPoint annotation_magnify_grip(NSRect image,
                                                           ScreenwidePreviewAnnotation item) {
  AnnotationArrowGeometry prepared = annotation_prepared(image, item);
  return NSMakePoint(prepared.end_head.c.x, prepared.end_head.c.y);
}

/// Where a sticker's turning grip sits: above its top side, where Rust's
/// preparation placed it.
SCREENWIDE_PREVIEW_PRIVATE NSPoint annotation_sticker_turn_grip(NSRect image,
                                                                ScreenwidePreviewAnnotation item) {
  AnnotationArrowGeometry prepared = annotation_prepared(image, item);
  return NSMakePoint(prepared.end_head.c.x, prepared.end_head.c.y);
}

/// How far `point` is from a stroke's drawn line, in display points, from
/// the fitted line its grips point into; with `body`, its box picks it too.
/// A stroke whose line was not published is nowhere.
static double annotation_draw_distance(ScreenwidePreviewSurface *surface, NSRect image,
                                       ScreenwidePreviewAnnotation item, NSPoint point,
                                       BOOL body) {
  // The most points a fitted line has: `path::MAX_CURVES` curves, two points
  // each and its first.
  enum { MaxPoints = 2 * 256 + 1 };
  NSUInteger total = surface.annotationPaths.length / (2 * sizeof(float));
  NSUInteger first = (NSUInteger)item.start_head;
  NSUInteger count = (NSUInteger)item.end_head;
  if (count == 0 || count > MaxPoints || first > total || count > total - first) return INFINITY;
  const float *normalised = (const float *)surface.annotationPaths.bytes + first * 2;
  float placed[MaxPoints * 2];
  for (NSUInteger index = 0; index < count; index++) {
    NSPoint at = annotation_display_point(image, normalised[index * 2], normalised[index * 2 + 1]);
    placed[index * 2] = (float)at.x;
    placed[index * 2 + 1] = (float)at.y;
  }
  NSPoint low = annotation_display_point(image, item.start_x, item.start_y);
  NSPoint high = annotation_display_point(image, item.end_x, item.end_y);
  return screenwide_freehand_distance(point.x, point.y, placed, (uint32_t)count,
                                      (float)(item.width * image.size.width), body ? 1u : 0u,
                                      low.x, low.y, high.x, high.y);
}

// An arrow has three grips; a counter one, the tip of its tail; a text box
// one, its pointer's tip; a redaction, a shape or a spotlight the eight of
// its box; a magnifier those of its zoom area's box and one on its loupe; a
// sticker those of the box it fits in and one that turns it; a highlight the
// selection's two ends.
SCREENWIDE_PREVIEW_PRIVATE NSUInteger annotation_grips(
    NSRect image, ScreenwidePreviewAnnotation item, NSPoint *handles, uint32_t *kinds) {
  if (annotation_kind_is_box(item.kind))
    return annotation_redact_grips(image, item, handles, kinds);
  if (item.kind == ScreenwideAnnotationKindHighlight) {
    handles[0] = annotation_display_point(image, item.start_x,
                                          (item.start_y + item.start_head) * 0.5);
    handles[1] = annotation_display_point(image, item.end_x, (item.end_head + item.end_y) * 0.5);
    kinds[0] = ScreenwideAnnotationHandleStart;
    kinds[1] = ScreenwideAnnotationHandleEnd;
    return 2;
  }
  if (item.kind == ScreenwideAnnotationKindCounter || item.kind == ScreenwideAnnotationKindText) {
    handles[0] = item.kind == ScreenwideAnnotationKindText ? annotation_text_grip(image, item)
                                                           : annotation_counter_tail(image, item);
    kinds[0] = ScreenwideAnnotationHandleTail;
    return 1;
  }
  handles[0] = annotation_display_point(image, item.start_x, item.start_y);
  handles[1] = annotation_display_point(image, item.middle_x, item.middle_y);
  handles[2] = annotation_display_point(image, item.end_x, item.end_y);
  kinds[0] = ScreenwideAnnotationHandleStart;
  kinds[1] = ScreenwideAnnotationHandleMiddle;
  kinds[2] = ScreenwideAnnotationHandleEnd;
  return 3;
}

/// The chosen annotation's grip under `point`, or -1. The pen holds nothing,
/// so it offers none.
SCREENWIDE_PREVIEW_PRIVATE NSInteger annotation_handle_at_point(
    ScreenwidePreviewSurface *surface, NSPoint point) {
  NSUInteger count = 0;
  const ScreenwidePreviewAnnotation *items = annotation_items(surface, &count);
  if (items == NULL || surface.annotationSelected < 0 ||
      (NSUInteger)surface.annotationSelected >= count ||
      annotation_active_mode(surface) == ScreenwideAnnotationModeDraw)
    return -1;
  NSPoint handles[SCREENWIDE_ANNOTATION_MAX_GRIPS];
  uint32_t kinds[SCREENWIDE_ANNOTATION_MAX_GRIPS];
  NSUInteger grips = annotation_grips(annotation_image_frame(surface),
                                      items[surface.annotationSelected], handles, kinds);
  for (NSUInteger index = 0; index < grips; index++) {
    if (fabs(point.x - handles[index].x) <= kAnnotationHandleHit &&
        fabs(point.y - handles[index].y) <= kAnnotationHandleHit)
      return (NSInteger)kinds[index];
  }
  return -1;
}

/// The topmost arrow whose drawn shape `point` lands on, or -1, with no
/// tolerance: picked, and haloed, exactly where it is painted. A shape or a
/// stroke is also picked by its inside once chosen or under the select tool,
/// but what is drawn wins over an inside whatever the stacking, so only a
/// press on nothing drawn falls to the inside it lands in. The pen picks
/// nothing up, and a stroke is picked by the select tool alone: another
/// drawing tool that chose one would hand over to the pen.
SCREENWIDE_PREVIEW_PRIVATE NSInteger annotation_shaft_at_point(
    ScreenwidePreviewSurface *surface, NSPoint point) {
  NSUInteger count = 0;
  const ScreenwidePreviewAnnotation *items = annotation_items(surface, &count);
  ScreenwideAnnotationMode mode = annotation_active_mode(surface);
  if (items == NULL || mode == ScreenwideAnnotationModeDraw) return -1;
  BOOL selecting = mode == ScreenwideAnnotationModeSelect || mode == ScreenwideAnnotationModeMarquee;
  for (int insides = 0; insides < 2; insides++) {
    for (NSInteger index = (NSInteger)count - 1; index >= 0; index--) {
      ScreenwidePreviewAnnotation item = items[(NSUInteger)index];
      if (item.kind == ScreenwideAnnotationKindDraw && !selecting) continue;
      BOOL whole = (item.kind == ScreenwideAnnotationKindShape ||
                    item.kind == ScreenwideAnnotationKindDraw) &&
                   (selecting || index == surface.annotationSelected);
      if (insides && !whole) continue;
      NSRect image = annotation_layer_image(surface, item.layer_id);
      if (image.size.width <= 0.0 || image.size.height <= 0.0) continue;
      double distance = item.kind == ScreenwideAnnotationKindDraw
          ? annotation_draw_distance(surface, image, item, point, insides == 1)
          : insides ? annotation_shape_body_distance(image, item, point)
                    : annotation_shaft_distance(image, item, point);
      if (distance <= 0.0) return index;
    }
  }
  return -1;
}
