// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import <CoreText/CoreText.h>

#import "recording_preview_surface_macos_private.h"

static const CGFloat ScreenwideSelectionLabelFontSize = 11.0;
static const CGFloat ScreenwideSelectionActionFontSize = 12.0;
static const CGFloat ScreenwideSelectionLabelStroke = 2.0;

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

/// (Re)builds `selectionLabelTexture` for `text`. Returns NO when the bitmap
/// could not be produced, in which case no label must be drawn.
///
/// Rasterised exactly like Keyframeless's OSC label: dimensions use a
/// monospaced face while action text uses the app's proportional UI family.
/// Dimensions receive the contrast halo; action text is drawn cleanly over its
/// button fill, matching the React component.
static BOOL update_selection_label(ScreenwidePreviewSurface *surface,
                                   NSString *text, CGFloat scale,
                                   uint32_t lightMode, BOOL action,
                                   BOOL secondary) {
  if (surface.device == nil || text.length == 0) return NO;
  if (!secondary && (surface.selectionLabelScale != scale ||
                     surface.selectionLabelLightMode != lightMode))
    surface.selectionSecondaryLabelTexture = nil;
  id<MTLTexture> knownTexture = secondary
      ? surface.selectionSecondaryLabelTexture : surface.selectionLabelTexture;
  NSString *knownText = secondary
      ? surface.selectionSecondaryLabelText : surface.selectionLabelText;
  if (knownTexture != nil &&
      surface.selectionLabelScale == scale &&
      surface.selectionLabelLightMode == lightMode &&
      [knownText isEqualToString:text])
    return YES;

  NSColor *fill = lightMode != 0
      ? [NSColor colorWithSRGBRed:0.149 green:0.149 blue:0.149 alpha:1.0]
      : [NSColor colorWithSRGBRed:1.0 green:1.0 blue:1.0 alpha:1.0];
  NSColor *halo = lightMode != 0
      ? [NSColor colorWithSRGBRed:1.0 green:1.0 blue:1.0 alpha:1.0]
      : [NSColor colorWithSRGBRed:0.0 green:0.0 blue:0.0 alpha:0.8];
  NSFont *font;
  if (action) {
    register_inter_font();
    NSFont *inter = [NSFont fontWithName:@"Inter"
                                   size:ScreenwideSelectionActionFontSize];
    NSFontDescriptor *descriptor = [inter.fontDescriptor
        fontDescriptorByAddingAttributes:@{
          NSFontTraitsAttribute : @{
            NSFontWeightTrait : @(NSFontWeightSemibold),
          },
        }];
    font = [NSFont fontWithDescriptor:descriptor
                                size:ScreenwideSelectionActionFontSize];
    if (font == nil)
      font = [NSFont systemFontOfSize:ScreenwideSelectionActionFontSize
                              weight:NSFontWeightSemibold];
  } else {
    font = [NSFont monospacedSystemFontOfSize:ScreenwideSelectionLabelFontSize
                                       weight:NSFontWeightMedium];
  }
  // A positive stroke width strokes the glyph without filling it, so this pass
  // lays down only the outline the fill pass then sits inside.
  NSDictionary *strokeAttributes = @{
    NSFontAttributeName : font,
    NSForegroundColorAttributeName : halo,
    NSStrokeColorAttributeName : halo,
    NSStrokeWidthAttributeName :
        @(ScreenwideSelectionLabelStroke / ScreenwideSelectionLabelFontSize * 100.0),
  };
  NSDictionary *fillAttributes = @{
    NSFontAttributeName : font,
    NSForegroundColorAttributeName : fill,
  };
  NSSize textSize = [text sizeWithAttributes:fillAttributes];
  // The inset leaves room for the halo, which spills outside the glyph box.
  NSInteger pointWidth = (NSInteger)ceil(textSize.width) + 4;
  NSInteger pointHeight = action ? 16 : (NSInteger)ceil(textSize.height) + 2;
  NSInteger pixelWidth = (NSInteger)MAX(round(pointWidth * scale), 1.0);
  NSInteger pixelHeight = (NSInteger)MAX(round(pointHeight * scale), 1.0);

  CGColorSpaceRef space = CGColorSpaceCreateWithName(kCGColorSpaceSRGB);
  CGContextRef context = CGBitmapContextCreate(
      NULL, (size_t)pixelWidth, (size_t)pixelHeight, 8, (size_t)pixelWidth * 4,
      space,
      (CGBitmapInfo)kCGImageAlphaPremultipliedLast | kCGBitmapByteOrder32Big);
  CGColorSpaceRelease(space);
  if (context == NULL) return NO;
  CGContextScaleCTM(context, scale, scale);
  NSGraphicsContext *graphics =
      [NSGraphicsContext graphicsContextWithCGContext:context flipped:NO];
  [NSGraphicsContext saveGraphicsState];
  [NSGraphicsContext setCurrentContext:graphics];
  NSPoint textOrigin = NSMakePoint(
      2.0, action ? (pointHeight - textSize.height) / 2.0 : 1.0);
  if (!action)
    [text drawAtPoint:textOrigin withAttributes:strokeAttributes];
  [text drawAtPoint:textOrigin withAttributes:fillAttributes];
  [NSGraphicsContext restoreGraphicsState];

  MTLTextureDescriptor *descriptor = [MTLTextureDescriptor
      texture2DDescriptorWithPixelFormat:MTLPixelFormatRGBA8Unorm
                                   width:(NSUInteger)pixelWidth
                                  height:(NSUInteger)pixelHeight
                               mipmapped:NO];
  descriptor.usage = MTLTextureUsageShaderRead;
  id<MTLTexture> texture = [surface.device newTextureWithDescriptor:descriptor];
  if (texture == nil) {
    CGContextRelease(context);
    return NO;
  }
  [texture replaceRegion:MTLRegionMake2D(0, 0, (NSUInteger)pixelWidth,
                                         (NSUInteger)pixelHeight)
             mipmapLevel:0
               withBytes:CGBitmapContextGetData(context)
             bytesPerRow:(NSUInteger)pixelWidth * 4];
  CGContextRelease(context);

  if (secondary) {
    surface.selectionSecondaryLabelTexture = texture;
    surface.selectionSecondaryLabelText = [text copy];
    surface.selectionSecondaryLabelSize = NSMakeSize(pointWidth, pointHeight);
  } else {
    surface.selectionLabelTexture = texture;
    surface.selectionLabelText = [text copy];
    surface.selectionLabelSize = NSMakeSize(pointWidth, pointHeight);
  }
  surface.selectionLabelScale = scale;
  surface.selectionLabelLightMode = lightMode;
  return YES;
}

@implementation ScreenwidePreviewSurface (Label)

- (BOOL)updateSelectionLabel:(NSString *)text
                      scale:(CGFloat)scale
                  lightMode:(uint32_t)lightMode
                     action:(BOOL)action {
  return update_selection_label(self, text, scale, lightMode, action, NO);
}

- (BOOL)updateSelectionSecondaryLabel:(NSString *)text
                               scale:(CGFloat)scale
                           lightMode:(uint32_t)lightMode {
  return update_selection_label(self, text, scale, lightMode, YES, YES);
}

@end
