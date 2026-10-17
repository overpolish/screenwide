// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "recording_preview_surface_macos_private.h"

// The keyboard shortcuts and the captions are drawn over the picture, sized
// and placed by their own tools. A press on one is reported, for the editor
// to open that tool, and never selects, moves or resizes it.

SCREENWIDE_PREVIEW_PRIVATE BOOL selection_is_overlay(
    ScreenwidePreviewSelection selection) {
  return selection.layer_id == ScreenwideKeyboardLayerId ||
         selection.layer_id == ScreenwideCaptionsLayerId;
}

// Where the overlay is drawn now: the keys of the shortcut showing or the
// words of the caption showing, not the room the editor gives them.
static BOOL overlay_visible_rect(
    ScreenwidePreviewSurface *surface, ScreenwidePreviewSelection selection,
    NSRect *rect) {
  if (!surface.workspaceMode || surface.views.count == 0) return NO;
  double x = 0.0, y = 0.0, width = 0.0, height = 0.0;
  int found = selection.layer_id == ScreenwideKeyboardLayerId
      ? screenwide_workspace_scene_keyboard_bounds(
            surface.scene, selection.pane_index, &x, &y, &width, &height)
      : screenwide_workspace_scene_caption_bounds(
            surface.scene, selection.pane_index, &x, &y, &width, &height);
  if (!found) return NO;
  selection.x = x;
  selection.y = y;
  selection.width = width;
  selection.height = height;
  *rect = selection_display_frame_for(surface, selection);
  return !NSIsEmptyRect(*rect);
}

SCREENWIDE_PREVIEW_PRIVATE NSRect selection_hit_frame(
    ScreenwidePreviewSurface *surface, ScreenwidePreviewSelection selection) {
  if (!selection_is_overlay(selection))
    return selection_display_frame_for(surface, selection);
  NSRect rect = NSZeroRect;
  overlay_visible_rect(surface, selection, &rect);
  return rect;
}

SCREENWIDE_PREVIEW_PRIVATE BOOL overlay_body_contains(
    ScreenwidePreviewSurface *surface, ScreenwidePreviewSelection selection,
    NSPoint point) {
  return !selection_is_overlay(selection) ||
      NSPointInRect(point, selection_hit_frame(surface, selection));
}

SCREENWIDE_PREVIEW_PRIVATE void report_overlay_press(
    ScreenwidePreviewSurface *surface, ScreenwidePreviewSelection target) {
  if (surface.selectionCallback != NULL)
    surface.selectionCallback((int32_t)target.layer_id,
                              surface.selectionContext);
}
