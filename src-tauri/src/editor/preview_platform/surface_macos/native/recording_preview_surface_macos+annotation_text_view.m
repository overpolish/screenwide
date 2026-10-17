// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The text view laid over a box being typed into: its keys, its edits and
//! its selection, which go to Rust and the compositor. Where the box's look
//! sets chosen words apart in their own faces, a line up or down goes by
//! Rust's layout of the words rather than the view's. Opening, placing and
//! closing the view is in `+annotation_text.m`.

#import "recording_preview_surface_macos_private.h"
#include <math.h>

/// The view's text laid out with no selection drawn behind it. The
/// compositor draws the selection into the box's type; the view's own would
/// stand flat over the box, and once its window is not the key one, as while
/// the panel is used, the view draws it grey and solid whatever its colours.
@interface ScreenwideAnnotationTextLayout : NSLayoutManager
@end

@implementation ScreenwideAnnotationTextLayout
- (void)fillBackgroundRectArray:(const NSRect *)rectArray
                          count:(NSUInteger)rectCount
              forCharacterRange:(NSRange)charRange
                          color:(NSColor *)color {
  (void)rectArray;
  (void)rectCount;
  (void)charRange;
  (void)color;
}
@end

@implementation ScreenwideAnnotationTextView
- (instancetype)initWithFrame:(NSRect)frame {
  self = [super initWithFrame:frame];
  if (self != nil) [self.textContainer replaceLayoutManager:[ScreenwideAnnotationTextLayout new]];
  return self;
}

- (BOOL)isFlipped {
  return YES;
}

/// Return in a caption ends the typing, as it does in a field: a caption's
/// lines are its rules' and the breaks placed in the panel, not typed.
- (void)insertNewline:(id)sender {
  if (self.surface.annotationTextCaption) annotation_text_finish(self.surface);
  else [super insertNewline:sender];
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
  NSData *bytes = [self.string dataUsingEncoding:NSUTF8StringEncoding] ?: [NSData data];
  screenwide_text_words_follow(self.surface.annotationTextWords, bytes.bytes,
                               (uint32_t)bytes.length);
  annotation_text_report(self.surface, 1);
  annotation_text_marks_changed(self.surface, YES);
}

- (void)setSelectedRanges:(NSArray<NSValue *> *)ranges
                 affinity:(NSSelectionAffinity)affinity
           stillSelecting:(BOOL)stillSelecting {
  [super setSelectedRanges:ranges affinity:affinity stillSelecting:stillSelecting];
  if (!self.settingLine) self.lineRun = NO;
  if (!self.finishing && self.surface.annotationTextView == self)
    annotation_text_marks_changed(self.surface, YES);
}

/// A line up or down through words set apart, as Rust lays them out,
/// extending the selection where `extend`. NO where the view's own layout
/// is what is drawn, which moves by it instead.
static BOOL move_line(ScreenwideAnnotationTextView *view, BOOL down, BOOL extend) {
  ScreenwidePreviewSurface *surface = view.surface;
  NSRect block = NSZeroRect;
  CGFloat points = 0.0;
  uint32_t align = 0;
  if (!annotation_text_sets_words_apart(surface) ||
      !annotation_text_words_place(surface, &block, &points, &align))
    return NO;
  NSString *text = view.string;
  NSRange range = view.selectedRange;
  if (!view.lineRun) {
    view.lineGoal = NAN;
    view.lineAnchor = range.length == 0 || down ? range.location : NSMaxRange(range);
  }
  NSUInteger from = range.length == 0               ? range.location
                    : range.location == view.lineAnchor ? NSMaxRange(range)
                                                        : range.location;
  double goal = view.lineGoal;
  uint32_t moved = screenwide_text_words_line(surface.annotationTextWords, points, align,
                                              annotation_text_utf8_offset(text, from),
                                              down ? 1 : 0, &goal);
  NSUInteger to = annotation_text_utf16_index(text, moved);
  if (!extend) view.lineAnchor = to;
  NSUInteger anchor = view.lineAnchor;
  view.settingLine = YES;
  [view setSelectedRange:NSMakeRange(MIN(anchor, to), MAX(anchor, to) - MIN(anchor, to))];
  view.settingLine = NO;
  view.lineRun = YES;
  view.lineGoal = goal;
  return YES;
}

- (void)moveUp:(id)sender {
  if (!move_line(self, NO, NO)) [super moveUp:sender];
}

- (void)moveDown:(id)sender {
  if (!move_line(self, YES, NO)) [super moveDown:sender];
}

- (void)moveUpAndModifySelection:(id)sender {
  if (!move_line(self, NO, YES)) [super moveUpAndModifySelection:sender];
}

- (void)moveDownAndModifySelection:(id)sender {
  if (!move_line(self, YES, YES)) [super moveDownAndModifySelection:sender];
}

/// Where the view's own layout is not what is drawn, the interaction view
/// takes every press in the box - carried back through the turn on a turned
/// layer, placed by Rust's layout where words are set apart - and says
/// itself where the I-beam shows.
- (NSView *)hitTest:(NSPoint)point {
  return annotation_text_routed(self.surface) ? nil : [super hitTest:point];
}

- (void)resetCursorRects {
  if (!annotation_text_routed(self.surface)) [super resetCursorRects];
}

- (void)cursorUpdate:(NSEvent *)event {
  if (!annotation_text_routed(self.surface)) [super cursorUpdate:event];
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

