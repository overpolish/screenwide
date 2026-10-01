// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the pointer does around a text box: the double-click that opens it
//! for typing, the press that closes it, and the I-beam over it meanwhile.
//! The typing itself is in `+annotation_text.m`.

#import "recording_preview_surface_macos_private.h"

#include "recording_preview_annotation_layers_macos.h"

static NSPoint annotation_display_point(NSRect image, double x, double y) {
  return NSMakePoint(NSMinX(image) + image.size.width * x,
                     NSMinY(image) + image.size.height * y);
}

#include "recording_preview_annotation_geometry_macos.h"

/// Whether a press at `point` would drop a fresh text box: empty picture
/// under the text tool, exactly as `annotation_mouse_down` reads it.
static BOOL annotation_press_drops_text(ScreenwidePreviewSurface *surface, NSPoint point) {
  return annotation_active_mode(surface) == ScreenwideAnnotationModeText &&
         annotation_handle_at_point(surface, point) < 0 &&
         annotation_shaft_at_point(surface, point) < 0;
}

SCREENWIDE_PREVIEW_PRIVATE BOOL annotation_text_press(
    ScreenwidePreviewInteractionView *view, NSEvent *event) {
  ScreenwidePreviewSurface *surface = view.surface;
  surface.annotationTextOpensAtPoint = NO;
  // The text view takes the presses on its own text. Any other press ends
  // the typing and then carries on as the press it is, so one press can pick
  // up this box or another, grab the OSC, choose a layer or pan. Two only end
  // it: one on empty picture under the text tool, since clicking away is how
  // typing is finished and must not drop a fresh box as well; and one that
  // leaves the box with nothing to read, because Rust removes that box and
  // the grips this press would be read against still count it.
  if (annotation_text_editing(surface)) {
    NSPoint point = [view convertPoint:event.locationInWindow fromView:nil];
    BOOL drops = annotation_press_drops_text(surface, point);
    if (annotation_text_finish(surface) && !drops) return NO;
    view.annotationPressIgnored = YES;
    return YES;
  }
  if (event.buttonNumber != 0 || event.clickCount != 2 ||
      annotation_active_mode(surface) == ScreenwideAnnotationModeNone ||
      surface.annotationTextCallback == NULL)
    return NO;
  NSPoint point = [view convertPoint:event.locationInWindow fromView:nil];
  NSInteger shaft = annotation_shaft_at_point(surface, point);
  NSUInteger count = 0;
  const ScreenwidePreviewAnnotation *items = annotation_items(surface, &count);
  if (shaft < 0 || (NSUInteger)shaft >= count ||
      items[shaft].kind != ScreenwideAnnotationKindText)
    return NO;
  // The first click chose the box and may have armed a move; the second opens
  // it for typing instead, with the caret where it landed.
  view.annotationDragActive = NO;
  view.annotationDragPending = NO;
  view.annotationDragBegun = NO;
  view.annotationPressIgnored = YES;
  surface.annotationTextOpenPoint = point;
  surface.annotationTextOpensAtPoint = YES;
  uint32_t layer = items[shaft].layer_id >= 0 ? (uint32_t)items[shaft].layer_id
                                              : surface.selection.layer_id;
  surface.annotationTextCallback(0, layer, items[shaft].index, NULL, 0, 0,
                                 surface.annotationTextContext);
  return YES;
}

/// The I-beam only where the text view takes the press: over the rest of
/// the box a press ends the typing and takes hold of the box, so the pointer
/// says that instead.
SCREENWIDE_PREVIEW_PRIVATE NSCursor *annotation_text_cursor(
    ScreenwidePreviewSurface *surface, NSPoint point) {
  NSView *text = surface.annotationTextView;
  return text != nil && NSPointInRect(point, text.frame) ? [NSCursor IBeamCursor] : nil;
}
