// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The caret and selection of a box being typed into.
//!
//! The text view keeps the keyboard, the input methods and the clipboard,
//! but its own caret and selection are clear: they would stand upright over
//! a turned layer, away from the box. The compositor draws both into the
//! box's type instead, as it does on Windows, so they turn and tilt with it.
//! The caret blinks on a timer here, at the system's rate.

#import "recording_preview_surface_macos_private.h"

SCREENWIDE_PREVIEW_PRIVATE void annotation_text_identity(ScreenwidePreviewSurface *surface,
                                                         uint32_t *layer, uint32_t *index) {
  NSUInteger count = 0;
  const ScreenwidePreviewAnnotation *items = annotation_items(surface, &count);
  NSInteger flat = surface.annotationTextIndex;
  *layer = surface.selection.layer_id;
  *index = 0;
  if (flat >= 0 && (NSUInteger)flat < count) {
    if (items[flat].layer_id >= 0) *layer = (uint32_t)items[flat].layer_id;
    *index = items[flat].index;
  }
}

/// How long the caret shows, and then hides: the system's own rate, in
/// milliseconds, where it is set.
static NSTimeInterval blink_period(void) {
  double ms = [NSUserDefaults.standardUserDefaults
      doubleForKey:@"NSTextInsertionPointBlinkPeriodOn"];
  return ms > 0.0 ? ms / 1000.0 : 0.5;
}

/// The UTF-8 offset of `utf16`, a UTF-16 offset into `text`: the scene draws
/// the marks by bytes of the text as Rust holds it.
static uint32_t utf8_offset(NSString *text, NSUInteger utf16) {
  utf16 = MIN(utf16, text.length);
  if (utf16 < text.length) utf16 = [text rangeOfComposedCharacterSequenceAtIndex:utf16].location;
  return (uint32_t)[[text substringToIndex:utf16] lengthOfBytesUsingEncoding:NSUTF8StringEncoding];
}

static void publish_marks(ScreenwidePreviewSurface *surface) {
  NSTextView *view = surface.annotationTextView;
  if (view == nil) return;
  uint32_t layer = 0, index = 0;
  annotation_text_identity(surface, &layer, &index);
  NSString *text = view.string;
  NSRange range = view.selectedRange;
  // A window in the background shows no caret, as a native field's does.
  BOOL caret = surface.annotationTextCaretOn && view.window.isKeyWindow;
  if (screenwide_workspace_scene_set_typing(surface.scene, 1, layer, index,
                                            utf8_offset(text, range.location),
                                            utf8_offset(text, NSMaxRange(range)), caret))
    redraw_workspace(surface);
}

SCREENWIDE_PREVIEW_PRIVATE void annotation_text_marks_changed(ScreenwidePreviewSurface *surface,
                                                              BOOL restart) {
  if (surface.annotationTextView == nil) return;
  if (restart || surface.annotationTextBlink == nil) {
    [surface.annotationTextBlink invalidate];
    surface.annotationTextCaretOn = YES;
    __weak ScreenwidePreviewSurface *weakSurface = surface;
    NSTimer *blink = [NSTimer timerWithTimeInterval:blink_period()
                                            repeats:YES
                                              block:^(NSTimer *timer) {
                                                ScreenwidePreviewSurface *strong = weakSurface;
                                                if (strong == nil ||
                                                    strong.annotationTextView == nil) {
                                                  [timer invalidate];
                                                  return;
                                                }
                                                strong.annotationTextCaretOn =
                                                    !strong.annotationTextCaretOn;
                                                publish_marks(strong);
                                              }];
    // Common modes, so the caret keeps blinking while a selection is dragged.
    [NSRunLoop.mainRunLoop addTimer:blink forMode:NSRunLoopCommonModes];
    surface.annotationTextBlink = blink;
  }
  publish_marks(surface);
}

SCREENWIDE_PREVIEW_PRIVATE void annotation_text_marks_end(ScreenwidePreviewSurface *surface) {
  [surface.annotationTextBlink invalidate];
  surface.annotationTextBlink = nil;
  if (screenwide_workspace_scene_set_typing(surface.scene, 0, 0, 0, 0, 0, 0))
    redraw_workspace(surface);
}
