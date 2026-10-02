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

/// One line set in the text box's face, ready to measure.
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
