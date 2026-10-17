// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Typing into a text box.
//!
//! The compositor draws the box and its text; the preview sits above the
//! webview, so a DOM field could never be seen over it. A native text view is
//! laid over the box's text instead, set in the same face at the same size
//! and line height, with its own glyphs transparent: it owns the caret, the
//! selection, the keyboard, the input methods and the clipboard, and every
//! change goes to Rust, which re-presents the box with the new text. What is
//! on screen is always the compositor's text, so nothing shifts on commit.
//! Where the box's look sets chosen words apart in their own faces, the
//! view's layout is not what is drawn, so Rust's layout of the words places
//! presses and line moves instead.

#import "recording_preview_surface_macos_private.h"
#import "../../../../../crates/text/src/engine/macos/annotation_text_box.h"
#include <math.h>

#include "recording_preview_annotation_layers_macos.h"

SCREENWIDE_PREVIEW_PRIVATE void on_main_async(dispatch_block_t block);

static NSPoint annotation_display_point(NSRect image, double x, double y) {
  return NSMakePoint(NSMinX(image) + image.size.width * x,
                     NSMinY(image) + image.size.height * y);
}

#include "recording_preview_annotation_geometry_macos.h"

/// Room either side of the text so a caret at the end of the widest line is
/// not clipped by the view's own edge, in points.
static const CGFloat kTextCaretRoom = 2.0;

SCREENWIDE_PREVIEW_PRIVATE void annotation_text_report(ScreenwidePreviewSurface *surface,
                                                       uint32_t phase) {
  ScreenwideAnnotationTextView *view = (ScreenwideAnnotationTextView *)surface.annotationTextView;
  if (view == nil || surface.annotationTextCallback == NULL) return;
  uint32_t layer = 0, index = 0;
  annotation_text_identity(surface, &layer, &index);
  NSData *bytes = [view.string dataUsingEncoding:NSUTF8StringEncoding] ?: [NSData data];
  surface.annotationTextRevision += 1;
  surface.annotationTextCallback(phase, layer, index, bytes.bytes, (uint32_t)bytes.length,
                                 surface.annotationTextRevision,
                                 surface.annotationTextContext);
}

/// Takes the text view away without a word to Rust, which is what Rust's own
/// end asks for.
static void remove_text_view(ScreenwidePreviewSurface *surface) {
  ScreenwideAnnotationTextView *view = (ScreenwideAnnotationTextView *)surface.annotationTextView;
  if (view == nil) return;
  view.finishing = YES;
  BOOL focused = view.window.firstResponder == view;
  surface.annotationTextView = nil;
  surface.annotationTextIndex = -1;
  surface.annotationTextCaption = NO;
  screenwide_text_words_free(surface.annotationTextWords);
  surface.annotationTextWords = NULL;
  annotation_text_marks_end(surface);
  [view removeFromSuperview];
  // Keyboard shortcuts belong to React again.
  if (focused && surface.webview != nil)
    [surface.webview.window makeFirstResponder:surface.webview];
}

SCREENWIDE_PREVIEW_PRIVATE BOOL annotation_text_editing(ScreenwidePreviewSurface *surface) {
  return surface.annotationTextView != nil;
}

SCREENWIDE_PREVIEW_PRIVATE BOOL annotation_text_finish(ScreenwidePreviewSurface *surface) {
  ScreenwideAnnotationTextView *view = (ScreenwideAnnotationTextView *)surface.annotationTextView;
  if (view == nil || view.finishing) return NO;
  view.finishing = YES;
  // The twin of Rust's `has_content`: a box left with only white space in it
  // is removed when the typing ends.
  NSCharacterSet *blank = [NSCharacterSet whitespaceAndNewlineCharacterSet];
  BOOL readable = [view.string stringByTrimmingCharactersInSet:blank].length > 0;
  annotation_text_report(surface, 2);
  remove_text_view(surface);
  return readable;
}

/// Sets the view's type the way the rasteriser sets the box's, in its look's
/// face at the size the box is drawn on screen.
static void apply_text_style(ScreenwideAnnotationTextView *view, CGFloat points,
                             uint32_t alignment) {
  if (fabs(view.fontPoints - points) < 0.01 && view.alignmentValue == alignment) return;
  view.fontPoints = points;
  view.alignmentValue = alignment;
  NSMutableParagraphStyle *paragraph =
      [screenwide_text_box_paragraph(points, alignment) mutableCopy];
  // Lines break only where the text has a line break, as the box's do.
  paragraph.lineBreakMode = NSLineBreakByClipping;
  NSDictionary *dress = @{
    NSFontAttributeName : screenwide_face_font(view.faceFamily, view.faceWeight, view.faceItalic,
                                               points),
    NSParagraphStyleAttributeName : paragraph,
    NSForegroundColorAttributeName : [NSColor clearColor],
  };
  [view.textStorage setAttributes:dress range:NSMakeRange(0, view.textStorage.length)];
  view.typingAttributes = dress;
  view.defaultParagraphStyle = paragraph;
}

