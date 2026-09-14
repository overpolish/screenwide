// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "recording_preview_surface_macos_private.h"

/// The native right press, split from the rest of the editor's pointer
/// handling: it selects nothing of its own and opens nothing natively, it
/// only tells the web layer which layer was pressed and where.
@implementation ScreenwidePreviewInteractionView (ContextMenu)

/// The layer under a right press, handed to the web layer so it can open the
/// app's own layer menu there. Nothing opens natively.
///
/// The press selects exactly what a left press would select first, so the menu
/// always acts on the layer it was opened on. A press on the shortcut layer,
/// on a canvas frame, or on empty canvas has no layer order to change and is
/// left alone.
- (BOOL)reportContextMenuAtPoint:(NSPoint)point {
  if (self.surface.contextMenuCallback == NULL ||
      !self.surface.editorEnabled || self.surface.editorSuspended ||
      !self.surface.selectionHitTestingEnabled)
    return NO;
  NSView *reference = self.surface.webview != nil ? self.surface.webview
                                                  : self.window.contentView;
  if (reference == nil) return NO;
  ScreenwidePreviewSelection target;
  uint8_t sharedHandle = 0;
  if (!shared_selection_hit(self.surface, point, &target, &sharedHandle) &&
      !selection_target_at_point(self.surface, point, &target))
    return NO;
  if (target.layer_id == ScreenwideFrameLayerId || selection_is_keyboard(target))
    return NO;
  if (!self.surface.hasSelection ||
      self.surface.selection.pane_index != target.pane_index ||
      self.surface.selection.layer_id != target.layer_id) {
    self.surface.hasSelection = YES;
    self.surface.selection = target;
    clear_selection_snap_guides(self.surface);
    if (self.surface.selectionCallback != NULL)
      self.surface.selectionCallback((int32_t)target.layer_id,
                                     self.surface.selectionContext);
    redraw_selection(self.surface);
    invalidate_selection_cursor_rects(self.surface);
  }
  self.selectionDragActive = NO;
  self.panning = NO;
  // The webview reports pointer coordinates from the top-left of the window's
  // content, with y growing downwards; this view is flipped and inset inside
  // that content, so the point goes through window base coordinates and is
  // measured against the webview's own rect there.
  NSPoint windowPoint = [self convertPoint:point toView:nil];
  NSRect referenceRect = [reference convertRect:reference.bounds toView:nil];
  self.surface.contextMenuCallback(
      target.layer_id, windowPoint.x - NSMinX(referenceRect),
      NSMaxY(referenceRect) - windowPoint.y, self.surface.contextMenuContext);
  return YES;
}

@end
