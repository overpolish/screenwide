// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "osc_gpu_macos.h"
#import "recording_preview_surface_macos_private.h"
#include <math.h>
#include "recording_preview_annotation_layers_macos.h"

/// A press has to travel this far before it draws an arrow rather than
/// clearing the choice: a click and a very short drag are the same gesture to
/// a hand, and neither should leave a stub behind.
static const CGFloat kAnnotationDragSlop = 3.0;
/// The grip's hit radius, matching the selection handles'.
static const CGFloat kAnnotationHandleHit = 8.0;

SCREENWIDE_PREVIEW_PRIVATE const ScreenwidePreviewAnnotation *annotation_items(
    ScreenwidePreviewSurface *surface, NSUInteger *count) {
  NSUInteger items = surface.annotations.length / sizeof(ScreenwidePreviewAnnotation);
  *count = items;
  return *count == 0 ? NULL : surface.annotations.bytes;
}

/// The whole source image's rectangle on screen, in this flipped view's
/// coordinates. Every normalised handle is placed inside it.
SCREENWIDE_PREVIEW_PRIVATE NSRect annotation_image_frame(
    ScreenwidePreviewSurface *surface) {
  NSUInteger count = 0;
  const ScreenwidePreviewAnnotation *items = annotation_items(surface, &count);
  NSInteger selected = surface.annotationSelected;
  int32_t layer = selected >= 0 && (NSUInteger)selected < count ? items[selected].layer_id : -1;
  return annotation_layer_image(surface, layer);
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

/// Whether a tool in hand makes a new annotation on empty picture.
SCREENWIDE_PREVIEW_PRIVATE BOOL annotation_drawing_mode(ScreenwideAnnotationMode mode) {
  return mode == ScreenwideAnnotationModeArrow ||
         mode == ScreenwideAnnotationModeCounter ||
         mode == ScreenwideAnnotationModeText;
}

/// The grips one annotation shows, in display points, and which grip each is.
/// An arrow has three; a counter one, the tip of its tail; a text box one,
/// its pointer's tip.
static NSUInteger annotation_grips(NSRect image, ScreenwidePreviewAnnotation item,
                                   NSPoint handles[3], ScreenwideAnnotationHandle kinds[3]) {
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

/// The chosen annotation's grip under `point`, or -1.
SCREENWIDE_PREVIEW_PRIVATE NSInteger annotation_handle_at_point(
    ScreenwidePreviewSurface *surface, NSPoint point) {
  NSUInteger count = 0;
  const ScreenwidePreviewAnnotation *items = annotation_items(surface, &count);
  if (items == NULL || surface.annotationSelected < 0 ||
      (NSUInteger)surface.annotationSelected >= count)
    return -1;
  NSPoint handles[3];
  ScreenwideAnnotationHandle kinds[3];
  NSUInteger grips = annotation_grips(annotation_image_frame(surface),
                                      items[surface.annotationSelected], handles, kinds);
  for (NSUInteger index = 0; index < grips; index++) {
    if (fabs(point.x - handles[index].x) <= kAnnotationHandleHit &&
        fabs(point.y - handles[index].y) <= kAnnotationHandleHit)
      return (NSInteger)kinds[index];
  }
  return -1;
}

/// The topmost arrow whose drawn shape `point` lands on, or -1. There is no
/// tolerance around it: the arrow is picked, and haloed, exactly where it is
/// painted, which is what keeps the halo off the space beside an annotation.
SCREENWIDE_PREVIEW_PRIVATE NSInteger annotation_shaft_at_point(
    ScreenwidePreviewSurface *surface, NSPoint point) {
  NSUInteger count = 0;
  const ScreenwidePreviewAnnotation *items = annotation_items(surface, &count);
  if (items == NULL) return -1;
  for (NSInteger index = (NSInteger)count - 1; index >= 0; index--) {
    NSRect image = annotation_layer_image(surface, items[index].layer_id);
    if (image.size.width <= 0.0 || image.size.height <= 0.0) continue;
    if (annotation_shaft_distance(image, items[(NSUInteger)index], point) <= 0.0)
      return index;
  }
  return -1;
}

/// Which snapping modifiers this sample was taken with: bit 0 Shift, bit 1
/// Command. Both are read at the moment the sample is reported rather than
/// latched at the press, so either can be taken and let go part way through a
/// drag.
static uint32_t annotation_snapped(void) {
  NSEventModifierFlags flags = [NSEvent modifierFlags];
  return ((flags & NSEventModifierFlagShift) != 0 ? 1u : 0u) |
         ((flags & NSEventModifierFlagCommand) != 0 ? 2u : 0u);
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
  uint32_t layer = surface.selection.layer_id;
  NSUInteger count = 0;
  const ScreenwidePreviewAnnotation *items = annotation_items(surface, &count);
  if ((targetKind == ScreenwideAnnotationTargetExisting || targetKind == ScreenwideAnnotationTargetSelect) && index < count) {
    if (items[index].layer_id >= 0) layer = (uint32_t)items[index].layer_id;
    index = items[index].index;
  }
  surface.annotationGestureCallback(phase, layer,
                                    targetKind, index, handle, x, y,
                                    annotation_snapped(),
                                    annotation_image_frame(surface).size.width,
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
  if (handle < 0 && shaft < 0 && !annotation_drawing_mode(mode)) {
    // Empty picture with only the select tool in hand: the arrow chrome lets
    // go, and the press carries on to the layer underneath. Recording
    // selection clears the annotation together with the new layer; screenshots
    // still publish their selected-image annotation document separately.
    NSUInteger count = 0;
    const ScreenwidePreviewAnnotation *items = annotation_items(surface, &count);
    NSInteger selected = surface.annotationSelected;
    if (selected >= 0 && (NSUInteger)selected < count && items[selected].layer_id < 0) {
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
    NSUInteger count = 0;
    const ScreenwidePreviewAnnotation *items = annotation_items(surface, &count);
    ScreenwidePreviewSelection target;
    if (annotation_layer_selection(surface, items[shaft].layer_id, &target)) {
      surface.selection = target;
      surface.hasSelection = YES;
    }
    view.annotationDragTargetKind = ScreenwideAnnotationTargetExisting;
    view.annotationDragIndex = (uint32_t)shaft;
    view.annotationDragHandle = ScreenwideAnnotationHandleBody;
    view.annotationDragPending = YES;
    emit_annotation_gesture(surface, 0, ScreenwideAnnotationTargetSelect,
                            (uint32_t)shaft, ScreenwideAnnotationHandleBody,
                            point);
    return YES;
  }
  // Empty picture: a new annotation. An arrow is drawn out, so it waits for the
  // press to prove a drag and a click leaves no stub behind. A counter and a
  // text box are dropped whole where the press lands, so they begin at once
  // and a click alone makes them; the drag that may follow carries them.
  view.annotationDragTargetKind = ScreenwideAnnotationTargetNew;
  view.annotationDragIndex = 0;
  if (mode == ScreenwideAnnotationModeCounter || mode == ScreenwideAnnotationModeText) {
    view.annotationDragHandle = ScreenwideAnnotationHandleBody;
    view.annotationDragBegun = YES;
    emit_annotation_gesture(surface, 0, ScreenwideAnnotationTargetNew, 0,
                            ScreenwideAnnotationHandleBody, point);
    return YES;
  }
  view.annotationDragHandle = ScreenwideAnnotationHandleEnd;
  view.annotationDragPending = YES;
  return YES;
}

SCREENWIDE_PREVIEW_PRIVATE BOOL annotation_mouse_dragged(
    ScreenwidePreviewInteractionView *view, NSPoint point) {
  ScreenwidePreviewSurface *surface = view.surface;
  if (view.annotationPressIgnored) return YES;
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
  if (view.annotationPressIgnored) {
    view.annotationPressIgnored = NO;
    return YES;
  }
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
  NSPoint handles[3];
  ScreenwideAnnotationHandle kinds[3];
  NSUInteger grips = annotation_grips(annotation_image_frame(surface),
                                      list[surface.annotationSelected], handles, kinds);
  // The same disc the selection OSC draws its corner grips with: a 4pt fill
  // with a one-device-pixel ring, snapped to a whole device pixel.
  CGFloat extent = 4.0 + 2.0 / scale;
  for (NSUInteger index = 0; index < grips; index++) {
    CGFloat x = round(handles[index].x * scale) / scale;
    CGFloat y = round(handles[index].y * scale) / scale;
    screenwide_region_osc_add_texture_quad(
        vertices, count, size,
        NSMakeRect(x - extent, y - extent, extent * 2.0, extent * 2.0),
        NSMakeRect(0.0, 0.0, 1.0, 1.0), 3);
  }
}
