// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "osc_gpu_macos.h"
#import "recording_preview_surface_macos_private.h"
#include <math.h>

/// A press has to travel this far before it draws an arrow rather than
/// clearing the choice: a click and a very short drag are the same gesture to
/// a hand, and neither should leave a stub behind.
static const CGFloat kAnnotationDragSlop = 3.0;
/// The grip's hit radius, matching the selection handles'.
static const CGFloat kAnnotationHandleHit = 8.0;
/// How many points the shaft is sampled at for hit testing. The curve is one
/// quadratic segment, so this is comfortably finer than a fingertip.
static const NSUInteger kAnnotationShaftSamples = 24;

static const ScreenwidePreviewAnnotation *annotation_items(
    ScreenwidePreviewSurface *surface, NSUInteger *count) {
  NSUInteger items = surface.annotations.length / sizeof(ScreenwidePreviewAnnotation);
  *count = MIN(items, ScreenwideMaxAnnotations);
  return *count == 0 ? NULL : surface.annotations.bytes;
}

/// The whole source image's rectangle on screen, in this flipped view's
/// coordinates. Every normalised handle is placed inside it.
SCREENWIDE_PREVIEW_PRIVATE NSRect annotation_image_frame(
    ScreenwidePreviewSurface *surface) {
  return selection_image_frame_for(surface, surface.selection);
}

static NSPoint annotation_display_point(NSRect image, double x, double y) {
  return NSMakePoint(NSMinX(image) + image.size.width * x,
                     NSMinY(image) + image.size.height * y);
}

/// The inverse: a point on screen, back in the layer's image-normalised
/// space. This is exactly what Rust turns into source pixels.
static BOOL annotation_normalised_point(ScreenwidePreviewSurface *surface,
                                        NSPoint point, double *x, double *y) {
  NSRect image = annotation_image_frame(surface);
  if (image.size.width <= 0.0 || image.size.height <= 0.0) return NO;
  *x = (point.x - NSMinX(image)) / image.size.width;
  *y = (point.y - NSMinY(image)) / image.size.height;
  return YES;
}

#include "recording_preview_annotation_geometry_macos.h"

/// The chosen arrow's grip under `point`, or -1.
SCREENWIDE_PREVIEW_PRIVATE NSInteger annotation_handle_at_point(
    ScreenwidePreviewSurface *surface, NSPoint point) {
  NSUInteger count = 0;
  const ScreenwidePreviewAnnotation *items = annotation_items(surface, &count);
  if (items == NULL || surface.annotationSelected < 0 ||
      (NSUInteger)surface.annotationSelected >= count)
    return -1;
  ScreenwidePreviewAnnotation item = items[surface.annotationSelected];
  NSRect image = annotation_image_frame(surface);
  NSPoint handles[3] = {
      annotation_display_point(image, item.start_x, item.start_y),
      annotation_display_point(image, item.middle_x, item.middle_y),
      annotation_display_point(image, item.end_x, item.end_y),
  };
  static const ScreenwideAnnotationHandle kinds[3] = {
      ScreenwideAnnotationHandleStart, ScreenwideAnnotationHandleMiddle,
      ScreenwideAnnotationHandleEnd};
  for (NSUInteger index = 0; index < 3; index++) {
    if (fabs(point.x - handles[index].x) <= kAnnotationHandleHit &&
        fabs(point.y - handles[index].y) <= kAnnotationHandleHit)
      return (NSInteger)kinds[index];
  }
  return -1;
}

/// The topmost arrow whose shaft `point` lands on, or -1. The tolerance is a
/// comfortable pointing target rather than the stroke's own width: the width
/// is in output pixels, which this side deliberately knows nothing about.
SCREENWIDE_PREVIEW_PRIVATE NSInteger annotation_shaft_at_point(
    ScreenwidePreviewSurface *surface, NSPoint point) {
  NSUInteger count = 0;
  const ScreenwidePreviewAnnotation *items = annotation_items(surface, &count);
  if (items == NULL) return -1;
  NSRect image = annotation_image_frame(surface);
  if (image.size.width <= 0.0 || image.size.height <= 0.0) return -1;
  for (NSInteger index = (NSInteger)count - 1; index >= 0; index--) {
    if (annotation_shaft_distance(image, items[(NSUInteger)index], point) <= 6.0)
      return index;
  }
  return -1;
}

static void emit_annotation_gesture(ScreenwidePreviewSurface *surface,
                                    uint32_t phase, uint32_t targetKind,
                                    uint32_t index, uint32_t handle,
                                    NSPoint point) {
  if (surface.annotationGestureCallback == NULL) return;
  double x = 0.0;
  double y = 0.0;
  if (!annotation_normalised_point(surface, point, &x, &y)) return;
  // The macOS workspace draws every layer into one pane, so the selection's
  // `pane_index` is always 0 and `layer_id` carries the workspace order the
  // gesture addresses - the same identity `emit_selection_gesture` reports.
  surface.annotationGestureCallback(phase, surface.selection.layer_id,
                                    targetKind, index, handle, x, y,
                                    surface.annotationGestureContext);
}

