// SPDX-License-Identifier: GPL-3.0-or-later

#import <AppKit/AppKit.h>
#import <Metal/Metal.h>

@interface ScreenwideOscTextTexture : NSObject
@property(nonatomic, strong) id<MTLTexture> texture;
@property(nonatomic) NSSize size;
@end

ScreenwideOscTextTexture *screenwide_osc_text_texture(
    id<MTLDevice> device, NSString *text, CGFloat scale,
    uint32_t light_mode, CGFloat font_size, CGFloat line_height);