/// The box being typed into as published, and the picture it lies on. NO
/// where it is gone from the grips or its picture has no size.
static BOOL typed_item(ScreenwidePreviewSurface *surface, ScreenwidePreviewAnnotation *item,
                       NSRect *image) {
  NSUInteger count = 0;
  const ScreenwidePreviewAnnotation *items = annotation_items(surface, &count);
  NSInteger flat = surface.annotationTextIndex;
  if (flat < 0 || (NSUInteger)flat >= count || items[flat].kind != ScreenwideAnnotationKindText)
    return NO;
  *item = items[flat];
  *image = annotation_layer_image(surface, item->layer_id);
  return image->size.width > 0.0 && image->size.height > 0.0;
}

SCREENWIDE_PREVIEW_PRIVATE BOOL annotation_text_words_place(ScreenwidePreviewSurface *surface,
                                                            NSRect *block, CGFloat *points,
                                                            uint32_t *align) {
  if (surface.annotationTextCaption) return caption_text_place(surface, block, points, align);
  ScreenwidePreviewAnnotation item;
  NSRect image;
  if (!typed_item(surface, &item, &image)) return NO;
  *block = annotation_text_block(image, item, points);
  *align = (uint32_t)item.start_head & 3u;
  return *points > 0.0;
}

SCREENWIDE_PREVIEW_PRIVATE void annotation_text_lay_view(ScreenwideAnnotationTextView *view,
                                                         NSRect block, CGFloat points,
                                                         uint32_t align) {
  apply_text_style(view, points, align);
  CGFloat width = MAX(block.size.width, 1.0);
  view.textContainer.size = NSMakeSize(width, CGFLOAT_MAX);
  view.frame = NSMakeRect(NSMinX(block) - kTextCaretRoom, NSMinY(block),
                          width + kTextCaretRoom * 2.0,
                          MAX(block.size.height, points * SCREENWIDE_TEXT_BOX_LINE_HEIGHT));
}

SCREENWIDE_PREVIEW_PRIVATE void annotation_text_layout(ScreenwidePreviewSurface *surface) {
  ScreenwideAnnotationTextView *view = (ScreenwideAnnotationTextView *)surface.annotationTextView;
  if (view == nil || view.finishing) return;
  NSRect block = NSZeroRect;
  CGFloat points = 0.0;
  uint32_t align = 0;
  // A caption is typed into whatever tool is in hand.
  if (surface.annotationTextCaption) {
    caption_text_layout(surface, view);
    return;
  }
  NSUInteger count = 0;
  const ScreenwidePreviewAnnotation *items = annotation_items(surface, &count);
  NSInteger flat = surface.annotationTextIndex;
  // The tool put down, the editor suspended, or the box gone from under it:
  // there is nothing left to type into, and what was typed is kept.
  if (annotation_active_mode(surface) == ScreenwideAnnotationModeNone || flat < 0 ||
      (NSUInteger)flat >= count || items[flat].kind != ScreenwideAnnotationKindText) {
    annotation_text_finish(surface);
    return;
  }
  if (annotation_text_words_place(surface, &block, &points, &align) && points > 0.0)
    annotation_text_lay_view(view, block, points, align);
}

void screenwide_preview_surface_set_annotation_text_callback(
    void *handle, screenwide_preview_annotation_text_callback callback, void *context) {
  if (handle == NULL) return;
  ScreenwidePreviewSurface *surface = (__bridge ScreenwidePreviewSurface *)handle;
  on_main_async(^{
    surface.annotationTextCallback = callback;
    surface.annotationTextContext = context;
  });
}

