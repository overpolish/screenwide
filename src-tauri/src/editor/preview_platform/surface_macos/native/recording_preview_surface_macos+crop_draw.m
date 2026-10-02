// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "recording_preview_surface_macos_private.h"
#include <math.h>

/// The operation number a crop draw is reported as, matching Rust's
/// `SelectionGestureOperation::CropDraw`.
static const uint32_t ScreenwideCropDrawOperation = 7;
/// How far a press outside the crop travels before it draws a new one, in
/// points. Anything shorter is a click, which leaves the crop as it was.
static const double ScreenwideCropDrawThreshold = 4.0;
/// The shortest side a drawn crop starts with, in points: the floor a
/// resized crop is held to.
static const double ScreenwideCropDrawMinimum = 36.0;

typedef struct {
  double deltaX, deltaY;
  uint32_t edges;
  BOOL centered;
} ScreenwideCropDrawSample;

static double clamp_range(double value, double low, double high) {
  return fmin(fmax(value, low), fmax(high, low));
}

/// One axis of a drawn crop window: the far edge's offset from the anchor, or
/// the half extent about it when centered. The twin of Rust's
/// `workspace_editor::apply_crop_draw`, which documents the rules.
static double crop_draw_axis(double anchor, double pointer, double low,
                             double high, double minimum, BOOL centered) {
  double offset = clamp_range(pointer, low, high) - anchor;
  double toward = offset < 0.0 ? -1.0 : 1.0;
  if (centered)
    return toward * fmin(fmax(fabs(offset), minimum / 2.0),
                         fmin(anchor - low, high - anchor));
  double length = fmax(fabs(offset), minimum);
  double forward = clamp_range(anchor + toward * length, low, high) - anchor;
  double backward = clamp_range(anchor - toward * length, low, high) - anchor;
  // Pressing against an edge and dragging into it would leave a sliver; the
  // window opens away from that edge instead.
  return fabs(forward) < minimum && fabs(backward) > fabs(forward) ? backward
                                                                   : forward;
}

/// Places the drawn crop window for the pointer in `event`, shows it and its
/// magnifier, and answers what the gesture reports for it.
static ScreenwideCropDrawSample crop_draw_apply(
    ScreenwidePreviewInteractionView *view, NSEvent *event) {
  ScreenwidePreviewSurface *surface = view.surface;
  ScreenwidePreviewSelection start = view.selectionDragStart;
  NSPoint point = [view convertPoint:event.locationInWindow fromView:nil];
  NSPoint anchor = view.cropDrawAnchor;
  NSRect image = selection_image_frame_for(surface, start);
  double scaleX = start.image_width / MAX(image.size.width, 1.0);
  double scaleY = start.image_height / MAX(image.size.height, 1.0);
  double pointerX = start.image_x + (point.x - NSMinX(image)) * scaleX;
  double pointerY = start.image_y + (point.y - NSMinY(image)) * scaleY;
  double minimumX = ScreenwideCropDrawMinimum * scaleX;
  double minimumY = ScreenwideCropDrawMinimum * scaleY;
  double lowX = start.image_x, highX = start.image_x + start.image_width;
  double lowY = start.image_y, highY = start.image_y + start.image_height;
  BOOL centered = (event.modifierFlags & NSEventModifierFlagOption) != 0 &&
      fmin(anchor.x - lowX, highX - anchor.x) >= minimumX / 2.0 &&
      fmin(anchor.y - lowY, highY - anchor.y) >= minimumY / 2.0;
  double deltaX = crop_draw_axis(anchor.x, pointerX, lowX, highX, minimumX, centered);
  double deltaY = crop_draw_axis(anchor.y, pointerY, lowY, highY, minimumY, centered);
  ScreenwidePreviewSelection drawn = start;
  drawn.x = centered ? anchor.x - fabs(deltaX) : fmin(anchor.x, anchor.x + deltaX);
  drawn.y = centered ? anchor.y - fabs(deltaY) : fmin(anchor.y, anchor.y + deltaY);
  drawn.width = fabs(deltaX) * (centered ? 2.0 : 1.0);
  drawn.height = fabs(deltaY) * (centered ? 2.0 : 1.0);
  uint32_t edges = (deltaX < 0.0 ? 1u : 2u) | (deltaY < 0.0 ? 4u : 8u);
  surface.selection = drawn;
  view.selectionDragEdges = edges;
  view.selectionDragCentered = centered;
  update_crop_magnifier(surface, point, edges);
  redraw_selection(surface);
  set_selection_cursor(screenwide_region_resize_cursor(edges));
  return (ScreenwideCropDrawSample){deltaX, deltaY, edges, centered};
}

static void emit_crop_draw(ScreenwidePreviewSurface *surface, uint32_t phase,
                           ScreenwideCropDrawSample sample) {
  uint32_t edges = sample.edges |
      (sample.centered ? ScreenwideCenteredResizeEdge : 0);
  emit_selection_gesture(surface, phase, ScreenwideCropDrawOperation, edges,
                         1.0, sample.deltaX, sample.deltaY);
}

