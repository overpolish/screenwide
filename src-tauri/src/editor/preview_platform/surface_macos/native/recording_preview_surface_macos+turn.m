// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The rotation gizmo over a layer a turn tips, and that layer's own chrome
// placed through its turn. The chrome is laid out as for the upright layer,
// in display points, and its corners and grips carried through the turn the
// compositor draws the layer with, then drawn in soft lines; a press on it
// is carried back the same way. The geometry is Rust's (`osc/src/turn.rs`),
// so the rings, the chrome and the picture cannot drift apart. The chosen
// image's rings are `+annotation_image_turn.m`'s, drawn here in their place.

#import "recording_preview_surface_macos_private.h"
#include <math.h>

/// The selection gesture a ring drag reports, `SelectionGestureOperation::Rotate`.
static const uint32_t kTurnOperation = 8;
/// The rings' vertex kind in `osc.wgsl`, one quad drawing all three.
static const uint32_t kTurnRingKind = 60;
/// A ring's coloured core, half across, and its dark outline, in points.
static const CGFloat kTurnRingCore = 2.0;
static const CGFloat kTurnRingOutline = 1.0;
/// How far the ring under the pointer, and the ring being dragged, are lit
/// toward white.
static const float kTurnRingHoverLight = 0.15f;
static const float kTurnRingDragLight = 0.35f;
/// How strongly the rings' far halves show.
static const float kTurnRingFarAlpha = 0.3f;

SCREENWIDE_PREVIEW_PRIVATE BOOL selection_is_turned(ScreenwidePreviewSelection selection) {
  return selection.turnable != 0 &&
         (selection.rotation_x != 0.0 || selection.rotation_y != 0.0 ||
          selection.rotation_z != 0.0);
}

SCREENWIDE_PREVIEW_PRIVATE BOOL selection_turn(ScreenwidePreviewSurface *surface,
                                               ScreenwidePreviewSelection selection,
                                               double forward[9], double inverse[9]) {
  if (!selection_is_turned(selection) ||
      selection.pane_index >= surface.editorBaseRects.count)
    return NO;
  NSRect frame = selection_display_frame_for(surface, selection);
  // The turn is seen from the canvas's own distance, so it is measured
  // against the canvas drawn on screen, the pane the layer is laid out in.
  NSRect pane = editor_frame(surface, surface.editorBaseRects[selection.pane_index].rectValue);
  return screenwide_turn_homographies(selection.rotation_x, selection.rotation_y,
                                      selection.rotation_z, NSMidX(frame), NSMidY(frame),
                                      MAX(pane.size.width, pane.size.height), forward,
                                      inverse) != 0;
}

SCREENWIDE_PREVIEW_PRIVATE NSPoint turn_carry(const double h[9], NSPoint point) {
  double w = h[6] * point.x + h[7] * point.y + h[8];
  if (w <= 1e-9) return NSMakePoint(-1e7, -1e7);
  return NSMakePoint((h[0] * point.x + h[1] * point.y + h[2]) / w,
                     (h[3] * point.x + h[4] * point.y + h[5]) / w);
}

SCREENWIDE_PREVIEW_PRIVATE NSPoint selection_unturned_point(
    ScreenwidePreviewSurface *surface, ScreenwidePreviewSelection selection, NSPoint point) {
  double forward[9], inverse[9];
  return selection_turn(surface, selection, forward, inverse) ? turn_carry(inverse, point)
                                                              : point;
}

SCREENWIDE_PREVIEW_PRIVATE NSPoint selected_layer_point(ScreenwidePreviewSurface *surface,
                                                        NSPoint point) {
  return selection_unturned_point(surface, surface.selection, point);
}

