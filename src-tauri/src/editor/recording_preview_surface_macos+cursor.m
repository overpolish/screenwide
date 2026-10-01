// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "recording_preview_surface_macos_private.h"

/// The workarea's cursor: the tracking area that owns it while the pointer is
/// over the preview, and the cursor each gesture and hover shows.
@implementation ScreenwidePreviewInteractionView (Cursor)
- (void)claimCursorControl {
  if (self.cursorRectsDisabled || self.window == nil) return;
  [self.window disableCursorRects];
  self.cursorRectsDisabled = YES;
}
- (void)releaseCursorControl {
  if (!self.cursorRectsDisabled || self.window == nil) return;
  [self.window enableCursorRects];
  self.cursorRectsDisabled = NO;
  [self.window resetCursorRects];
  expected_selection_cursor = nil;
  expected_selection_move_cursor = NO;
}
- (void)updateTrackingAreas {
  [super updateTrackingAreas];
  if (self.selectionTrackingArea != nil)
    [self removeTrackingArea:self.selectionTrackingArea];
  self.selectionTrackingArea = [[NSTrackingArea alloc]
      initWithRect:self.bounds
           options:NSTrackingMouseMoved | NSTrackingActiveAlways |
                   NSTrackingMouseEnteredAndExited |
                   NSTrackingInVisibleRect |
                   NSTrackingCursorUpdate
             owner:self userInfo:nil];
  [self addTrackingArea:self.selectionTrackingArea];
}
- (void)mouseMoved:(NSEvent *)event {
  [self claimCursorControl];
  NSPoint point = [self convertPoint:event.locationInWindow fromView:nil];
  annotation_update_hover(self.surface, point, self.annotationDragActive);
  set_selection_cursor_at_point(self.surface, point);
}
- (void)mouseEntered:(NSEvent *)event { [self mouseMoved:event]; }
- (void)mouseExited:(NSEvent *)event {
  annotation_update_hover(self.surface, NSZeroPoint, YES);
  [self releaseCursorControl];
  [[NSCursor arrowCursor] set];
  (void)event;
}
- (void)resetCursorRects {
  // Cursor rects overlap at every handle and AppKit repeatedly restores the
  // workspace cursor after `mouseMoved:` selects a resize cursor. The tracking
  // area is the single cursor authority for this native workarea.
  [super resetCursorRects];
}
- (void)cursorUpdate:(NSEvent *)event {
  if (self.selectionDragActive &&
      (self.selectionDragOperation == 2 || self.selectionDragOperation == 4))
    set_selection_cursor(screenwide_region_resize_cursor(1 | 4));
  else if (self.selectionDragActive &&
           (self.selectionDragOperation == 1 || self.selectionDragOperation == 3 ||
            self.selectionDragOperation == 6 ||
            (self.selectionDragOperation == 7 && self.selectionDragEdges != 0)))
    set_selection_cursor(screenwide_region_resize_cursor(self.selectionDragEdges));
  else if (self.selectionDragActive && self.selectionDragOperation != 7)
    set_selection_move_cursor();
  else if (self.panning)
    set_selection_cursor([NSCursor closedHandCursor]);
  else {
    NSPoint point = [self convertPoint:event.locationInWindow fromView:nil];
    set_selection_cursor_at_point(self.surface, point);
  }
}
@end
