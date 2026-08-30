// SPDX-License-Identifier: GPL-3.0-or-later

#import "screenshot_region_osc_macos_private.h"

id<MTLTexture> screenwide_osc_icon_texture(id<MTLDevice> device) {
  static NSMapTable<id<MTLDevice>, id<MTLTexture>> *cache;
  static dispatch_once_t once;
  dispatch_once(&once, ^{
    cache = [NSMapTable strongToStrongObjectsMapTable];
  });
  @synchronized(cache) {
    id<MTLTexture> cached = [cache objectForKey:device];
    if (cached)
      return cached;
    ScreenwideOscIconAtlas atlas = screenwide_osc_icon_atlas();
    if (!atlas.pixels || atlas.length < (size_t)atlas.width * atlas.height)
      return nil;
    MTLTextureDescriptor *descriptor = [MTLTextureDescriptor
        texture2DDescriptorWithPixelFormat:MTLPixelFormatR8Unorm
                                     width:atlas.width
                                    height:atlas.height
                                 mipmapped:NO];
    descriptor.usage = MTLTextureUsageShaderRead;
    id<MTLTexture> texture = [device newTextureWithDescriptor:descriptor];
    [texture replaceRegion:MTLRegionMake2D(0, 0, atlas.width, atlas.height)
               mipmapLevel:0
                 withBytes:atlas.pixels
               bytesPerRow:atlas.width];
    [cache setObject:texture forKey:device];
    return texture;
  }
}

void screenwide_region_osc_add_icon(ScreenwideRegionOscVertex *vertices,
                                    NSUInteger *count, NSSize size,
                                    uint8_t icon, CGFloat left, CGFloat top,
                                    CGFloat icon_size) {
  if (icon == 0)
    return;
  screenwide_region_osc_add_quad(
      vertices, count, size,
      NSMakeRect(left, top, icon_size, icon_size), 21 + icon);
}
