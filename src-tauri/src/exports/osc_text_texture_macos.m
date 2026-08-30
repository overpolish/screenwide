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

ScreenwideOscTextTexture *screenwide_osc_text_texture(
    id<MTLDevice> device, NSString *text, CGFloat scale,
    uint32_t light_mode, CGFloat font_size, CGFloat line_height) {
  if (!device || text.length == 0 || scale <= 0.0 || font_size <= 0.0)
    return nil;
  register_inter_font();
  NSFont *inter = [NSFont fontWithName:@"Inter" size:font_size];
  NSFontDescriptor *descriptor = [inter.fontDescriptor
      fontDescriptorByAddingAttributes:@{
        NSFontTraitsAttribute : @{ NSFontWeightTrait : @(NSFontWeightSemibold) },
      }];
  NSFont *font = [NSFont fontWithDescriptor:descriptor size:font_size];
  if (!font)
    font = [NSFont systemFontOfSize:font_size weight:NSFontWeightSemibold];
  NSColor *fill = light_mode != 0
      ? [NSColor colorWithSRGBRed:0.149 green:0.149 blue:0.149 alpha:1.0]
      : NSColor.whiteColor;
  NSDictionary *attributes = @{
    NSFontAttributeName : font,
    NSForegroundColorAttributeName : fill,
  };
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
