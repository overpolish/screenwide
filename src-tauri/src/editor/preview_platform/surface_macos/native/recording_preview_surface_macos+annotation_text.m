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

@interface ScreenwideAnnotationTextView : NSTextView
@property(nonatomic, weak) ScreenwidePreviewSurface *surface;
@property(nonatomic) BOOL finishing;
@property(nonatomic) CGFloat fontPoints;
@property(nonatomic) uint32_t alignmentValue;
/// The face the box's look sets its type in; no family is the app's Inter.
@property(nonatomic, copy) NSString *faceFamily;
@property(nonatomic) uint32_t faceWeight;
@property(nonatomic) BOOL faceItalic;
@end

static void report_text(ScreenwidePreviewSurface *surface, uint32_t phase) {
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
  report_text(surface, 2);
  remove_text_view(surface);
  return readable;
}

@implementation ScreenwideAnnotationTextView
- (BOOL)isFlipped {
  return YES;
}

/// Escape commits: undo is there for a change that was not wanted.
- (void)cancelOperation:(id)sender {
  (void)sender;
  annotation_text_finish(self.surface);
}

- (BOOL)resignFirstResponder {
  BOOL resigned = [super resignFirstResponder];
  // Anything else taking the keyboard - a click on the webview's panel, the
  // workarea, another window - ends the typing. It ends a turn later, never
  // in the middle of AppKit handing the keyboard over.
  if (resigned && !self.finishing) {
    __weak ScreenwideAnnotationTextView *weakSelf = self;
    dispatch_async(dispatch_get_main_queue(), ^{
      ScreenwideAnnotationTextView *view = weakSelf;
      if (view != nil && view.surface.annotationTextView == view)
        annotation_text_finish(view.surface);
    });
  }
  return resigned;
}

- (void)didChangeText {
  [super didChangeText];
  if (self.finishing) return;
  report_text(self.surface, 1);
  annotation_text_marks_changed(self.surface, YES);
}

- (void)setSelectedRanges:(NSArray<NSValue *> *)ranges
                 affinity:(NSSelectionAffinity)affinity
           stillSelecting:(BOOL)stillSelecting {
  [super setSelectedRanges:ranges affinity:affinity stillSelecting:stillSelecting];
  if (!self.finishing && self.surface.annotationTextView == self)
    annotation_text_marks_changed(self.surface, YES);
}

/// On a turned layer the view stands upright, away from the box it types
/// into: the interaction view takes every press there and carries it back
/// through the turn, and says itself where the I-beam shows.
- (NSView *)hitTest:(NSPoint)point {
  return annotation_text_turned(self.surface) ? nil : [super hitTest:point];
}

- (void)resetCursorRects {
  if (!annotation_text_turned(self.surface)) [super resetCursorRects];
}

- (void)cursorUpdate:(NSEvent *)event {
  if (!annotation_text_turned(self.surface)) [super cursorUpdate:event];
}

/// The editing keys a text field answers without a menu to route them: the
/// app's menu is not guaranteed to carry an Edit menu, and without one these
/// would go nowhere.
- (BOOL)performKeyEquivalent:(NSEvent *)event {
  if (self.window.firstResponder != self) return [super performKeyEquivalent:event];
  NSEventModifierFlags flags = event.modifierFlags &
      (NSEventModifierFlagCommand | NSEventModifierFlagShift |
       NSEventModifierFlagOption | NSEventModifierFlagControl);
  NSString *key = event.charactersIgnoringModifiers.lowercaseString;
  if (flags == NSEventModifierFlagCommand) {
    if ([key isEqualToString:@"a"]) [self selectAll:nil];
    else if ([key isEqualToString:@"c"]) [self copy:nil];
    else if ([key isEqualToString:@"v"]) [self pasteAsPlainText:nil];
    else if ([key isEqualToString:@"x"]) [self cut:nil];
    else if ([key isEqualToString:@"z"]) [self.undoManager undo];
    else return [super performKeyEquivalent:event];
    return YES;
  }
  if (flags == (NSEventModifierFlagCommand | NSEventModifierFlagShift) &&
      [key isEqualToString:@"z"]) {
    [self.undoManager redo];
    return YES;
  }
  return [super performKeyEquivalent:event];
}
@end

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

SCREENWIDE_PREVIEW_PRIVATE void annotation_text_layout(ScreenwidePreviewSurface *surface) {
  ScreenwideAnnotationTextView *view = (ScreenwideAnnotationTextView *)surface.annotationTextView;
  if (view == nil || view.finishing) return;
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
  ScreenwidePreviewAnnotation item = items[flat];
  NSRect image = annotation_layer_image(surface, item.layer_id);
  if (image.size.width <= 0.0 || image.size.height <= 0.0) return;
  CGFloat points = 0.0;
  NSRect block = annotation_text_block(image, item, &points);
  if (points <= 0.0) return;
  apply_text_style(view, points, (uint32_t)item.start_head & 3u);
  CGFloat width = MAX(block.size.width, 1.0);
  view.textContainer.size = NSMakeSize(width, CGFLOAT_MAX);
  view.frame = NSMakeRect(NSMinX(block) - kTextCaretRoom, NSMinY(block),
                          width + kTextCaretRoom * 2.0,
                          MAX(block.size.height, points * SCREENWIDE_TEXT_BOX_LINE_HEIGHT));
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

/// Opens the box at `index` in the published grips for typing, holding
/// `text`, set in the face `family`, `weight` and `italic` name.
void screenwide_preview_surface_begin_annotation_text(void *handle, int32_t index,
                                                      const uint8_t *text, uint32_t length,
                                                      const uint8_t *family, uint32_t family_length,
                                                      uint32_t weight, uint32_t italic) {
  if (handle == NULL) return;
  ScreenwidePreviewSurface *surface = (__bridge ScreenwidePreviewSurface *)handle;
  NSString *value = text == NULL || length == 0
      ? @""
      : [[NSString alloc] initWithBytes:text length:length encoding:NSUTF8StringEncoding];
  NSString *face = family == NULL ? nil
      : [[NSString alloc] initWithBytes:family length:family_length encoding:NSUTF8StringEncoding];
  on_main_async(^{
    remove_text_view(surface);
    if (surface.interaction == nil) return;
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
    surface.annotationTextRevision = 0;
    surface.annotationTextView = view;
    [surface.interaction addSubview:view];
    annotation_text_layout(surface);
    // A box opened by a double-click puts the caret where the click landed;
    // one fresh from its own press has nothing typed to place it in.
    NSUInteger caret = view.string.length;
    if (surface.annotationTextOpensAtPoint) {
      caret = MIN(annotation_text_index_at(surface, surface.annotationTextOpenPoint), caret);
      surface.annotationTextOpensAtPoint = NO;
    }
    [view setSelectedRange:NSMakeRange(caret, 0)];
    [view.window makeFirstResponder:view];
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
