// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The rotation gizmo over the chosen image: its rings, and a drag of them
// that turns the image in depth and flat. The twin of the Windows overlay's
// `annotation/image_turn.rs`.
//
// The rings show the turn the image is seen at. On a layer that is turned
// itself, that is the image's own turn within the layer's, centred where the
// layer's turn carries the image's middle; a drag works the image's own turn
// back out of the turn the rings reach. Each sample is reported as the
// annotation gesture's turn target, which carries that turn in place of a
// point.

#import "recording_preview_surface_macos_private.h"
#include <math.h>

/// The selected layer's turn, in degrees, where it is turned.
static BOOL layer_turn(ScreenwidePreviewSurface *surface, double out[3]) {
  if (!surface.hasSelection || !selection_is_turned(surface.selection)) return NO;
  out[0] = surface.selection.rotation_x;
  out[1] = surface.selection.rotation_y;
  out[2] = surface.selection.rotation_z;
  return YES;
}

SCREENWIDE_PREVIEW_PRIVATE BOOL annotation_image_gizmo(ScreenwidePreviewSurface *surface,
                                                       ScreenwideImageGizmo *gizmo) {
  ScreenwideAnnotationMode mode = annotation_active_mode(surface);
  if ((mode != ScreenwideAnnotationModeSelect && mode != ScreenwideAnnotationModeImage) ||
      annotation_has_group(surface) || object_picking_active(surface))
    return NO;
  NSUInteger items = 0;
  const ScreenwidePreviewAnnotation *list = annotation_items(surface, &items);
  NSInteger selected = surface.annotationSelected;
  if (list == NULL || selected < 0 || (NSUInteger)selected >= items ||
      list[selected].kind != ScreenwideAnnotationKindImage)
    return NO;
  NSRect image = annotation_image_frame(surface);
  if (image.size.width <= 0.0 || image.size.height <= 0.0) return NO;
  ScreenwidePreviewAnnotation item = list[selected];
  NSPoint middle = NSMakePoint(NSMinX(image) + image.size.width * item.middle_x,
                               NSMinY(image) + image.size.height * item.middle_y);
  double storage[9];
  const double *turn = annotation_chrome_turn(surface, storage);
  ScreenwidePreviewInteractionView *view = surface.interaction;
  ScreenwideImageTurnHold hold = view.imageTurnHold;
  if (view.imageTurnActive && hold.index == selected) {
    memcpy(gizmo->own, hold.own, sizeof(gizmo->own));
    memcpy(gizmo->seen, hold.seen, sizeof(gizmo->seen));
  } else {
    // Its flat turn runs counter-clockwise as its angle runs clockwise.
    gizmo->own[0] = item.tilt_x;
    gizmo->own[1] = item.tilt_y;
    gizmo->own[2] = -item.start_head * 180.0 / M_PI;
    double layer[3];
    if (layer_turn(surface, layer)) screenwide_turn_compose(layer, gizmo->own, gizmo->seen);
    else memcpy(gizmo->seen, gizmo->own, sizeof(gizmo->seen));
  }
  gizmo->centre = turn != NULL ? turn_carry(turn, middle) : middle;
  // The half extents ride as shares of the drawn width.
  gizmo->radius =
      screenwide_turn_ring_radius(2.0 * MIN(item.end_head, item.width) * image.size.width);
  gizmo->index = selected;
  return YES;
}

SCREENWIDE_PREVIEW_PRIVATE ScreenwideTurnRingHit annotation_image_ring_at(
    ScreenwidePreviewSurface *surface, NSPoint point) {
  ScreenwideImageGizmo gizmo;
  if (!annotation_image_gizmo(surface, &gizmo)) return (ScreenwideTurnRingHit){-1, 0, 0.0};
  return screenwide_turn_ring_at(gizmo.seen[0], gizmo.seen[1], gizmo.seen[2], gizmo.radius,
                                 point.x - gizmo.centre.x, point.y - gizmo.centre.y);
}

/// Reports the held ring's sample: the image's own turn about X and Y as the
/// point, about Z as the picture's width.
static void emit_image_turn(ScreenwidePreviewSurface *surface, uint32_t phase,
                            ScreenwideImageTurnHold hold) {
  if (surface.annotationGestureCallback == NULL) return;
  NSUInteger items = 0;
  const ScreenwidePreviewAnnotation *list = annotation_items(surface, &items);
  uint32_t layer = surface.selection.layer_id;
  uint32_t index = 0;
  if (list != NULL && hold.index >= 0 && (NSUInteger)hold.index < items) {
    // An annotation is measured on its own layer; a still's carry none, and
    // the selection's is theirs.
    if (list[hold.index].layer_id >= 0) layer = (uint32_t)list[hold.index].layer_id;
    index = list[hold.index].index;
  }
  surface.annotationGestureCallback(phase, layer, ScreenwideAnnotationTargetTurn, index, 0,
                                    hold.own[0], hold.own[1], 0, hold.own[2],
                                    surface.annotationGestureContext);
}

