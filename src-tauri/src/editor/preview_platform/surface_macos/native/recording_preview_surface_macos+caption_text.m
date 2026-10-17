// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Typing into the caption showing, on the picture. A double-click on it
//! opens it in the same text view a text box is typed in; its words are laid
//! out by the scene as the caption draws them, so the view takes no press of
//! its own and Rust's layout places the caret. Return ends the typing rather
//! than starting a line. A right click on one of its words names that word
//! for the captions' word menu.

#import "recording_preview_surface_macos_private.h"

#include "recording_preview_annotation_layers_macos.h"

SCREENWIDE_PREVIEW_PRIVATE void on_main_async(dispatch_block_t block);

SCREENWIDE_PREVIEW_PRIVATE BOOL caption_text_place(ScreenwidePreviewSurface *surface,
                                                   NSRect *block, CGFloat *points,
                                                   uint32_t *align) {
  ScreenwidePreviewSelection target = surface.annotationTextCaptionTarget;
  double rect[4] = {0.0, 0.0, 0.0, 0.0};
  double size = 0.0, share = 0.0;
  if (!screenwide_workspace_scene_caption_editing_block(surface.scene, target.pane_index, rect,
                                                        &size, &share))
    return NO;
  target.x = rect[0];
  target.y = rect[1];
  target.width = rect[2];
  target.height = rect[3];
  *block = selection_display_frame_for(surface, target);
  *points = block->size.height * size;
  *align = (uint32_t)llround(MIN(MAX(share, 0.0), 1.0) * 2.0);
  return !NSIsEmptyRect(*block) && *points > 0.0;
}

SCREENWIDE_PREVIEW_PRIVATE void caption_text_layout(ScreenwidePreviewSurface *surface,
                                                    ScreenwideAnnotationTextView *view) {
  NSRect block = NSZeroRect;
  CGFloat points = 0.0;
  uint32_t align = 0;
  if (!caption_text_place(surface, &block, &points, &align)) return;
  annotation_text_lay_view(view, block, points, align);
  // The scene lays the caption out a draw after it opens, so a caret put
  // where the opening press landed waits for that. This runs inside that
  // redraw, and moving the caret redraws to show it, so the caret is moved
  // once the redraw has returned.
  if (surface.annotationTextOpensAtPoint) {
    surface.annotationTextOpensAtPoint = NO;
    dispatch_async(dispatch_get_main_queue(), ^{
      if (surface.annotationTextView != view || view.finishing) return;
      NSUInteger caret = annotation_text_index_at(surface, surface.annotationTextOpenPoint);
      [view setSelectedRange:NSMakeRange(MIN(caret, view.string.length), 0)];
    });
  }
}

/// The caption showing under `point`, as the captions layer a press on it
/// reports, with the select tool or the Captions tool in hand. It is found
/// where the scene draws it rather than among the layers a press picks: the
/// select tool's first click on the caption brings up the Captions tool,
/// which picks no layers, so a double-click's second press lands there.
SCREENWIDE_PREVIEW_PRIVATE BOOL caption_target_at(ScreenwidePreviewSurface *surface,
                                                  NSPoint point,
                                                  ScreenwidePreviewSelection *target) {
  if (annotation_active_mode(surface) != ScreenwideAnnotationModeNone) return NO;
  // The twin of the captions target React hands the select tool: the whole
  // canvas of the screen's pane, tested against where the caption shows.
  ScreenwidePreviewSelection caption = {0};
  caption.pane_index = 0;
  caption.layer_id = ScreenwideCaptionsLayerId;
  caption.radius_disabled = 1;
  caption.width = 1.0;
  caption.height = 1.0;
  if (!overlay_body_contains(surface, caption, point)) return NO;
  *target = caption;
  return YES;
}

SCREENWIDE_PREVIEW_PRIVATE BOOL caption_text_press(ScreenwidePreviewInteractionView *view,
                                                   NSEvent *event, NSPoint point) {
  ScreenwidePreviewSurface *surface = view.surface;
  ScreenwidePreviewSelection target;
  if (event.buttonNumber != 0 || event.clickCount != 2 || surface.annotationTextCallback == NULL ||
      annotation_text_editing(surface) || !caption_target_at(surface, point, &target))
    return NO;
  view.selectionDragActive = NO;
  view.panning = NO;
  // The press chooses the captions as any press on them does, and opens the
  // caption under it for typing, its caret where it landed.
  report_overlay_press(surface, target);
  surface.annotationTextCaptionTarget = target;
  surface.annotationTextOpenPoint = selected_layer_point(surface, point);
  surface.annotationTextOpensAtPoint = YES;
  surface.annotationTextCallback(0, ScreenwideCaptionsLayerId, 0, NULL, 0, 0,
                                 surface.annotationTextContext);
  return YES;
}

