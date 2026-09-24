// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import <AppKit/AppKit.h>
#import <CoreText/CoreText.h>
#include <math.h>

#import "gpu_compositor_macos_annotation_text_box.h"

NSParagraphStyle *screenwide_text_box_paragraph(CGFloat size, uint32_t alignment) {
  NSMutableParagraphStyle *paragraph = [[NSMutableParagraphStyle alloc] init];
  CGFloat line = size * SCREENWIDE_TEXT_BOX_LINE_HEIGHT;
  paragraph.minimumLineHeight = line;
  paragraph.maximumLineHeight = line;
  paragraph.alignment = alignment == 1   ? NSTextAlignmentCenter
                        : alignment == 2 ? NSTextAlignmentRight
                                         : NSTextAlignmentLeft;
  return paragraph;
}

/// One line set in the text box's face, ready to measure or draw. The colour
/// comes from the context, so a raster can draw it white for the kernel to
/// tint.
static CTLineRef text_box_line(NSString *line, NSFont *font) {
  NSAttributedString *attributed = [[NSAttributedString alloc]
      initWithString:line
          attributes:@{
            NSFontAttributeName : font,
            (__bridge NSString *)kCTForegroundColorFromContextAttributeName : @YES,
          }];
  return CTLineCreateWithAttributedString((__bridge CFAttributedStringRef)attributed);
}

static double line_width(NSString *line, NSFont *font) {
  if (line.length == 0) return 0.0;
  CTLineRef set = text_box_line(line, font);
  double width = CTLineGetTypographicBounds(set, NULL, NULL, NULL);
  CFRelease(set);
  return width;
}

static NSArray<NSString *> *text_box_lines(const char *text, uint32_t length) {
  if (text == NULL || length == 0) return @[];
  NSString *value = [[NSString alloc] initWithBytes:text
                                             length:length
                                           encoding:NSUTF8StringEncoding];
  return [value componentsSeparatedByString:@"\n"] ?: @[];
}

double screenwide_text_box_line_width(const uint8_t *text, uint32_t length, double font) {
  if (text == NULL || length == 0 || !(font > 0.0)) return 0.0;
  @autoreleasepool {
    NSString *line = [[NSString alloc] initWithBytes:text
                                              length:length
                                            encoding:NSUTF8StringEncoding];
    if (line == nil) return 0.0;
    return line_width(line, screenwide_annotation_font(font, NO));
  }
}

/// The block at `size` atlas pixels: its lines, their widths and the widest.
typedef struct {
  NSArray<NSString *> *lines;
  NSFont *font;
  CGFloat width;
  CGFloat line;
} ScreenwideTextBlock;

static ScreenwideTextBlock text_block(const char *text, uint32_t length, CGFloat size) {
  ScreenwideTextBlock block = {
      .lines = text_box_lines(text, length),
      .font = screenwide_annotation_font(size, NO),
      .width = 0.0,
      .line = size * SCREENWIDE_TEXT_BOX_LINE_HEIGHT,
  };
  for (NSString *line in block.lines)
    block.width = MAX(block.width, line_width(line, block.font));
  return block;
}

uint32_t screenwide_text_box_cell(const char *text, uint32_t length, float size,
                                  uint32_t *width, uint32_t *height) {
  if (!(size > 1.0f)) return 0;
  @autoreleasepool {
    ScreenwideTextBlock block =
        text_block(text, length, size * SCREENWIDE_COUNTER_TEXT_SUPERSAMPLE);
    if (block.width <= 0.0) return 0;
    // One transparent pixel of margin each side keeps the block's edge off
    // the cell's.
    *width = (uint32_t)ceil(block.width) + 2;
    *height = (uint32_t)ceil(block.line * block.lines.count) + 2;
    return 1;
  }
}

void screenwide_text_box_draw(uint8_t *pixels, uint32_t atlas_width,
                              ScreenwideAnnotationTextRect rect, const char *text,
                              uint32_t length, float size, uint32_t alignment) {
  @autoreleasepool {
    ScreenwideTextBlock block =
        text_block(text, length, size * SCREENWIDE_COUNTER_TEXT_SUPERSAMPLE);
    CGContextRef context = screenwide_annotation_cell_context(rect);
    if (context == NULL) return;
    CGContextSetRGBFillColor(context, 1.0, 1.0, 1.0, 1.0);
    CGFloat ascent = CTFontGetAscent((__bridge CTFontRef)block.font);
    CGFloat descent = CTFontGetDescent((__bridge CTFontRef)block.font);
    CGFloat share = alignment == 1 ? 0.5 : alignment == 2 ? 1.0 : 0.0;
    // Each line sits centred in its own fixed-height band, counted from the
    // top of the block; Core Graphics counts up from the bottom.
    for (NSUInteger index = 0; index < block.lines.count; index++) {
      NSString *line = block.lines[index];
      if (line.length == 0) continue;
      CTLineRef set = text_box_line(line, block.font);
      CGFloat width = CTLineGetTypographicBounds(set, NULL, NULL, NULL);
      CGFloat baseline = 1.0 + block.line * index +
                         (block.line - (ascent + descent)) * 0.5 + ascent;
      CGContextSetTextPosition(context, 1.0 + (block.width - width) * share,
                               rect.height - baseline);
      CTLineDraw(set, context);
      CFRelease(set);
    }
    screenwide_annotation_cell_copy(context, pixels, atlas_width, rect);
  }
}
