// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "recording_preview_surface_macos_private.h"

@implementation ScreenwidePreviewInteractionView (WorkspaceMove)
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
  // A pane a scene places has no canvas to grow: the scene owns where it sits
  // on a canvas of a fixed size, whether it fixes the pane or leaves it free.
  if (!self.surface.workspaceMode || self.surface.editorBaseRects.count == 0 ||
      self.surface.selection.framed != 0) {
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