SCREENWIDE_PREVIEW_PRIVATE int64_t caption_text_offset_at(ScreenwidePreviewSurface *surface,
                                                        NSPoint point) {
  ScreenwideAnnotationTextView *view = (ScreenwideAnnotationTextView *)surface.annotationTextView;
  if (view == nil || view.finishing || !surface.annotationTextCaption ||
      !NSPointInRect(point, view.frame))
    return -1;
  NSUInteger at = MIN(annotation_text_index_at(surface, selected_layer_point(surface, point)),
                      view.string.length);
  return (int64_t)[[view.string substringToIndex:at]
      lengthOfBytesUsingEncoding:NSUTF8StringEncoding];
}

/// Types `text` over bytes `start` to `end` of the caption being typed into,
/// as typing would: it undoes with the typing and reports as a change of it.
void screenwide_preview_surface_replace_caption_text(void *handle, uint32_t start, uint32_t end,
                                                     const uint8_t *text, uint32_t length) {
  if (handle == NULL) return;
  ScreenwidePreviewSurface *surface = (__bridge ScreenwidePreviewSurface *)handle;
  NSString *typed = text == NULL || length == 0
      ? @""
      : [[NSString alloc] initWithBytes:text length:length encoding:NSUTF8StringEncoding];
  if (typed == nil) return;
  on_main_async(^{
    ScreenwideAnnotationTextView *view = (ScreenwideAnnotationTextView *)surface.annotationTextView;
    if (view == nil || view.finishing || !surface.annotationTextCaption) return;
    NSUInteger from = annotation_text_utf16_index(view.string, start);
    NSUInteger to = annotation_text_utf16_index(view.string, end);
    if (to <= from) return;
    NSRange range = NSMakeRange(from, to - from);
    if (![view shouldChangeTextInRange:range replacementString:typed]) return;
    [view.textStorage replaceCharactersInRange:range withString:typed];
    [view didChangeText];
    [view setSelectedRange:NSMakeRange(from + typed.length, 0)];
  });
}

SCREENWIDE_PREVIEW_PRIVATE int32_t caption_word_at(ScreenwidePreviewSurface *surface,
                                                   ScreenwidePreviewSelection target,
                                                   NSPoint point) {
  double x = 0.0, y = 0.0, width = 0.0, height = 0.0;
  if (!screenwide_workspace_scene_caption_bounds(surface.scene, target.pane_index, &x, &y, &width,
                                                 &height))
    return -1;
  // The caption's words are hit-tested where they are drawn: the point is
  // carried from the box their bounds are shown in back into its canvas.
  NSRect shown = selection_hit_frame(surface, target);
  if (NSIsEmptyRect(shown)) return -1;
  double across = x + (point.x - NSMinX(shown)) / shown.size.width * width;
  double down = y + (point.y - NSMinY(shown)) / shown.size.height * height;
  return screenwide_workspace_scene_caption_word_at(surface.scene, target.pane_index, across,
                                                    down);
}

/// Opens the caption showing for typing, holding `text`, set in the face
/// `family`, `weight` and `italic` name, its words laid out by `words`, the
/// caret at byte `caret` of `text`, or -1 for where the opening press landed.
/// A caption opened from its word menu was not pressed on, so it is typed
/// into on the screen's pane.
void screenwide_preview_surface_begin_caption_text(void *handle, const uint8_t *text,
                                                   uint32_t length, const uint8_t *family,
                                                   uint32_t family_length, uint32_t weight,
                                                   uint32_t italic, void *words, int64_t caret) {
  if (handle == NULL) {
    screenwide_text_words_free(words);
    return;
  }
  ScreenwidePreviewSurface *surface = (__bridge ScreenwidePreviewSurface *)handle;
  NSString *value = text == NULL || length == 0
      ? @""
      : [[NSString alloc] initWithBytes:text length:length encoding:NSUTF8StringEncoding];
  NSString *face = family == NULL ? nil
      : [[NSString alloc] initWithBytes:family length:family_length encoding:NSUTF8StringEncoding];
  on_main_async(^{
    if (!surface.annotationTextOpensAtPoint) {
      ScreenwidePreviewSelection target = surface.annotationTextCaptionTarget;
      target.layer_id = ScreenwideCaptionsLayerId;
      surface.annotationTextCaptionTarget = target;
    }
    annotation_text_open(surface, -1, value, face, weight, italic, words, YES, caret);
  });
}