SCREENWIDE_PREVIEW_PRIVATE void annotation_text_open(ScreenwidePreviewSurface *surface,
                                                     NSInteger index, NSString *value,
                                                     NSString *face, uint32_t weight,
                                                     uint32_t italic, void *words, BOOL caption,
                                                     int64_t caret_utf8) {
  remove_text_view(surface);
  if (surface.interaction == nil) {
    screenwide_text_words_free(words);
    return;
  }
  ScreenwideAnnotationTextView *view =
      [[ScreenwideAnnotationTextView alloc] initWithFrame:NSMakeRect(0, 0, 1, 1)];
  view.surface = surface;
  view.faceFamily = face;
  view.faceWeight = weight;
  view.faceItalic = italic != 0;
  view.drawsBackground = NO;
  view.richText = NO;
  view.importsGraphics = NO;
  view.allowsUndo = YES;
  view.usesFindBar = NO;
  view.automaticQuoteSubstitutionEnabled = NO;
  view.automaticDashSubstitutionEnabled = NO;
  view.automaticTextReplacementEnabled = NO;
  view.automaticSpellingCorrectionEnabled = NO;
  view.continuousSpellCheckingEnabled = NO;
  view.grammarCheckingEnabled = NO;
  view.horizontallyResizable = NO;
  view.verticallyResizable = NO;
  view.textContainerInset = NSMakeSize(kTextCaretRoom, 0.0);
  view.textContainer.lineFragmentPadding = 0.0;
  view.textContainer.widthTracksTextView = NO;
  view.textContainer.heightTracksTextView = NO;
  // The compositor draws the caret and the selection into the box's type.
  view.insertionPointColor = [NSColor clearColor];
  view.selectedTextAttributes = @{NSBackgroundColorAttributeName : [NSColor clearColor]};
  view.string = value ?: @"";
  surface.annotationTextIndex = index;
  surface.annotationTextCaption = caption;
  surface.annotationTextWords = words;
  surface.annotationTextRevision = 0;
  surface.annotationTextView = view;
  [surface.interaction addSubview:view];
  annotation_text_layout(surface);
  // A box opened by a double-click puts the caret where the click landed, a
  // caption opened on one of its words after that word; one fresh from its
  // own press has nothing typed to place it in.
  NSUInteger caret = view.string.length;
  if (caret_utf8 >= 0) {
    caret = annotation_text_utf16_index(view.string, (NSUInteger)caret_utf8);
    surface.annotationTextOpensAtPoint = NO;
  } else if (surface.annotationTextOpensAtPoint && !caption) {
    caret = MIN(annotation_text_index_at(surface, surface.annotationTextOpenPoint), caret);
    surface.annotationTextOpensAtPoint = NO;
  }
  [view setSelectedRange:NSMakeRange(caret, 0)];
  [view.window makeFirstResponder:view];
}

/// Opens the box at `index` in the published grips for typing, holding
/// `text`, set in the face `family`, `weight` and `italic` name, and the
/// layout of its words where its look sets chosen ones apart, which the
/// typing frees as it ends.
void screenwide_preview_surface_begin_annotation_text(void *handle, int32_t index,
                                                      const uint8_t *text, uint32_t length,
                                                      const uint8_t *family, uint32_t family_length,
                                                      uint32_t weight, uint32_t italic,
                                                      void *words) {
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
    annotation_text_open(surface, index, value, face, weight, italic, words, NO, -1);
  });
}

/// Hands the box being typed into the layout of its words again, after the
/// panel chose words or dressed it while it is typed into: NULL where it sets
/// none apart now. The layout follows any change typed since Rust made it.
void screenwide_preview_surface_set_annotation_text_words(void *handle, void *words) {
  if (handle == NULL) {
    screenwide_text_words_free(words);
    return;
  }
  ScreenwidePreviewSurface *surface = (__bridge ScreenwidePreviewSurface *)handle;
  on_main_async(^{
    ScreenwideAnnotationTextView *view =
        (ScreenwideAnnotationTextView *)surface.annotationTextView;
    if (view == nil || view.finishing) {
      screenwide_text_words_free(words);
      return;
    }
    NSData *bytes = [view.string dataUsingEncoding:NSUTF8StringEncoding] ?: [NSData data];
    screenwide_text_words_follow(words, bytes.bytes, (uint32_t)bytes.length);
    screenwide_text_words_free(surface.annotationTextWords);
    surface.annotationTextWords = words;
    // Whether the view takes presses itself may have changed with it.
    [view.window invalidateCursorRectsForView:view];
  });
}

/// Takes the typing away, as Rust's own end of the session asks.
void screenwide_preview_surface_end_annotation_text(void *handle) {
  if (handle == NULL) return;
  ScreenwidePreviewSurface *surface = (__bridge ScreenwidePreviewSurface *)handle;
  on_main_async(^{
    remove_text_view(surface);
  });
}