SCREENWIDE_PREVIEW_PRIVATE NSPoint selection_drag_travel(ScreenwidePreviewInteractionView *view,
                                                         NSPoint *point) {
  NSPoint origin = view.selectionDragOrigin;
  if (view.selectionDragOperation == 1 || view.selectionDragOperation == 2) {
    *point = selection_unturned_point(view.surface, view.selectionDragStart, *point);
    origin = selection_unturned_point(view.surface, view.selectionDragStart, origin);
  }
  return NSMakePoint(point->x - origin.x, point->y - origin.y);
}

SCREENWIDE_PREVIEW_PRIVATE void turn_add_selection(ScreenwideRegionOscVertex *vertices,
                                                   NSUInteger *count, NSSize size,
                                                   ScreenwidePreviewSurface *surface,
                                                   CGFloat scale) {
  double forward[9], inverse[9];
  if (!selection_turn(surface, surface.selection, forward, inverse)) return;
  NSRect frame = selection_display_frame(surface);
  // The radius dot sits where the upright selection draws it.
  double offset = MIN(frame.size.width, frame.size.height) *
                      surface.selection.radius_percent / 100.0 * 0.55 +
                  10.0;
  const NSPoint corners[4] = {
      turn_carry(forward, NSMakePoint(NSMinX(frame), NSMinY(frame))),
      turn_carry(forward, NSMakePoint(NSMaxX(frame), NSMinY(frame))),
      turn_carry(forward, NSMakePoint(NSMaxX(frame), NSMaxY(frame))),
      turn_carry(forward, NSMakePoint(NSMinX(frame), NSMaxY(frame))),
  };
  NSPoint dot = turn_carry(forward, NSMakePoint(NSMinX(frame) + offset, NSMinY(frame) + offset));
  screenwide_region_osc_add_turned_selection(vertices, count, size, corners, &dot, scale);
}

SCREENWIDE_PREVIEW_PRIVATE BOOL turn_rings_shown(ScreenwidePreviewSurface *surface) {
  ScreenwideAnnotationMode mode = annotation_active_mode(surface);
  return surface.hasSelection && surface.selection.turnable != 0 &&
         surface.selection.crop_mode == 0 && surface.editorEnabled &&
         !surface.editorSuspended &&
         (mode == ScreenwideAnnotationModeNone || mode == ScreenwideAnnotationModeSelect) &&
         !annotation_owns_chrome(surface) && !object_picking_active(surface);
}

/// The gizmo's middle, the turned layer's, which its turn leaves in place.
static NSPoint turn_centre(ScreenwidePreviewSurface *surface) {
  NSRect frame = selection_display_frame(surface);
  return NSMakePoint(NSMidX(frame), NSMidY(frame));
}

/// The rings' radius over the selected layer, in display points.
static double turn_radius(ScreenwidePreviewSurface *surface) {
  NSRect frame = selection_display_frame(surface);
  return screenwide_turn_ring_radius(MIN(frame.size.width, frame.size.height));
}

static ScreenwideTurnRingHit turn_ring_at(ScreenwidePreviewSurface *surface, NSPoint point) {
  ScreenwidePreviewSelection selection = surface.selection;
  NSPoint centre = turn_centre(surface);
  return screenwide_turn_ring_at(selection.rotation_x, selection.rotation_y,
                                 selection.rotation_z, turn_radius(surface),
                                 point.x - centre.x, point.y - centre.y);
}