SCREENWIDE_PREVIEW_PRIVATE BOOL image_turn_mouse_down(ScreenwidePreviewInteractionView *view,
                                                      NSEvent *event, NSPoint point) {
  ScreenwidePreviewSurface *surface = view.surface;
  ScreenwideImageGizmo gizmo;
  if (event.buttonNumber != 0 || surface.annotationGestureCallback == NULL ||
      !annotation_image_gizmo(surface, &gizmo))
    return NO;
  ScreenwideTurnRingHit hit =
      screenwide_turn_ring_at(gizmo.seen[0], gizmo.seen[1], gizmo.seen[2], gizmo.radius,
                              point.x - gizmo.centre.x, point.y - gizmo.centre.y);
  if (hit.axis < 0) return NO;
  ScreenwideImageTurnHold hold;
  hold.drag = screenwide_turn_ring_begin(gizmo.seen[0], gizmo.seen[1], gizmo.seen[2],
                                         gizmo.radius, (uint32_t)hit.axis, hit.angle);
  hold.index = gizmo.index;
  memcpy(hold.own, gizmo.own, sizeof(hold.own));
  memcpy(hold.seen, gizmo.seen, sizeof(hold.seen));
  hold.origin = point;
  hold.edges = hit.edges;
  view.imageTurnHold = hold;
  view.imageTurnActive = YES;
  view.selectionDragActive = NO;
  view.panning = NO;
  surface.turnHoveredRing = 0;
  emit_image_turn(surface, 0, hold);
  set_selection_cursor(screenwide_region_resize_cursor(hit.edges) ?: [NSCursor closedHandCursor]);
  redraw_selection(surface);
  return YES;
}

SCREENWIDE_PREVIEW_PRIVATE BOOL image_turn_mouse_dragged(ScreenwidePreviewInteractionView *view,
                                                         NSEvent *event) {
  if (!view.imageTurnActive) return NO;
  ScreenwidePreviewSurface *surface = view.surface;
  ScreenwideImageTurnHold hold = view.imageTurnHold;
  NSUInteger items = 0;
  const ScreenwidePreviewAnnotation *list = annotation_items(surface, &items);
  if (list == NULL || hold.index < 0 || (NSUInteger)hold.index >= items ||
      list[hold.index].kind != ScreenwideAnnotationKindImage) {
    // The image went away under the ring: the turn it reached goes with it.
    view.imageTurnActive = NO;
    emit_image_turn(surface, 3, hold);
    redraw_selection(surface);
    return YES;
  }
  NSPoint point = [view convertPoint:event.locationInWindow fromView:nil];
  ScreenwideTurnRingDrag drag = hold.drag;
  // Shift holds the dragged axis to whole steps, as it holds a layer's.
  screenwide_turn_ring_drag(&drag, point.x - hold.origin.x, point.y - hold.origin.y,
                            (event.modifierFlags & NSEventModifierFlagShift) != 0 ? 1 : 0,
                            hold.seen);
  hold.drag = drag;
  double layer[3];
  if (layer_turn(surface, layer)) {
    double near[3];
    memcpy(near, hold.own, sizeof(near));
    screenwide_turn_within(layer, hold.seen, near, hold.own);
  } else {
    memcpy(hold.own, hold.seen, sizeof(hold.own));
  }
  view.imageTurnHold = hold;
  redraw_selection(surface);
  emit_image_turn(surface, 1, hold);
  return YES;
}

SCREENWIDE_PREVIEW_PRIVATE BOOL image_turn_mouse_up(ScreenwidePreviewInteractionView *view,
                                                    NSEvent *event) {
  if (!view.imageTurnActive) return NO;
  // A release can land past the last drag sample; it is the turn kept.
  image_turn_mouse_dragged(view, event);
  if (!view.imageTurnActive) return YES;
  view.imageTurnActive = NO;
  ScreenwidePreviewSurface *surface = view.surface;
  emit_image_turn(surface, 2, view.imageTurnHold);
  redraw_selection(surface);
  set_selection_cursor_at_point(surface, [view convertPoint:event.locationInWindow fromView:nil]);
  return YES;
}