SCREENWIDE_PREVIEW_PRIVATE BOOL annotation_mouse_down(
    ScreenwidePreviewInteractionView *view, NSPoint point) {
  ScreenwidePreviewSurface *surface = view.surface;
  ScreenwideAnnotationMode mode = annotation_active_mode(surface);
  if (mode == ScreenwideAnnotationModeNone) return NO;
  // A press is never a hover: the halo goes out before anything moves.
  annotation_update_hover(surface, point, YES);
  NSInteger handle = annotation_handle_at_point(surface, point);
  NSInteger shaft = handle >= 0 ? -1 : annotation_shaft_at_point(surface, point);
  if (handle < 0 && shaft < 0 && mode != ScreenwideAnnotationModeArrow) {
    // Empty picture with only the select tool in hand: the arrow chrome lets
    // go, and the press carries on to the layer underneath.
    if (surface.annotationSelected >= 0) {
      surface.annotationSelected = -1;
      emit_annotation_gesture(surface, 0, ScreenwideAnnotationTargetNone, 0,
                              ScreenwideAnnotationHandleBody, point);
    }
    return NO;
  }
  view.panning = NO;
  view.selectionDragActive = NO;
  view.annotationDragActive = YES;
  view.annotationDragBegun = NO;
  view.annotationDragPending = NO;
  view.annotationDragOrigin = point;
  if (handle >= 0) {
    // A grip waits for the press to travel before it begins, exactly as a new
    // arrow does: a click that lands on a handle must not nudge the arrow by
    // the few points between the handle's centre and the pointer, nor leave an
    // edit in the history for it.
    view.annotationDragTargetKind = ScreenwideAnnotationTargetExisting;
    view.annotationDragIndex = (uint32_t)surface.annotationSelected;
    view.annotationDragHandle = (uint32_t)handle;
    view.annotationDragPending = YES;
    return YES;
  }
  if (shaft >= 0) {
    // Choosing an arrow is complete on the press: Rust commits the choice and
    // the OSC moves to it. The press may still turn into a move of the whole
    // arrow, which begins as its own gesture once it has travelled - exactly
    // as a grip does, so a click never leaves an edit in the history.
    surface.annotationSelected = shaft;
    view.annotationDragTargetKind = ScreenwideAnnotationTargetExisting;
    view.annotationDragIndex = (uint32_t)shaft;
    view.annotationDragHandle = ScreenwideAnnotationHandleBody;
    view.annotationDragPending = YES;
    emit_annotation_gesture(surface, 0, ScreenwideAnnotationTargetSelect,
                            (uint32_t)shaft, ScreenwideAnnotationHandleBody,
                            point);
    return YES;
  }
  // Empty picture: a new arrow, once the press proves to be a drag.
  view.annotationDragTargetKind = ScreenwideAnnotationTargetNew;
  view.annotationDragIndex = 0;
  view.annotationDragHandle = ScreenwideAnnotationHandleEnd;
  view.annotationDragPending = YES;
  return YES;
}

SCREENWIDE_PREVIEW_PRIVATE BOOL annotation_mouse_dragged(
    ScreenwidePreviewInteractionView *view, NSPoint point) {
  ScreenwidePreviewSurface *surface = view.surface;
  if (!view.annotationDragActive) return NO;
  if (view.annotationDragPending) {
    if (hypot(point.x - view.annotationDragOrigin.x,
              point.y - view.annotationDragOrigin.y) < kAnnotationDragSlop)
      return YES;
    view.annotationDragPending = NO;
    view.annotationDragBegun = YES;
    emit_annotation_gesture(surface, 0, view.annotationDragTargetKind,
                            view.annotationDragIndex, view.annotationDragHandle,
                            view.annotationDragOrigin);
  }
  emit_annotation_gesture(surface, 1, view.annotationDragTargetKind,
                          view.annotationDragIndex, view.annotationDragHandle,
                          point);
  return YES;
}

SCREENWIDE_PREVIEW_PRIVATE BOOL annotation_mouse_up(
    ScreenwidePreviewInteractionView *view, NSPoint point) {
  ScreenwidePreviewSurface *surface = view.surface;
  if (!view.annotationDragActive) return NO;
  BOOL begun = view.annotationDragBegun;
  if (begun)
    emit_annotation_gesture(surface, 2, view.annotationDragTargetKind,
                            view.annotationDragIndex, view.annotationDragHandle,
                            point);
  view.annotationDragActive = NO;
  view.annotationDragPending = NO;
  view.annotationDragBegun = NO;
  return YES;
}

SCREENWIDE_PREVIEW_PRIVATE void annotation_add_osc(
    ScreenwideRegionOscVertex *vertices, NSUInteger *count, NSSize size,
    ScreenwidePreviewSurface *surface, CGFloat scale) {
  NSUInteger items = 0;
  const ScreenwidePreviewAnnotation *list = annotation_items(surface, &items);
  if (list == NULL || surface.annotationSelected < 0 ||
      (NSUInteger)surface.annotationSelected >= items)
    return;
  ScreenwidePreviewAnnotation item = list[surface.annotationSelected];
  NSRect image = annotation_image_frame(surface);
  NSPoint handles[3] = {
      annotation_display_point(image, item.start_x, item.start_y),
      annotation_display_point(image, item.middle_x, item.middle_y),
      annotation_display_point(image, item.end_x, item.end_y),
  };
  // The same disc the selection OSC draws its corner grips with: a 4pt fill
  // with a one-device-pixel ring, snapped to a whole device pixel.
  CGFloat extent = 4.0 + 2.0 / scale;
  for (NSUInteger index = 0; index < 3; index++) {
    CGFloat x = round(handles[index].x * scale) / scale;
    CGFloat y = round(handles[index].y * scale) / scale;
    screenwide_region_osc_add_texture_quad(
        vertices, count, size,
        NSMakeRect(x - extent, y - extent, extent * 2.0, extent * 2.0),
        NSMakeRect(0.0, 0.0, 1.0, 1.0), 3);
  }
}
