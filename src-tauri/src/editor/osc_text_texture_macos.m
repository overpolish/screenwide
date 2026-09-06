// SPDX-License-Identifier: GPL-3.0-or-later

#import "osc_text_texture_macos.h"
#import <CoreText/CoreText.h>

@implementation ScreenwideOscTextTexture
@end

static void register_inter_font(void) {
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

static void register_roboto_mono_font(void) {
  static dispatch_once_t once;
  dispatch_once(&once, ^{
    NSURL *url = [[NSBundle mainBundle]
        URLForResource:@"RobotoMono-VariableFont_wght"
         withExtension:@"ttf"
          subdirectory:@"fonts"];
    if (url != nil)
      CTFontManagerRegisterFontsForURL((__bridge CFURLRef)url,
                                      kCTFontManagerScopeProcess, NULL);
  });
}

static NSFont *text_font(CGFloat font_size, BOOL mono) {
  if (mono)
    register_roboto_mono_font();
  else
    register_inter_font();
  NSString *name = mono ? @"Roboto Mono" : @"Inter";
  NSFont *base = [NSFont fontWithName:name size:font_size];
  NSFontDescriptor *descriptor = [base.fontDescriptor
      fontDescriptorByAddingAttributes:@{
        NSFontTraitsAttribute : @{ NSFontWeightTrait : @(NSFontWeightSemibold) },
      }];
  NSFont *font = [NSFont fontWithDescriptor:descriptor size:font_size];
  if (font)
    return font;
  return mono ? [NSFont monospacedSystemFontOfSize:font_size
                                           weight:NSFontWeightSemibold]
              : [NSFont systemFontOfSize:font_size
                                  weight:NSFontWeightSemibold];
}

static NSDictionary *text_attributes(CGFloat font_size, BOOL mono,
                                      uint32_t light_mode) {
  NSColor *fill = light_mode != 0
      ? [NSColor colorWithSRGBRed:0.149 green:0.149 blue:0.149 alpha:1.0]
      : NSColor.whiteColor;
  return @{
    NSFontAttributeName : text_font(font_size, mono),
    NSForegroundColorAttributeName : fill,
  };
}

static ScreenwideOscTextTexture *text_texture(
    id<MTLDevice> device, NSString *text, CGFloat scale,
    uint32_t light_mode, CGFloat font_size, CGFloat line_height,
    BOOL mono) {
  if (!device || text.length == 0 || scale <= 0.0 || font_size <= 0.0)
    return nil;
  NSDictionary *attributes = text_attributes(font_size, mono, light_mode);
  NSSize measured = [text sizeWithAttributes:attributes];
  NSInteger point_width = MAX((NSInteger)ceil(measured.width), 1);
  NSInteger point_height = MAX((NSInteger)ceil(line_height), 1);
  NSInteger pixel_width = MAX((NSInteger)round(point_width * scale), 1);
  NSInteger pixel_height = MAX((NSInteger)round(point_height * scale), 1);
  CGColorSpaceRef space = CGColorSpaceCreateWithName(kCGColorSpaceSRGB);
  CGContextRef context = CGBitmapContextCreate(
      NULL, (size_t)pixel_width, (size_t)pixel_height, 8,
      (size_t)pixel_width * 4, space,
      (CGBitmapInfo)kCGImageAlphaPremultipliedLast |
          kCGBitmapByteOrder32Big);
  CGColorSpaceRelease(space);
  if (!context)
    return nil;
  CGContextScaleCTM(context, scale, scale);
  NSGraphicsContext *graphics =
      [NSGraphicsContext graphicsContextWithCGContext:context flipped:NO];
  [NSGraphicsContext saveGraphicsState];
  [NSGraphicsContext setCurrentContext:graphics];
  [text drawAtPoint:NSMakePoint(0.0, (point_height - measured.height) * 0.5)
      withAttributes:attributes];
  [NSGraphicsContext restoreGraphicsState];
  MTLTextureDescriptor *texture_descriptor = [MTLTextureDescriptor
      texture2DDescriptorWithPixelFormat:MTLPixelFormatRGBA8Unorm
                                   width:(NSUInteger)pixel_width
                                  height:(NSUInteger)pixel_height
                               mipmapped:NO];
  texture_descriptor.usage = MTLTextureUsageShaderRead;
  id<MTLTexture> texture = [device newTextureWithDescriptor:texture_descriptor];
  if (!texture) {
    CGContextRelease(context);
    return nil;
  }
  [texture replaceRegion:MTLRegionMake2D(0, 0, (NSUInteger)pixel_width,
                                         (NSUInteger)pixel_height)
             mipmapLevel:0
               withBytes:CGBitmapContextGetData(context)
             bytesPerRow:(NSUInteger)pixel_width * 4];
  CGContextRelease(context);
  ScreenwideOscTextTexture *result = [ScreenwideOscTextTexture new];
  result.texture = texture;
  result.size = NSMakeSize(point_width, point_height);
  return result;
}

ScreenwideOscTextTexture *screenwide_osc_text_texture(
    id<MTLDevice> device, NSString *text, CGFloat scale,
    uint32_t light_mode, CGFloat font_size, CGFloat line_height) {
  return text_texture(device, text, scale, light_mode, font_size,
                      line_height, NO);
}

ScreenwideOscTextTexture *screenwide_osc_mono_text_texture(
    id<MTLDevice> device, NSString *text, CGFloat scale,
    uint32_t light_mode, CGFloat font_size, CGFloat line_height) {
  return text_texture(device, text, scale, light_mode, font_size,
                      line_height, YES);
}

ScreenwideOscTextTexture *screenwide_osc_mono_hex_atlas(
    id<MTLDevice> device, CGFloat scale, uint32_t light_mode,
    CGFloat font_size, CGFloat line_height) {
  // Every ruler value is assembled from these fixed-width cells. Rasterising
  // the glyphs once avoids rebuilding a CoreText bitmap and Metal texture for
  // every sampled pixel while the pointer is moving.
  static NSString *glyphs = @"#0123456789ABCDEF× px≈";
  if (!device || scale <= 0.0 || font_size <= 0.0)
    return nil;
  NSDictionary *attributes = text_attributes(font_size, YES, light_mode);
  NSSize measured = [glyphs sizeWithAttributes:attributes];
  CGFloat glyph_width = ceil(measured.width) / glyphs.length;
  NSInteger glyph_pixel_width =
      MAX((NSInteger)ceil(glyph_width * scale), 1);
  const NSInteger gutter = 1;
  NSInteger cell_pixel_width = glyph_pixel_width + gutter * 2;
  NSInteger point_height = MAX((NSInteger)ceil(line_height), 1);
  NSInteger pixel_height = MAX((NSInteger)round(point_height * scale), 1);
  NSInteger pixel_width = cell_pixel_width * (NSInteger)glyphs.length;

  CGColorSpaceRef space = CGColorSpaceCreateWithName(kCGColorSpaceSRGB);
  CGContextRef context = CGBitmapContextCreate(
      NULL, (size_t)pixel_width, (size_t)pixel_height, 8,
      (size_t)pixel_width * 4, space,
      (CGBitmapInfo)kCGImageAlphaPremultipliedLast |
          kCGBitmapByteOrder32Big);
  CGColorSpaceRelease(space);
  if (!context)
    return nil;
  CGContextScaleCTM(context, scale, scale);
  NSGraphicsContext *graphics =
      [NSGraphicsContext graphicsContextWithCGContext:context flipped:NO];
  [NSGraphicsContext saveGraphicsState];
  [NSGraphicsContext setCurrentContext:graphics];
  CGFloat text_height = [glyphs sizeWithAttributes:attributes].height;
  CGFloat y = (point_height - text_height) * 0.5;
  for (NSUInteger index = 0; index < glyphs.length; index++) {
    NSString *glyph = [glyphs substringWithRange:NSMakeRange(index, 1)];
    CGFloat x = (index * cell_pixel_width + gutter) / scale;
    [glyph drawAtPoint:NSMakePoint(x, y) withAttributes:attributes];
  }
  [NSGraphicsContext restoreGraphicsState];

  MTLTextureDescriptor *descriptor = [MTLTextureDescriptor
      texture2DDescriptorWithPixelFormat:MTLPixelFormatRGBA8Unorm
                                   width:(NSUInteger)pixel_width
                                  height:(NSUInteger)pixel_height
                               mipmapped:NO];
  descriptor.usage = MTLTextureUsageShaderRead;
  id<MTLTexture> texture = [device newTextureWithDescriptor:descriptor];
  if (!texture) {
    CGContextRelease(context);
    return nil;
  }
  [texture replaceRegion:MTLRegionMake2D(0, 0, (NSUInteger)pixel_width,
                                         (NSUInteger)pixel_height)
             mipmapLevel:0
               withBytes:CGBitmapContextGetData(context)
             bytesPerRow:(NSUInteger)pixel_width * 4];
  CGContextRelease(context);

  ScreenwideOscTextTexture *result = [ScreenwideOscTextTexture new];
  result.texture = texture;
  result.size = NSMakeSize(glyph_width * glyphs.length, point_height);
  result.atlasGlyphWidth = glyph_width;
  // Sample from texel centre to texel centre. Linear filtering can then use
  // the transparent gutter outside the sampled range without either pulling
  // in the next glyph or trimming this glyph's edge pixels.
  result.atlasGlyphUOffset = ((CGFloat)gutter + 0.5) / pixel_width;
  result.atlasGlyphUWidth =
      (CGFloat)MAX(glyph_pixel_width - 1, 0) / pixel_width;
  return result;
}
