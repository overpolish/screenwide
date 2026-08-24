// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "recording_preview_surface_macos_private.h"
#include <math.h>

static const double kSelectionActionTransitionSeconds = 0.15;

static void selection_action_target(ScreenwidePreviewSurface *surface,
                                    float target[2]) {
  if (surface.selectionActionPressed) {
    target[0] = 0.918f; target[1] = 0.231f;
  } else if (surface.selectionActionHovered) {
    target[0] = 0.832f; target[1] = 0.322f;
  } else {
    target[0] = 0.924f; target[1] = 0.226f;
  }
}

SCREENWIDE_PREVIEW_PRIVATE void selection_action_shades(
    ScreenwidePreviewSurface *surface, float shades[2]) {
  if (surface.selectionActionTransitionStarted <= 0.0) {
    // Objective-C scalar properties begin at zero. Until the first state
    // change, the action is visually in its normal opaque state; afterwards
    // the last transition target is the stable current colour.
    shades[0] = surface.selectionActionToLight > 0.0f
        ? surface.selectionActionToLight : 0.924f;
    shades[1] = surface.selectionActionToDark > 0.0f
        ? surface.selectionActionToDark : 0.226f;
    return;
  }
  double progress = fmin(1.0, (CACurrentMediaTime() -
      surface.selectionActionTransitionStarted) /
      kSelectionActionTransitionSeconds);
  shades[0] = surface.selectionActionFromLight +
      (surface.selectionActionToLight - surface.selectionActionFromLight) * progress;
  shades[1] = surface.selectionActionFromDark +
      (surface.selectionActionToDark - surface.selectionActionFromDark) * progress;
}

static void schedule_selection_action_frame(ScreenwidePreviewSurface *surface,
                                             uint64_t revision) {
  dispatch_after(dispatch_time(DISPATCH_TIME_NOW, 16 * NSEC_PER_MSEC),
                 dispatch_get_main_queue(), ^{
    if (surface.selectionActionAnimationRevision != revision) return;
    redraw_selection(surface);
    if (CACurrentMediaTime() - surface.selectionActionTransitionStarted <
        kSelectionActionTransitionSeconds) {
      schedule_selection_action_frame(surface, revision);
    } else {
      surface.selectionActionTransitionStarted = 0.0;
    }
  });
}

SCREENWIDE_PREVIEW_PRIVATE void selection_action_begin_transition(
    ScreenwidePreviewSurface *surface) {
  float current[2], target[2];
  selection_action_shades(surface, current);
  selection_action_target(surface, target);
  if (fabsf(current[0] - target[0]) < 0.0001f &&
      fabsf(current[1] - target[1]) < 0.0001f) return;
  surface.selectionActionFromLight = current[0];
  surface.selectionActionFromDark = current[1];
  surface.selectionActionToLight = target[0];
  surface.selectionActionToDark = target[1];
  surface.selectionActionTransitionStarted = CACurrentMediaTime();
  uint64_t revision = ++surface.selectionActionAnimationRevision;
  schedule_selection_action_frame(surface, revision);
}

SCREENWIDE_PREVIEW_PRIVATE double selection_recenter_scale(
    ScreenwidePreviewSelection start, ScreenwidePreviewSelection resized,
    uint32_t edges) {
  BOOL verticalOnly = (edges & (4 | 8)) != 0 && (edges & (1 | 2)) == 0;
  double startSize = verticalOnly ? start.height : start.width;
  double resizedSize = verticalOnly ? resized.height : resized.width;
  return resizedSize / MAX(startSize, 0.000001);
}

SCREENWIDE_PREVIEW_PRIVATE BOOL selection_action_hover(
    ScreenwidePreviewSurface *surface, NSPoint point) {
  BOOL hovered = surface.selectionActionOperation != 0 &&
                 NSPointInRect(point, surface.selectionActionRect);
  if (surface.selectionActionHovered != hovered) {
    surface.selectionActionHovered = hovered;
    selection_action_begin_transition(surface);
    redraw_selection(surface);
  }
  return hovered;
}

