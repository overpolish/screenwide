// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import <AppKit/AppKit.h>
#import <CoreText/CoreText.h>
#include <math.h>

#import "gpu_compositor_macos_annotation_text.h"
#import "gpu_compositor_macos_annotation_types.h"

void screenwide_register_inter_font(void) {
  static dispatch_once_t once;
  dispatch_once(&once, ^{
    NSURL *url = [[NSBundle mainBundle]
        URLForResource:@"Inter-VariableFont_opsz,wght"
         withExtension:@"ttf"
          subdirectory:@"fonts"];
    if (url != nil)
      CTFontManagerRegisterFontsForURL((__bridge CFURLRef)url,
                                       kCTFontManagerScopeProcess, NULL);
  });
}

/// The weight a counter's number is set at, named and as a variation.
///
/// The bundled Inter is one variable face carrying nine named instances on a
/// `wght` axis, so its semibold has to be asked for by name - a weight
/// *trait* on the descriptor only picks between separate family members, of
/// which there are none, and leaves the number at regular. The variation is
/// the fallback for a system that matched the family but not the instance.
static const uint32_t kScreenwideWeightAxis = 'wght';
static const CGFloat kScreenwideCounterWeight = 600.0;

/// Tabular figures, so 1 and 8 take the same room and a number does not
/// shift its weight as it grows.
static NSArray *counter_figures(void) {
  return @[ @{
    NSFontFeatureTypeIdentifierKey : @(kNumberSpacingType),
    NSFontFeatureSelectorIdentifierKey : @(kMonospacedNumbersSelector),
  } ];
}

/// The face a counter's number is set in.
static NSFont *counter_font(CGFloat size) {
  screenwide_register_inter_font();
  NSFontDescriptor *named = [NSFontDescriptor fontDescriptorWithFontAttributes:@{
    NSFontFamilyAttribute : @"Inter",
    NSFontFaceAttribute : @"SemiBold",
    NSFontFeatureSettingsAttribute : counter_figures(),
  }];
  NSFont *font = [NSFont fontWithDescriptor:named size:size];
  if (font != nil &&
      [font.fontName rangeOfString:@"SemiBold"].location != NSNotFound)
    return font;
  NSFont *base = [NSFont fontWithName:@"Inter" size:size];
  if (base == nil)
    return [NSFont systemFontOfSize:size weight:NSFontWeightSemibold];
  NSFontDescriptor *varied = [base.fontDescriptor fontDescriptorByAddingAttributes:@{
    (__bridge NSString *)kCTFontVariationAttribute :
        @{@(kScreenwideWeightAxis) : @(kScreenwideCounterWeight)},
    NSFontFeatureSettingsAttribute : counter_figures(),
  }];
  return [NSFont fontWithDescriptor:varied size:size] ?: base;
}

static NSDictionary *counter_dress(CGFloat size) {
  // White, because the kernel tints the number to whatever reads on the
  // disc's own colour.
  return @{
    NSFontAttributeName : counter_font(size),
    NSForegroundColorAttributeName : [NSColor whiteColor],
  };
}

/// One number as it will be rasterised: everything in atlas pixels, which are
/// [`SCREENWIDE_COUNTER_TEXT_SUPERSAMPLE`] to the drawn pixel.
typedef struct {
  uint32_t mark;
  NSDictionary *dress;
  NSString *text;
  CGSize measured;
  CGSize cell;
} ScreenwideCounterText;

/// The dress and the room one number needs, in atlas pixels. A number too
/// wide for its disc is narrowed rather than allowed to touch the edge: three
/// digits in a small disc still read, and the disc stays the size the style
/// asked for.
static ScreenwideCounterText counter_text(uint32_t value, float radius) {
  CGFloat supersample = SCREENWIDE_COUNTER_TEXT_SUPERSAMPLE;
  CGFloat diameter = radius * 2.0 * supersample;
  CGFloat size = diameter * SCREENWIDE_COUNTER_TEXT_CAP_SHARE /
                 SCREENWIDE_COUNTER_CAP_HEIGHT;
  ScreenwideCounterText result = {
      .text = [NSString stringWithFormat:@"%u", value],
  };
  result.dress = counter_dress(size);
  result.measured = [result.text sizeWithAttributes:result.dress];
  CGFloat limit = diameter * SCREENWIDE_COUNTER_TEXT_WIDTH_SHARE;
  if (result.measured.width > limit && result.measured.width > 0.0) {
    result.dress = counter_dress(size * limit / result.measured.width);
    result.measured = [result.text sizeWithAttributes:result.dress];
  }
  // One transparent pixel of margin keeps a four-tap sample on one number.
  result.cell = CGSizeMake(ceil(result.measured.width) + 2.0,
                           ceil(result.measured.height) + 2.0);
  return result;
}

@interface ScreenwideAnnotationTextAtlas : NSObject
@property(nonatomic, strong) id<MTLBuffer> pixels;
@property(nonatomic, strong) NSData *rects;
@property(nonatomic) ScreenwideAnnotationTextUniforms uniforms;
@end

@implementation ScreenwideAnnotationTextAtlas
@end

/// The atlas is keyed on exactly what it was drawn from, so an unchanged
/// preview frame is a dictionary lookup. A counter being dragged or animated
/// walks through its own sizes, and the whole cache is dropped once it fills
/// rather than tracked by age, as the keyboard artwork's is.
static NSMutableDictionary<NSString *, ScreenwideAnnotationTextAtlas *> *atlas_cache(void) {
  static NSMutableDictionary *cache;
  static dispatch_once_t once;
  dispatch_once(&once, ^{
    cache = [NSMutableDictionary dictionary];
  });
  return cache;
}

