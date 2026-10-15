// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the pointer does around a text box: the double-click that opens it
//! for typing, the press that closes it, and the I-beam over it meanwhile.
//! On a turned layer the presses in the box too: the text view stands
//! upright away from it, so the press is carried back through the turn here
//! and placed in the view's text. The typing itself is in `+annotation_text.m`.

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

SCREENWIDE_PREVIEW_PRIVATE BOOL annotation_text_turned(ScreenwidePreviewSurface *surface) {
  double storage[9];
  return surface != nil && surface.annotationTextView != nil &&
         annotation_chrome_turn(surface, storage) != NULL;
}

SCREENWIDE_PREVIEW_PRIVATE NSUInteger annotation_text_index_at(ScreenwidePreviewSurface *surface,
                                                              NSPoint point) {
  NSTextView *text = surface.annotationTextView;
  if (text == nil) return 0;
  [text.layoutManager ensureLayoutForTextContainer:text.textContainer];
  return MIN([text characterIndexForInsertionAtPoint:[text convertPoint:point
                                                                fromView:surface.interaction]],
             text.string.length);
}

/// Selects from character `from` to character `to`, in the steps the press
/// that began it selects by.
static void select_between(ScreenwidePreviewSurface *surface, NSUInteger from, NSUInteger to) {
  NSTextView *text = surface.annotationTextView;
  NSRange range = NSMakeRange(MIN(from, to), MAX(from, to) - MIN(from, to));
  if (surface.annotationTextSelectingBy != NSSelectByCharacter)
    range = [text selectionRangeForProposedRange:range
                                     granularity:surface.annotationTextSelectingBy];
  [text setSelectedRange:range];
}

/// A press at `point`, on the layer standing upright, in the box being typed
/// into on a turned layer: it places the caret, a double-click selects the
/// word and a triple-click the paragraph, and Shift extends the selection.
/// NO where the press is not that.
static BOOL annotation_text_turned_press(ScreenwidePreviewSurface *surface, NSPoint point,
                                         NSEvent *event) {
  NSTextView *text = surface.annotationTextView;
  if (!annotation_text_turned(surface) || !NSPointInRect(point, text.frame)) return NO;
  NSUInteger at = annotation_text_index_at(surface, point);
  surface.annotationTextSelectingBy = event.clickCount >= 3   ? NSSelectByParagraph
                                      : event.clickCount == 2 ? NSSelectByWord
                                                              : NSSelectByCharacter;
  NSRange chosen = text.selectedRange;
  BOOL extend = (event.modifierFlags & NSEventModifierFlagShift) != 0 &&
                surface.annotationTextSelectingBy == NSSelectByCharacter;
  surface.annotationTextSelectingFrom =
      !extend ? at : at < chosen.location ? NSMaxRange(chosen) : chosen.location;
  select_between(surface, surface.annotationTextSelectingFrom, at);
  return YES;
}

SCREENWIDE_PREVIEW_PRIVATE void annotation_text_drag(ScreenwidePreviewSurface *surface,
                                                     NSPoint point) {
  if (!annotation_text_turned(surface)) return;
  select_between(surface, surface.annotationTextSelectingFrom,
                 annotation_text_index_at(surface, point));
}

SCREENWIDE_PREVIEW_PRIVATE BOOL annotation_text_press(
    ScreenwidePreviewInteractionView *view, NSEvent *event) {
  ScreenwidePreviewSurface *surface = view.surface;
  surface.annotationTextOpensAtPoint = NO;
  // The text view takes the presses on its own text. Any other press ends
  // the typing and then carries on as the press it is, so one press can pick
  // up this box or another, grab the OSC, choose a layer or pan - or, on
  // empty picture under the text tool, drop the next box, so a run of boxes
  // takes a press each. Only one press just ends it: one that leaves the box
  // with nothing to read and lands on something other than empty picture,
  // because Rust removes that box and the grips this press would be read
  // against still count it. A fresh box is read against no grip.
  // The points are on the layer standing upright, where the grips and the
  // text view are.
  NSPoint point = selected_layer_point(
      surface, [view convertPoint:event.locationInWindow fromView:nil]);
  if (annotation_text_editing(surface)) {
    if (annotation_text_turned_press(surface, point, event)) {
      view.annotationPressIgnored = YES;
      return YES;
    }
    BOOL drops = annotation_press_drops_text(surface, point);
    if (annotation_text_finish(surface) || drops) return NO;
    view.annotationPressIgnored = YES;
    return YES;
  }
  if (event.buttonNumber != 0 || event.clickCount != 2 ||
      annotation_active_mode(surface) == ScreenwideAnnotationModeNone ||
      surface.annotationTextCallback == NULL)
    return NO;
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
