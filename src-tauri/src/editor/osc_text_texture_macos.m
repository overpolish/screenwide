// SPDX-License-Identifier: GPL-3.0-or-later

#import "osc_text_texture_macos.h"
#import "osc_controls.h"
#import <CoreText/CoreText.h>

@implementation ScreenwideOscTextTexture
@end

/// The cells the ruler assembles every readout from. Kept next to the atlas
/// builder so the cell order and the index lookup cannot drift apart.
static NSString *atlas_glyphs(void) {
  return @"#0123456789ABCDEF\u00D7 px\u2248";
}

NSUInteger screenwide_osc_atlas_glyph_index(unichar glyph) {
  NSRange found = [atlas_glyphs()
      rangeOfString:[NSString stringWithCharacters:&glyph length:1]];
  return found.location == NSNotFound ? 0 : found.location;
}

CGFloat screenwide_osc_atlas_advance(ScreenwideOscTextTexture *atlas,
                                     NSUInteger index) {
  NSData *advances = atlas.atlasAdvances;
  if (advances == nil || (index + 1) * sizeof(CGFloat) > advances.length)
    return atlas.atlasGlyphWidth;
  return ((const CGFloat *)advances.bytes)[index];
}

CGFloat screenwide_osc_atlas_pitch(ScreenwideOscTextTexture *atlas,
                                   NSString *glyphs) {
  CGFloat pitch = 0.0;
  for (NSUInteger index = 0; index < glyphs.length; index++)
    pitch = MAX(pitch, screenwide_osc_atlas_advance(
                           atlas, screenwide_osc_atlas_glyph_index(
                                      [glyphs characterAtIndex:index])));
  return pitch;
}

CGFloat screenwide_osc_atlas_text_width(ScreenwideOscTextTexture *atlas,
                                        NSString *text) {
  CGFloat width = 0.0;
  for (NSUInteger index = 0; index < text.length; index++)
    width += screenwide_osc_atlas_advance(
        atlas,
        screenwide_osc_atlas_glyph_index([text characterAtIndex:index]));
  return width;
}

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

// Native labels use the same body role as their React peers: Inter at the
// regular weight, never a display weight, so a native button reads like a
// window button rather than a badge.
static NSFont *text_font(CGFloat font_size, BOOL mono, BOOL tabular) {
  if (mono)
    register_roboto_mono_font();
  else
    register_inter_font();
  NSString *name = mono ? @"Roboto Mono" : @"Inter";
  NSFont *base = [NSFont fontWithName:name size:font_size];
  NSMutableDictionary *attributes = [@{
    NSFontTraitsAttribute : @{ NSFontWeightTrait : @(NSFontWeightRegular) },
  } mutableCopy];
  if (tabular)
    // Tabular figures give every digit the same advance, so a readout that
    // counts up does not jitter and a fixed cell holds any digit.
    attributes[NSFontFeatureSettingsAttribute] = @[ @{
      NSFontFeatureTypeIdentifierKey : @(kNumberSpacingType),
      NSFontFeatureSelectorIdentifierKey : @(kMonospacedNumbersSelector),
    } ];
  NSFontDescriptor *descriptor =
      [base.fontDescriptor fontDescriptorByAddingAttributes:attributes];
  NSFont *font = [NSFont fontWithDescriptor:descriptor size:font_size];
  if (font)
    return font;
  return mono ? [NSFont monospacedSystemFontOfSize:font_size
                                           weight:NSFontWeightRegular]
              : [NSFont systemFontOfSize:font_size
                                  weight:NSFontWeightRegular];
}

NSFont *screenwide_osc_body_font(void) {
  return text_font(screenwide_osc_control_metrics(0, 0).font_size, NO, YES);
}

// The label tier from `src/index.css`: pure black or pure white at 85%, the
// same colour the control foreground resolves to for icons.
static NSDictionary *text_attributes(CGFloat font_size, BOOL mono,
                                      BOOL tabular, uint32_t light_mode) {
  NSColor *fill = light_mode != 0
      ? [NSColor colorWithSRGBRed:0.0 green:0.0 blue:0.0 alpha:0.85]
      : [NSColor colorWithSRGBRed:1.0 green:1.0 blue:1.0 alpha:0.85];
  return @{
    NSFontAttributeName : text_font(font_size, mono, tabular),
    NSForegroundColorAttributeName : fill,
  };
}

