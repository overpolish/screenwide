// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the pointer does around a text box: the double-click that opens it
//! for typing, the click that closes it, and the I-beam over it meanwhile.
//! The typing itself is in `+annotation_text.m`.

#import "recording_preview_surface_macos_private.h"

#include "recording_preview_annotation_layers_macos.h"

static NSPoint annotation_display_point(NSRect image, double x, double y) {
  return NSMakePoint(NSMinX(image) + image.size.width * x,
                     NSMinY(image) + image.size.height * y);
}

#include "recording_preview_annotation_geometry_macos.h"

SCREENWIDE_PREVIEW_PRIVATE BOOL annotation_text_press(
    ScreenwidePreviewInteractionView *view, NSEvent *event) {
  ScreenwidePreviewSurface *surface = view.surface;
  surface.annotationTextOpensAtPoint = NO;
  // The click that closes a box does nothing else: it drops no new box, and
  // a drag that follows it does not pan.
  if (annotation_text_editing(surface)) {
    annotation_text_finish(surface);
    view.annotationPressIgnored = YES;
    return YES;
  }
  if (event.buttonNumber != 0 || event.clickCount != 2 ||
      annotation_active_mode(surface) == ScreenwideAnnotationModeNone ||
      surface.annotationTextCallback == NULL)
    return NO;
  NSPoint point = [view convertPoint:event.locationInWindow fromView:nil];
  NSInteger shaft = annotation_shaft_at_point(surface, point);
  NSUInteger count = 0;
  const ScreenwidePreviewAnnotation *items = annotation_items(surface, &count);
  if (shaft < 0 || (NSUInteger)shaft >= count ||
      items[shaft].kind != ScreenwideAnnotationKindText)
    return NO;
  // The first click chose the box and may have armed a move; the second opens
  // it for typing instead, with the caret where it landed.
  view.annotationDragActive = NO;
  view.annotationDragPending = NO;
  view.annotationDragBegun = NO;
  view.annotationPressIgnored = YES;
  surface.annotationTextOpenPoint = point;
  surface.annotationTextOpensAtPoint = YES;
  uint32_t layer = items[shaft].layer_id >= 0 ? (uint32_t)items[shaft].layer_id
                                              : surface.selection.layer_id;
  surface.annotationTextCallback(0, layer, items[shaft].index, NULL, 0, 0,
                                 surface.annotationTextContext);
  return YES;
}

SCREENWIDE_PREVIEW_PRIVATE NSCursor *annotation_text_cursor(
    ScreenwidePreviewSurface *surface, NSPoint point) {
  NSUInteger count = 0;
  const ScreenwidePreviewAnnotation *items = annotation_items(surface, &count);
  NSInteger flat = surface.annotationTextIndex;
  if (!annotation_text_editing(surface) || flat < 0 || (NSUInteger)flat >= count)
    return nil;
  NSRect image = annotation_layer_image(surface, items[flat].layer_id);
  return annotation_shaft_distance(image, items[flat], point) <= 0.0 ? [NSCursor IBeamCursor]
                                                                     : nil;
}
