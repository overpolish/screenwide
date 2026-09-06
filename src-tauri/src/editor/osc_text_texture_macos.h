// SPDX-License-Identifier: GPL-3.0-or-later

#import <AppKit/AppKit.h>
#import <Metal/Metal.h>

@interface ScreenwideOscTextTexture : NSObject
@property(nonatomic, strong) id<MTLTexture> texture;
@property(nonatomic) NSSize size;
@property(nonatomic) CGFloat atlasGlyphWidth;
@property(nonatomic) CGFloat atlasGlyphUOffset;
@property(nonatomic) CGFloat atlasGlyphUWidth;
@end

ScreenwideOscTextTexture *screenwide_osc_text_texture(
    id<MTLDevice> device, NSString *text, CGFloat scale,
    uint32_t light_mode, CGFloat font_size, CGFloat line_height);
ScreenwideOscTextTexture *screenwide_osc_mono_text_texture(
    id<MTLDevice> device, NSString *text, CGFloat scale,
    uint32_t light_mode, CGFloat font_size, CGFloat line_height);
ScreenwideOscTextTexture *screenwide_osc_mono_hex_atlas(
    id<MTLDevice> device, CGFloat scale, uint32_t light_mode,
    CGFloat font_size, CGFloat line_height);