SCREENWIDE_PREVIEW_PRIVATE void turn_add_rings(ScreenwideRegionOscVertex *vertices,
                                               NSUInteger *count, NSSize size,
                                               ScreenwideRegionOscRenderState *state,
                                               ScreenwidePreviewSurface *surface,
                                               CGFloat scale) {
  // The chosen image's rings, or else the selected layer's.
  ScreenwideImageGizmo gizmo;
  if (!annotation_image_gizmo(surface, &gizmo)) {
    if (!turn_rings_shown(surface)) return;
    ScreenwidePreviewSelection selection = surface.selection;
    gizmo.centre = turn_centre(surface);
    gizmo.radius = turn_radius(surface);
    gizmo.seen[0] = selection.rotation_x;
    gizmo.seen[1] = selection.rotation_y;
    gizmo.seen[2] = selection.rotation_z;
  }
  NSPoint centre = gizmo.centre;
  double radius = gizmo.radius;
  ScreenwidePreviewInteractionView *view = surface.interaction;
  NSInteger dragged = view.turnDragActive    ? (NSInteger)view.turnDrag.axis
                      : view.imageTurnActive ? (NSInteger)view.imageTurnHold.drag.axis
                                             : -1;
  NSInteger held = dragged >= 0 ? dragged : surface.turnHoveredRing - 1;
  double basis[18];
  screenwide_turn_ring_basis(gizmo.seen[0], gizmo.seen[1], gizmo.seen[2], basis);
  // Each direction scaled to the ring in the target's pixels; the depth
  // only gives the side, so it stays as it is.
  for (NSUInteger row = 0; row < 6; row++) {
    state->turn_rings[row * 4] = (float)(basis[row * 3] * radius * scale);
    state->turn_rings[row * 4 + 1] = (float)(basis[row * 3 + 1] * radius * scale);
    state->turn_rings[row * 4 + 2] = (float)basis[row * 3 + 2];
  }
  state->turn_rings[24] = (float)(centre.x * scale);
  state->turn_rings[25] = (float)(centre.y * scale);
  state->turn_rings[26] = (float)(kTurnRingCore * scale);
  state->turn_rings[27] = (float)(kTurnRingOutline * scale);
  state->turn_rings[28] = (float)held;
  state->turn_rings[29] = dragged >= 0 ? kTurnRingDragLight : kTurnRingHoverLight;
  state->turn_rings[30] = kTurnRingFarAlpha;
  // The rings and a pixel round them for the shader to anti-alias across.
  CGFloat reach = radius + kTurnRingCore + kTurnRingOutline + 1.0 / scale;
  screenwide_region_osc_add_quad(
      vertices, count, size,
      NSMakeRect(centre.x - reach, centre.y - reach, reach * 2.0, reach * 2.0), kTurnRingKind);
}

SCREENWIDE_PREVIEW_PRIVATE NSCursor *turn_cursor(ScreenwidePreviewSurface *surface,
                                                 NSPoint point) {
  ScreenwideTurnRingHit hit = annotation_image_ring_at(surface, point);
  if (hit.axis < 0 && turn_rings_shown(surface)) hit = turn_ring_at(surface, point);
  if (hit.axis < 0) return nil;
  return screenwide_region_resize_cursor(hit.edges) ?: [NSCursor arrowCursor];
}

SCREENWIDE_PREVIEW_PRIVATE void turn_update_hover(ScreenwidePreviewSurface *surface,
                                                  NSPoint point) {
  NSInteger ring = 0;
  ScreenwidePreviewInteractionView *view = surface.interaction;
  if (!view.turnDragActive && !view.imageTurnActive) {
    ScreenwideTurnRingHit hit = annotation_image_ring_at(surface, point);
    if (hit.axis < 0 && turn_rings_shown(surface)) hit = turn_ring_at(surface, point);
    ring = hit.axis + 1;
  }
  if (ring == surface.turnHoveredRing) return;
  surface.turnHoveredRing = ring;
  redraw_selection(surface);
}

/// Reports the selection's turn as a sample of the ring drag: about X and Y
/// as its travel, about Z as its scale.
static void emit_turn(ScreenwidePreviewSurface *surface, uint32_t phase) {
  ScreenwidePreviewSelection selection = surface.selection;
  emit_selection_gesture(surface, phase, kTurnOperation, 0, selection.rotation_z,
                         selection.rotation_x, selection.rotation_y);
}