/// Whether a crop still covers the whole picture it is cut from. Moving such
/// a crop does nothing, so a press on it is free to draw a new one.
static BOOL crop_is_uncropped(ScreenwidePreviewSelection crop) {
  const double tolerance = 1e-4;
  return fabs(crop.x - crop.image_x) < tolerance &&
         fabs(crop.y - crop.image_y) < tolerance &&
         fabs(crop.width - crop.image_width) < tolerance &&
         fabs(crop.height - crop.image_height) < tolerance;
}

SCREENWIDE_PREVIEW_PRIVATE BOOL crop_draw_starts_at_point(
    ScreenwidePreviewSurface *surface, NSPoint point) {
  // A scene's crop window keeps its box's shape, so it is never drawn afresh.
  if (!surface.hasSelection || surface.selection.crop_mode == 0 ||
      surface.selection.framed != 0)
    return NO;
  ScreenwidePreviewSelection active = surface.selection;
  ScreenwidePreviewSelection target;
  uint8_t handle = 0;
  BOOL hit = shared_selection_hit(surface, point, &target, &handle) ||
      (surface.selectionHitTestingEnabled &&
       selection_target_at_point(surface, point, &target));
  // A grip resizes, and another layer's crop is chosen by the press; only the
  // body of the active crop, and only while it crops nothing, draws instead.
  if (hit)
    return handle == 0 && target.pane_index == active.pane_index &&
           target.layer_id == active.layer_id && crop_is_uncropped(active);
  if (selection_handle_edges(surface, point) != 0) return NO;
  if (NSPointInRect(point, selection_display_frame(surface)))
    return crop_is_uncropped(active);
  return YES;
}

SCREENWIDE_PREVIEW_PRIVATE BOOL crop_draw_mouse_down(
    ScreenwidePreviewInteractionView *view, NSPoint point, NSEvent *event) {
  ScreenwidePreviewSurface *surface = view.surface;
  ScreenwidePreviewSelection selection = surface.selection;
  if (event.buttonNumber != 0 || !crop_draw_starts_at_point(surface, point) ||
      selection.image_width <= 0.0 || selection.image_height <= 0.0)
    return NO;
  NSRect image = selection_image_frame_for(surface, selection);
  if (NSIsEmptyRect(image)) return NO;
  view.cropDrawAnchor = NSMakePoint(
      clamp_range(selection.image_x + (point.x - NSMinX(image)) /
                      image.size.width * selection.image_width,
                  selection.image_x, selection.image_x + selection.image_width),
      clamp_range(selection.image_y + (point.y - NSMinY(image)) /
                      image.size.height * selection.image_height,
                  selection.image_y, selection.image_y + selection.image_height));
  view.cropDrawBegun = NO;
  view.selectionDragActive = YES;
  view.selectionDragOperation = ScreenwideCropDrawOperation;
  view.selectionDragEdges = 0;
  view.selectionDragCentered = NO;
  view.selectionDragOrigin = point;
  view.selectionDragStart = selection;
  view.panning = NO;
  clear_selection_snap_guides(surface);
  set_selection_cursor([NSCursor crosshairCursor]);
  return YES;
}

SCREENWIDE_PREVIEW_PRIVATE BOOL crop_draw_mouse_dragged(
    ScreenwidePreviewInteractionView *view, NSEvent *event) {
  if (!view.selectionDragActive ||
      view.selectionDragOperation != ScreenwideCropDrawOperation)
    return NO;
  if (!view.cropDrawBegun) {
    NSPoint point = [view convertPoint:event.locationInWindow fromView:nil];
    if (hypot(point.x - view.selectionDragOrigin.x,
              point.y - view.selectionDragOrigin.y) <
        ScreenwideCropDrawThreshold)
      return YES;
    view.cropDrawBegun = YES;
    ScreenwidePreviewSelection start = view.selectionDragStart;
    emit_selection_gesture(view.surface, 0, ScreenwideCropDrawOperation, 0, 1.0,
                           view.cropDrawAnchor.x - start.x,
                           view.cropDrawAnchor.y - start.y);
  }
  emit_crop_draw(view.surface, 1, crop_draw_apply(view, event));
  return YES;
}

SCREENWIDE_PREVIEW_PRIVATE BOOL crop_draw_mouse_up(
    ScreenwidePreviewInteractionView *view, NSEvent *event) {
  if (!view.selectionDragActive ||
      view.selectionDragOperation != ScreenwideCropDrawOperation)
    return NO;
  ScreenwidePreviewSurface *surface = view.surface;
  // AppKit can deliver the release at a newer location than the last drag,
  // so the end reports the window placed for the release itself.
  if (view.cropDrawBegun) emit_crop_draw(surface, 2, crop_draw_apply(view, event));
  view.cropDrawBegun = NO;
  view.selectionDragActive = NO;
  view.selectionDragOperation = 0;
  view.selectionDragEdges = 0;
  view.selectionDragCentered = NO;
  ScreenwideRegionMagnifier cleared = surface.workspaceMagnifier;
  cleared.active = 0;
  surface.workspaceMagnifier = cleared;
  redraw_selection(surface);
  set_selection_cursor_at_point(
      surface, [view convertPoint:event.locationInWindow fromView:nil]);
  return YES;
}
