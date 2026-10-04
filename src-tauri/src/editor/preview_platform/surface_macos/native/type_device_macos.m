// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/// Inter SemiBold set by Core Text, for the annotation atlas Rust lays out:
/// the face's metrics at a size, how far a line advances, and lines drawn as
/// coverage; and an emoji set in Apple Color Emoji, drawn in its own colours
/// for a sticker. The twin of DirectWrite's `type_device` on Windows.

#import <AppKit/AppKit.h>
#import <CoreText/CoreText.h>
#include <math.h>

#import "../../../cursor_export/macos/gpu_compositor/gpu_compositor_macos_annotation_text_box.h"

typedef struct {
  double x;
  double y;
  const uint8_t *text;
  uint32_t length;
} ScreenwideTypeLine;

/// One line in the face, coloured by whatever fill the context has.
static CTLineRef type_line(const uint8_t *text, uint32_t length, NSFont *font) {
  NSString *value = [[NSString alloc] initWithBytes:text
                                             length:length
                                           encoding:NSUTF8StringEncoding];
  if (value.length == 0) return NULL;
  NSAttributedString *attributed = [[NSAttributedString alloc]
      initWithString:value
          attributes:@{
            NSFontAttributeName : font,
            (__bridge NSString *)kCTForegroundColorFromContextAttributeName : @YES,
          }];
  return CTLineCreateWithAttributedString((__bridge CFAttributedStringRef)attributed);
}

void screenwide_type_metrics(double size, uint32_t tabular, double *ascent, double *descent) {
  @autoreleasepool {
    NSFont *font = screenwide_annotation_font(size, tabular != 0);
    *ascent = font.ascender;
    *descent = -font.descender;
  }
}

double screenwide_type_advance(const uint8_t *text, uint32_t length, double size,
                               uint32_t tabular) {
  if (text == NULL || length == 0 || !(size > 0.0)) return 0.0;
  @autoreleasepool {
    CTLineRef line = type_line(text, length, screenwide_annotation_font(size, tabular != 0));
    if (line == NULL) return 0.0;
    double width = CTLineGetTypographicBounds(line, NULL, NULL, NULL);
    CFRelease(line);
    return width;
  }
}

/// `lines`, each with its top-left at the point given and its baseline the
/// ascent below that, drawn white over black into `coverage`: `width` by
/// `height` bytes, top row first. Returns 0 when no context could be made.
uint32_t screenwide_type_draw(double size, uint32_t tabular, uint32_t width, uint32_t height,
                              const ScreenwideTypeLine *lines, uint32_t count,
                              uint8_t *coverage) {
  if (coverage == NULL || width == 0 || height == 0) return 0;
  memset(coverage, 0, (size_t)width * height);
  @autoreleasepool {
    CGColorSpaceRef gray = CGColorSpaceCreateDeviceGray();
    CGContextRef context = CGBitmapContextCreate(coverage, width, height, 8, width, gray,
                                                 (CGBitmapInfo)kCGImageAlphaNone);
    CGColorSpaceRelease(gray);
    if (context == NULL) return 0;
    CGContextSetGrayFillColor(context, 1.0, 1.0);
    CGContextSetAllowsAntialiasing(context, true);
    CGContextSetShouldAntialias(context, true);
    // Plain greyscale coverage, placed where the layout put it: no font
    // smoothing to thicken it, and no snapping to whole pixels.
    CGContextSetShouldSmoothFonts(context, false);
    CGContextSetAllowsFontSubpixelPositioning(context, true);
    CGContextSetShouldSubpixelPositionFonts(context, true);
    CGContextSetAllowsFontSubpixelQuantization(context, false);
    CGContextSetShouldSubpixelQuantizeFonts(context, false);
    NSFont *font = screenwide_annotation_font(size, tabular != 0);
    double ascent = font.ascender;
    for (uint32_t index = 0; index < count && lines != NULL; index++) {
      if (lines[index].text == NULL || lines[index].length == 0) continue;
      CTLineRef line = type_line(lines[index].text, lines[index].length, font);
      if (line == NULL) continue;
      // Core Graphics counts up from the bottom row.
      CGContextSetTextPosition(context, lines[index].x,
                               (double)height - (lines[index].y + ascent));
      CTLineDraw(line, context);
      CFRelease(line);
    }
    CGContextFlush(context);
    CGContextRelease(context);
    return 1;
  }
}

/// `text` set in Apple Color Emoji at `size`, its line's top-left at `x`, `y`
/// and its baseline the font's ascent below that, drawn in its own colours
/// into `pixels`: `width` by `height` premultiplied BGRA pixels, top row
/// first, clear around it. Returns 0 when nothing could be drawn.
uint32_t screenwide_emoji_draw(const uint8_t *text, uint32_t length, double size,
                               uint32_t width, uint32_t height, double x, double y,
                               uint8_t *pixels) {
  if (text == NULL || length == 0 || pixels == NULL || width == 0 || height == 0 ||
      !(size > 0.0))
    return 0;
  memset(pixels, 0, (size_t)width * height * 4);
  @autoreleasepool {
    NSString *value = [[NSString alloc] initWithBytes:text
                                               length:length
                                             encoding:NSUTF8StringEncoding];
    if (value.length == 0) return 0;
    CTFontRef font = CTFontCreateWithName(CFSTR("AppleColorEmoji"), size, NULL);
    if (font == NULL) return 0;
    NSAttributedString *attributed = [[NSAttributedString alloc]
        initWithString:value
            attributes:@{(__bridge NSString *)kCTFontAttributeName : (__bridge id)font}];
    CTLineRef line = CTLineCreateWithAttributedString((__bridge CFAttributedStringRef)attributed);
    double ascent = CTFontGetAscent(font);
    CFRelease(font);
    if (line == NULL) return 0;
    CGColorSpaceRef space = CGColorSpaceCreateWithName(kCGColorSpaceSRGB);
    // Little-endian, alpha first: the bytes run blue, green, red, alpha.
    CGContextRef context = CGBitmapContextCreate(
        pixels, width, height, 8, (size_t)width * 4, space,
        (CGBitmapInfo)kCGImageAlphaPremultipliedFirst | kCGBitmapByteOrder32Little);
    CGColorSpaceRelease(space);
    if (context == NULL) {
      CFRelease(line);
      return 0;
    }
    CGContextSetAllowsAntialiasing(context, true);
    CGContextSetShouldAntialias(context, true);
    CGContextSetInterpolationQuality(context, kCGInterpolationHigh);
    // Core Graphics counts up from the bottom row.
    CGContextSetTextPosition(context, x, (double)height - (y + ascent));
    CTLineDraw(line, context);
    CFRelease(line);
    CGContextFlush(context);
    CGContextRelease(context);
    return 1;
  }
}