id<MTLBuffer> screenwide_annotation_text_atlas(
    id<MTLDevice> device, const uint32_t *values, const float *radii,
    uint32_t count, ScreenwideAnnotationTextRect *rects,
    ScreenwideAnnotationTextUniforms *uniforms) {
  if (count > SCREENWIDE_MAX_ANNOTATIONS) count = SCREENWIDE_MAX_ANNOTATIONS;
  if (rects != NULL)
    memset(rects, 0, sizeof(ScreenwideAnnotationTextRect) * count);
  if (uniforms != NULL) *uniforms = (ScreenwideAnnotationTextUniforms){0};
  if (device == nil || values == NULL || radii == NULL || count == 0) return nil;

  // A number smaller than a pixel across has nothing to rasterise; the disc
  // it belongs to is mid-arrival and is drawn without it for a frame or two.
  NSMutableString *key = [NSMutableString stringWithString:@"counter"];
  uint32_t wanted = 0;
  for (uint32_t index = 0; index < count; index++) {
    if (values[index] == 0 || !(radii[index] > 1.0f)) continue;
    // Quantised to a quarter pixel, so a preview nudged by rounding reuses
    // the atlas it already has.
    [key appendFormat:@"|%u@%.2f", values[index],
                      roundf(radii[index] * 4.0f) / 4.0f];
    wanted++;
  }
  if (wanted == 0) return nil;

  ScreenwideAnnotationTextAtlas *known = atlas_cache()[key];
  if (known != nil &&
      known.rects.length == sizeof(ScreenwideAnnotationTextRect) * count) {
    if (rects != NULL) memcpy(rects, known.rects.bytes, known.rects.length);
    if (uniforms != NULL) *uniforms = known.uniforms;
    return known.pixels;
  }

  ScreenwideCounterText prepared[SCREENWIDE_MAX_ANNOTATIONS];
  uint32_t rows = 0;
  CGFloat atlasWidth = 0.0;
  CGFloat atlasHeight = 0.0;
  for (uint32_t index = 0; index < count; index++) {
    if (values[index] == 0 || !(radii[index] > 1.0f)) continue;
    prepared[rows] = counter_text(values[index], radii[index]);
    prepared[rows].mark = index;
    atlasWidth = MAX(atlasWidth, prepared[rows].cell.width);
    atlasHeight += prepared[rows].cell.height;
    rows++;
  }
  NSUInteger pixelWidth = MAX((NSUInteger)atlasWidth, 1u);
  NSUInteger pixelHeight = MAX((NSUInteger)atlasHeight, 1u);
  CGColorSpaceRef space = CGColorSpaceCreateWithName(kCGColorSpaceSRGB);
  CGContextRef context = CGBitmapContextCreate(
      NULL, pixelWidth, pixelHeight, 8, pixelWidth * 4, space,
      (CGBitmapInfo)kCGImageAlphaPremultipliedLast | kCGBitmapByteOrder32Big);
  CGColorSpaceRelease(space);
  if (context == NULL) return nil;
  // Grayscale antialiasing, as the keyboard artwork and the OSC text use: the
  // number is tinted by the kernel, so subpixel coverage would be wrong.
  CGContextSetShouldSmoothFonts(context, false);
  NSGraphicsContext *graphics =
      [NSGraphicsContext graphicsContextWithCGContext:context flipped:NO];
  [NSGraphicsContext saveGraphicsState];
  [NSGraphicsContext setCurrentContext:graphics];
  NSMutableData *placed =
      [NSMutableData dataWithLength:sizeof(ScreenwideAnnotationTextRect) * count];
  ScreenwideAnnotationTextRect *written = placed.mutableBytes;
  CGFloat top = 0.0;
  for (uint32_t row = 0; row < rows; row++) {
    ScreenwideCounterText text = prepared[row];
    // The context is not flipped, so a row is drawn from the bottom up while
    // the rectangle handed back is in the top-down space the kernels sample.
    CGFloat bottom = (CGFloat)pixelHeight - top - text.cell.height;
    [text.text drawAtPoint:NSMakePoint(
                               (text.cell.width - text.measured.width) * 0.5,
                               bottom + (text.cell.height - text.measured.height) * 0.5)
            withAttributes:text.dress];
    written[text.mark] = (ScreenwideAnnotationTextRect){
        .x = 0.0f,
        .y = (float)top,
        .width = (float)text.cell.width,
        .height = (float)text.cell.height,
    };
    top += text.cell.height;
  }
  [NSGraphicsContext restoreGraphicsState];
  id<MTLBuffer> pixels =
      [device newBufferWithBytes:CGBitmapContextGetData(context)
                          length:pixelWidth * pixelHeight * 4
                         options:MTLResourceStorageModeShared];
  CGContextRelease(context);
  if (pixels == nil) return nil;
  ScreenwideAnnotationTextAtlas *atlas = [ScreenwideAnnotationTextAtlas new];
  atlas.pixels = pixels;
  atlas.rects = placed;
  atlas.uniforms = (ScreenwideAnnotationTextUniforms){(uint32_t)pixelWidth,
                                                      (uint32_t)pixelHeight};
  if (rects != NULL) memcpy(rects, written, placed.length);
  if (uniforms != NULL) *uniforms = atlas.uniforms;
  NSMutableDictionary *cache = atlas_cache();
  if (cache.count >= 64) [cache removeAllObjects];
  cache[key] = atlas;
  return pixels;
}