SCREENWIDE_PREVIEW_PRIVATE BOOL selection_action_begin(
    ScreenwidePreviewSurface *surface, NSInteger button, NSPoint point) {
  if (button != 0 || surface.selectionActionOperation == 0 ||
      !NSPointInRect(point, surface.selectionActionRect)) return NO;
  surface.selectionActionPressed = YES;
  surface.selectionActionHovered = YES;
  selection_action_begin_transition(surface);
  redraw_selection(surface);
  set_selection_cursor([NSCursor arrowCursor]);
  return YES;
}

SCREENWIDE_PREVIEW_PRIVATE BOOL selection_action_drag(
    ScreenwidePreviewSurface *surface, NSPoint point) {
  if (!surface.selectionActionPressed) return NO;
  selection_action_hover(surface, point);
  set_selection_cursor([NSCursor arrowCursor]);
  return YES;
}

SCREENWIDE_PREVIEW_PRIVATE BOOL selection_action_end(
    ScreenwidePreviewSurface *surface, NSPoint point) {
  if (!surface.selectionActionPressed) return NO;
  BOOL activate = selection_action_hover(surface, point);
  uint32_t operation = surface.selectionActionOperation;
  surface.selectionActionPressed = NO;
  selection_action_begin_transition(surface);
  redraw_selection(surface);
  if (activate && operation != 0)
    emit_selection_gesture(surface, 0, operation, 0, 1.0, 0.0, 0.0);
  set_selection_cursor([NSCursor arrowCursor]);
  return YES;
}

SCREENWIDE_PREVIEW_PRIVATE ScreenwidePreviewSelection selection_recenter_resize(
    ScreenwidePreviewSelection start, uint32_t edges, double deltaX,
    double deltaY, NSSize pane, double *scale) {
  BOOL verticalOnly = (edges & (4 | 8)) != 0 && (edges & (1 | 2)) == 0;
  double paneWidth = MAX(pane.width, 1.0);
  double paneHeight = MAX(pane.height, 1.0);
  double startInset = verticalOnly
      ? (start.height - start.image_height) * paneHeight / 2.0
      : (start.width - start.image_width) * paneWidth / 2.0;
  double requestedDelta = verticalOnly
      ? ((edges & 4) ? -deltaY : (edges & 8) ? deltaY : 0.0) * paneHeight
      : ((edges & 1) ? -deltaX : (edges & 2) ? deltaX : 0.0) * paneWidth;
  double requestedInset = startInset + requestedDelta;
  double maximumInset = fmin(
      fmin((start.image_x - start.recenter_x) * paneWidth,
           (start.image_y - start.recenter_y) * paneHeight),
      fmin((start.recenter_x + start.recenter_width - start.image_x -
            start.image_width) * paneWidth,
           (start.recenter_y + start.recenter_height - start.image_y -
            start.image_height) * paneHeight));
  double inset = fmin(fmax(maximumInset, 0.0), fmax(requestedInset, 0.0));

  ScreenwidePreviewSelection resized = start;
  resized.x = start.image_x - inset / paneWidth;
  resized.y = start.image_y - inset / paneHeight;
  resized.width = start.image_width + 2.0 * inset / paneWidth;
  resized.height = start.image_height + 2.0 * inset / paneHeight;
  *scale = selection_recenter_scale(start, resized, edges);
  return resized;
}

SCREENWIDE_PREVIEW_PRIVATE void selection_recenter_drag(
    ScreenwidePreviewSurface *surface, ScreenwidePreviewSelection start,
    uint32_t edges, double deltaX, double deltaY, NSSize pane) {
  double scale = 1.0;
  ScreenwidePreviewSelection resized = selection_recenter_resize(
      start, edges, deltaX, deltaY, pane, &scale);
  surface.selection = resized;
  apply_editor_transform(surface);
  emit_selection_gesture(surface, 1, 1, edges, scale,
                         resized.x - start.x, resized.y - start.y);
}
