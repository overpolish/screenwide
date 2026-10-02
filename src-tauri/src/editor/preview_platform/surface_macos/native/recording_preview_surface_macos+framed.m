// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "recording_preview_surface_macos_private.h"
#include <math.h>

/// The zoom one corner drag reaches at most each way, matching the camera
/// framing's own limits.
static const double ScreenwideFramedScaleLimit = 8.0;
/// The shortest a framed crop window's side may be pulled, in points.
static const double ScreenwideFramedCropMinimum = 36.0;

/// Where the anchor of a framed crop resize sits along one axis, as a share
/// of the window: the far edge of a pulled one, else the middle.
static double framed_anchor_share(uint32_t edges, uint32_t low, uint32_t high) {
  return (edges & low) ? 1.0 : (edges & high) ? 0.0 : 0.5;
}

/// The longest a framed crop window may run from its `anchor` inside the
/// image's `low` to `high`, for an anchor at `share` of the window.
static double framed_room(double anchor, double share, double low, double high) {
  if (share == 1.0) return anchor - low;
  if (share == 0.0) return high - anchor;
  return 2.0 * fmin(anchor - low, high - anchor);
}

/// `start` resized by the pointer's travel `dx`, `dy` from `edges`, keeping
/// its shape inside its image and no smaller than `minimumX` by `minimumY`.
/// The twin of `apply_framed_crop_resize` in `workspace_editor/crop.rs`.
static ScreenwidePreviewSelection framed_crop_resize(
    ScreenwidePreviewSelection start, uint32_t edges, double dx, double dy,
    double minimumX, double minimumY) {
  double width = MAX(start.width, DBL_EPSILON);
  double height = MAX(start.height, DBL_EPSILON);
  double shareX = framed_anchor_share(edges, 1, 2);
  double shareY = framed_anchor_share(edges, 4, 8);
  double grownX = (width + (shareX == 1.0 ? -dx : dx)) / width;
  double grownY = (height + (shareY == 1.0 ? -dy : dy)) / height;
  double scale = shareX != 0.5 && shareY != 0.5 ? fmax(grownX, grownY)
      : shareX != 0.5 ? grownX : grownY;
  double anchorX = start.x + shareX * width;
  double anchorY = start.y + shareY * height;
  double largest = fmin(
      framed_room(anchorX, shareX, start.image_x,
                  start.image_x + start.image_width) / width,
      framed_room(anchorY, shareY, start.image_y,
                  start.image_y + start.image_height) / height);
  double smallest = fmin(fmax(minimumX / width, minimumY / height), largest);
  scale = fmin(largest, fmax(smallest, scale));
  ScreenwidePreviewSelection resized = start;
  resized.width = width * scale;
  resized.height = height * scale;
  resized.x = anchorX - shareX * resized.width;
  resized.y = anchorY - shareY * resized.height;
  return resized;
}

/// Reports the pointer in `event` for a drag on a selection a scene fixes
/// (`framed` 1; 2 is a pane a scene places but leaves free, which drags like
/// any layer). The outline stays where it is: a move reports how far the
/// pointer has travelled, which pans the picture inside the pane, and a
/// corner reports how far it has been pulled from the pane's middle, which
/// zooms it. A crop window laid over a scene's picture keeps the box's shape
/// as it is resized, reported as its corner's travel and how far it grew.
/// Nothing snaps or grows the canvas. Answers whether the drag was one of
/// these.
static BOOL framed_selection_report(ScreenwidePreviewInteractionView *view,
                                    NSEvent *event, uint32_t phase) {
  ScreenwidePreviewSelection start = view.selectionDragStart;
  uint32_t operation = view.selectionDragOperation;
  if (!view.selectionDragActive || start.framed != 1 ||
      (operation != 0 && operation != 1 && operation != 6))
    return NO;
  ScreenwidePreviewSurface *surface = view.surface;
  if (start.pane_index >= surface.editorBaseRects.count) return NO;
  NSPoint point = [view convertPoint:event.locationInWindow fromView:nil];
  NSRect pane = surface.editorBaseRects[start.pane_index].rectValue;
  double width = MAX(pane.size.width * surface.editorZoom, 1.0);
  double height = MAX(pane.size.height * surface.editorZoom, 1.0);
  double dx = (point.x - view.selectionDragOrigin.x) / width;
  double dy = (point.y - view.selectionDragOrigin.y) / height;
  clear_selection_snap_guides(surface);
  if (operation == 0) {
    if (event.modifierFlags & NSEventModifierFlagShift) {
      if (fabs(dx * width) >= fabs(dy * height)) dy = 0.0;
      else dx = 0.0;
    }
    emit_selection_gesture(surface, phase, 0, 0, 1.0, dx, dy);
    return YES;
  }
  if (operation == 6) {
    ScreenwidePreviewSelection resized = framed_crop_resize(
        start, view.selectionDragEdges, dx, dy,
        ScreenwideFramedCropMinimum / width,
        ScreenwideFramedCropMinimum / height);
    surface.selection = resized;
    redraw_selection(surface);
    emit_selection_gesture(surface, phase, 6, view.selectionDragEdges,
                           resized.width / MAX(start.width, DBL_EPSILON),
                           resized.x - start.x, resized.y - start.y);
    return YES;
  }
  uint32_t edges = view.selectionDragEdges;
  double middleX = start.x + start.width / 2.0;
  double middleY = start.y + start.height / 2.0;
  double handleX = (edges & 1) ? start.x
      : (edges & 2) ? start.x + start.width : middleX;
  double handleY = (edges & 4) ? start.y
      : (edges & 8) ? start.y + start.height : middleY;
  double vectorX = (handleX - middleX) * width;
  double vectorY = (handleY - middleY) * height;
  double length = vectorX * vectorX + vectorY * vectorY;
  double scale = length > 0.0
      ? ((vectorX + dx * width) * vectorX + (vectorY + dy * height) * vectorY) /
          length
      : 1.0;
  scale = fmin(ScreenwideFramedScaleLimit,
               fmax(1.0 / ScreenwideFramedScaleLimit, scale));
  emit_selection_gesture(surface, phase, 1, edges, scale, 0.0, 0.0);
  return YES;
}

BOOL framed_selection_mouse_dragged(ScreenwidePreviewInteractionView *view,
                                    NSEvent *event) {
  return framed_selection_report(view, event, 1);
}

BOOL framed_selection_mouse_up(ScreenwidePreviewInteractionView *view,
                               NSEvent *event) {
  // The ordinary mouse-up reports the outline's travel, which a framed drag
  // never has. Report the pointer's instead; mouse-up skips its own report
  // and keeps only its clean-up.
  return framed_selection_report(view, event, 2);
}
