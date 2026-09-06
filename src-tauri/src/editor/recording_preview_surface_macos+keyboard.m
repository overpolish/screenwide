// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "recording_preview_surface_macos_private.h"

SCREENWIDE_PREVIEW_PRIVATE BOOL selection_is_keyboard(
    ScreenwidePreviewSelection selection) {
  return selection.layer_id == UINT32_MAX - 1;
}

static BOOL keyboard_visible_hit_rect(
    ScreenwidePreviewSurface *surface, ScreenwidePreviewSelection selection,
    NSRect *rect) {
  if (rect == NULL || !selection_is_keyboard(selection) ||
      !surface.workspaceMode || surface.views.count == 0) return NO;
  double x = 0.0, y = 0.0, width = 0.0, height = 0.0;
  [surface.workspaceLock lock];
  int found = screenwide_gpu_still_presenter_workspace_keyboard_bounds(
      surface.views[0].compositor, selection.pane_index,
      &x, &y, &width, &height);
  [surface.workspaceLock unlock];
  if (!found) return NO;
  selection.x = x;
  selection.y = y;
  selection.width = width;
  selection.height = height;
  *rect = selection_display_frame_for(surface, selection);
  return !NSIsEmptyRect(*rect);
}

SCREENWIDE_PREVIEW_PRIVATE NSRect keyboard_hit_frame(
    ScreenwidePreviewSurface *surface, ScreenwidePreviewSelection selection) {
  NSRect rect = selection_display_frame_for(surface, selection);
  keyboard_visible_hit_rect(surface, selection, &rect);
  return rect;
}

SCREENWIDE_PREVIEW_PRIVATE BOOL keyboard_body_contains(
    ScreenwidePreviewSurface *surface, ScreenwidePreviewSelection selection,
    NSPoint point) {
  return !selection_is_keyboard(selection) ||
      NSPointInRect(point, keyboard_hit_frame(surface, selection));
}

SCREENWIDE_PREVIEW_PRIVATE void begin_keyboard_transform(
    ScreenwidePreviewSurface *surface) {
  if (!surface.workspaceMode || surface.views.count == 0) return;
  [surface.workspaceLock lock];
  screenwide_gpu_still_presenter_begin_workspace_resize(
      surface.views[0].compositor);
  [surface.workspaceLock unlock];
}

SCREENWIDE_PREVIEW_PRIVATE void update_keyboard_transform(
    ScreenwidePreviewSurface *surface, ScreenwidePreviewSelection selection,
    double scale) {
  if (!surface.workspaceMode || surface.views.count == 0 ||
      !selection_is_keyboard(selection)) return;
  [surface.workspaceLock lock];
  screenwide_gpu_still_presenter_update_workspace_keyboard(
      surface.views[0].compositor, selection.pane_index,
      selection.x + selection.width / 2.0,
      selection.y + selection.height / 2.0, scale);
  [surface.workspaceLock unlock];
}

@implementation ScreenwidePreviewInteractionView (Keyboard)
- (void)beginWorkspaceMove {
  self.selectionMoveDeltaX = 0.0;
  self.selectionMoveDeltaY = 0.0;
  self.selectionMoveAutoFitActive = NO;
  self.selectionMoveAutoFitBounds = NSZeroRect;
  self.selectionMoveTargetsStart = [self.surface.selectionTargets copy];
  self.selectionMoveZoomStart = self.surface.editorZoom;
  self.selectionMovePanStart =
      NSMakePoint(self.surface.editorPanX, self.surface.editorPanY);
  self.selectionFramePaneStarts = [self.surface.editorBaseRects copy];
  if (selection_is_keyboard(self.surface.selection)) {
    self.selectionMoveFrameStart = NSZeroRect;
    begin_keyboard_transform(self.surface);
    return;
  }
  if (!self.surface.workspaceMode || self.surface.editorBaseRects.count == 0) {
    self.selectionMoveFrameStart = NSZeroRect;
    return;
  }
  NSUInteger paneIndex = self.surface.selection.pane_index;
  self.selectionMoveFrameStart = paneIndex < self.surface.editorBaseRects.count
      ? self.surface.editorBaseRects[paneIndex].rectValue
      : NSZeroRect;
  begin_workspace_frame_resize(self.surface);
}
@end