static ScreenwideOscTextTexture *text_texture(
    id<MTLDevice> device, NSString *text, CGFloat scale,
    uint32_t light_mode, CGFloat font_size, CGFloat line_height,
    BOOL mono) {
  if (!device || text.length == 0 || scale <= 0.0 || font_size <= 0.0)
    return nil;
  NSDictionary *attributes = text_attributes(font_size, mono, NO, light_mode);
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
  // Match the frontend's grayscale antialiasing without font smoothing weight.
  CGContextSetShouldSmoothFonts(context, false);
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
  // Rasterising the glyphs once avoids rebuilding a CoreText bitmap and Metal
  // texture for every sampled pixel while the pointer is moving.
  NSString *glyphs = atlas_glyphs();
  if (!device || scale <= 0.0 || font_size <= 0.0)
    return nil;
  NSDictionary *attributes = text_attributes(font_size, NO, YES, light_mode);
  // Tabular figures make the digits one width; the handful of symbols around
  // them are narrower or wider. The texture cell takes the widest advance and
  // centres each glyph in it, so one uv rectangle shape serves every cell,
  // while callers step on screen by the glyph's own advance.
#define SCREENWIDE_MAX_ATLAS_GLYPHS 64
  if (glyphs.length > SCREENWIDE_MAX_ATLAS_GLYPHS)
    return nil;
  CGFloat glyph_width = 0.0;
  CGFloat advances[SCREENWIDE_MAX_ATLAS_GLYPHS];
  for (NSUInteger index = 0; index < glyphs.length; index++) {
    NSString *glyph = [glyphs substringWithRange:NSMakeRange(index, 1)];
    advances[index] = [glyph sizeWithAttributes:attributes].width;
    glyph_width = MAX(glyph_width, advances[index]);
  }
  glyph_width = ceil(glyph_width);
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
  // Match the frontend's grayscale antialiasing without font smoothing weight.
  CGContextSetShouldSmoothFonts(context, false);
  CGContextScaleCTM(context, scale, scale);
  NSGraphicsContext *graphics =
      [NSGraphicsContext graphicsContextWithCGContext:context flipped:NO];
  [NSGraphicsContext saveGraphicsState];
  [NSGraphicsContext setCurrentContext:graphics];
  CGFloat text_height = [glyphs sizeWithAttributes:attributes].height;
  CGFloat y = (point_height - text_height) * 0.5;
  for (NSUInteger index = 0; index < glyphs.length; index++) {
    NSString *glyph = [glyphs substringWithRange:NSMakeRange(index, 1)];
    CGFloat x = (index * cell_pixel_width + gutter) / scale +
                (glyph_width - advances[index]) * 0.5;
    NSMutableDictionary *glyphAttributes = [attributes mutableCopy];
    if ([glyph isEqualToString:@"p"] || [glyph isEqualToString:@"x"])
      // NumberField decorations use content-fg-secondary in each appearance.
      glyphAttributes[NSForegroundColorAttributeName] =
          [attributes[NSForegroundColorAttributeName]
              colorWithAlphaComponent:light_mode != 0 ? 0.50 : 0.55];
    [glyph drawAtPoint:NSMakePoint(x, y) withAttributes:glyphAttributes];
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
  CGFloat total = 0.0;
  for (NSUInteger index = 0; index < glyphs.length; index++)
    total += advances[index];
  result.size = NSMakeSize(total, point_height);
  result.atlasGlyphWidth = glyph_width;
  // UVs describe cell edges; rasterization samples at pixel centres already.
  // Insetting by half a texel stretches N - 1 texels across N pixels and blurs.
  result.atlasGlyphUOffset = (CGFloat)gutter / pixel_width;
  result.atlasGlyphUWidth = (CGFloat)glyph_pixel_width / pixel_width;
  result.atlasAdvances = [NSData dataWithBytes:advances
                                        length:glyphs.length * sizeof(CGFloat)];
  return result;
}