SCREENWIDE_PREVIEW_PRIVATE BOOL turn_mouse_down(ScreenwidePreviewInteractionView *view,
                                                NSEvent *event, NSPoint point) {
  // The chosen image's rings sit over the image, inside the layer's chrome.
  if (image_turn_mouse_down(view, event, point)) return YES;
  ScreenwidePreviewSurface *surface = view.surface;
  if (event.buttonNumber != 0 || !turn_rings_shown(surface) ||
      surface.selectionGestureCallback == NULL)
    return NO;
  ScreenwideTurnRingHit hit = turn_ring_at(surface, point);
  if (hit.axis < 0) return NO;
  ScreenwidePreviewSelection selection = surface.selection;
  view.turnDrag = screenwide_turn_ring_begin(selection.rotation_x, selection.rotation_y,
                                             selection.rotation_z, turn_radius(surface),
                                             (uint32_t)hit.axis, hit.angle);
  view.turnDragOrigin = point;
  view.turnDragEdges = hit.edges;
  view.turnDragActive = YES;
  view.selectionDragActive = NO;
  view.panning = NO;
  surface.turnHoveredRing = 0;
  clear_selection_snap_guides(surface);
  emit_turn(surface, 0);
  set_selection_cursor(screenwide_region_resize_cursor(hit.edges) ?: [NSCursor closedHandCursor]);
  redraw_selection(surface);
  return YES;
}

SCREENWIDE_PREVIEW_PRIVATE BOOL turn_mouse_dragged(ScreenwidePreviewInteractionView *view,
                                                   NSEvent *event) {
  if (image_turn_mouse_dragged(view, event)) return YES;
  if (!view.turnDragActive) return NO;
  ScreenwidePreviewSurface *surface = view.surface;
  NSPoint point = [view convertPoint:event.locationInWindow fromView:nil];
  ScreenwideTurnRingDrag drag = view.turnDrag;
  double turned[3];
  // Shift holds the dragged axis to whole steps, as it holds an image's
  // turn and a counter's tail.
  screenwide_turn_ring_drag(&drag, point.x - view.turnDragOrigin.x,
                            point.y - view.turnDragOrigin.y,
                            (event.modifierFlags & NSEventModifierFlagShift) != 0 ? 1 : 0,
                            turned);
  view.turnDrag = drag;
  ScreenwidePreviewSelection selection = surface.selection;
  selection.rotation_x = turned[0];
  selection.rotation_y = turned[1];
  selection.rotation_z = turned[2];
  surface.selection = selection;
  redraw_selection(surface);
  emit_turn(surface, 1);
  return YES;
}

SCREENWIDE_PREVIEW_PRIVATE BOOL turn_mouse_up(ScreenwidePreviewInteractionView *view,
                                              NSEvent *event) {
  if (image_turn_mouse_up(view, event)) return YES;
  if (!view.turnDragActive) return NO;
  // A release can land past the last drag sample; it is the turn kept.
  turn_mouse_dragged(view, event);
  view.turnDragActive = NO;
  ScreenwidePreviewSurface *surface = view.surface;
  emit_turn(surface, 2);
  redraw_selection(surface);
  set_selection_cursor_at_point(surface, [view convertPoint:event.locationInWindow fromView:nil]);
  return YES;
}

SCREENWIDE_PREVIEW_PRIVATE BOOL selection_held_by_drag(ScreenwidePreviewSurface *surface,
                                                       BOOL has_selection) {
  ScreenwidePreviewInteractionView *view = surface.interaction;
  if (!view.turnDragActive) return has_selection && view.selectionDragActive;
  if (has_selection) return YES;
  // The layer went away under the ring: the turn it reached goes with it.
  view.turnDragActive = NO;
  ScreenwideTurnRingDrag drag = view.turnDrag;
  ScreenwidePreviewSelection selection = surface.selection;
  selection.rotation_x = drag.press[0] * 180.0 / M_PI;
  selection.rotation_y = drag.press[1] * 180.0 / M_PI;
  selection.rotation_z = drag.press[2] * 180.0 / M_PI;
  surface.selection = selection;
  emit_turn(surface, 3);
  return NO;
}
