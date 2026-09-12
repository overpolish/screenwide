// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "gpu_compositor_macos_background_image.h"

/// How many decoded pictures stay resident. Two covers a canvas whose
/// background is being swapped back and forth without letting a session's
/// whole picture history sit in memory.
static const NSUInteger kScreenwideBackgroundImageLimit = 2;

/// One registered picture. The decoded pixels are kept so the texture can be
/// made again for another Metal device: every presenter and every export
/// creates its own device, and a texture belongs to exactly one of them.
@interface ScreenwideBackgroundImage : NSObject
@property(nonatomic) uint32_t identifier;
@property(nonatomic, strong) NSData *pixels;
@property(nonatomic) uint32_t width;
@property(nonatomic) uint32_t height;
@property(nonatomic, strong) NSMapTable<id<MTLDevice>, id<MTLTexture>> *textures;
@end

@implementation ScreenwideBackgroundImage
@end

/// Registered pictures, oldest first. Guarded by `registry_lock` because the
/// compositors call in from whichever thread runs their export or present.
static NSMutableArray<ScreenwideBackgroundImage *> *registry;
static NSMapTable<id<MTLDevice>, id<MTLTexture>> *placeholders;
static NSLock *registry_lock;

static void ensure_registry(void) {
  static dispatch_once_t once;
  dispatch_once(&once, ^{
    registry = [NSMutableArray array];
    placeholders = [NSMapTable weakToStrongObjectsMapTable];
    registry_lock = [NSLock new];
  });
}

/// A single transparent texel keeps the shader's texture slot bound when the
/// canvas paints no picture. The shader never samples it in that case.
static id<MTLTexture> placeholder_texture(id<MTLDevice> device) {
  id<MTLTexture> texture = [placeholders objectForKey:device];
  if (texture != nil) return texture;
  MTLTextureDescriptor *descriptor = [MTLTextureDescriptor
      texture2DDescriptorWithPixelFormat:MTLPixelFormatRGBA8Unorm
                                   width:1
                                  height:1
                               mipmapped:NO];
  descriptor.usage = MTLTextureUsageShaderRead;
  texture = [device newTextureWithDescriptor:descriptor];
  if (texture == nil) return nil;
  const uint8_t clear[4] = {0, 0, 0, 0};
  [texture replaceRegion:MTLRegionMake2D(0, 0, 1, 1)
             mipmapLevel:0
               withBytes:clear
             bytesPerRow:4];
  [placeholders setObject:texture forKey:device];
  return texture;
}

static id<MTLTexture> image_texture(ScreenwideBackgroundImage *image,
                                    id<MTLDevice> device) {
  id<MTLTexture> texture = [image.textures objectForKey:device];
  if (texture != nil) return texture;
  MTLTextureDescriptor *descriptor = [MTLTextureDescriptor
      texture2DDescriptorWithPixelFormat:MTLPixelFormatRGBA8Unorm
                                   width:image.width
                                  height:image.height
                               mipmapped:NO];
  descriptor.usage = MTLTextureUsageShaderRead;
  texture = [device newTextureWithDescriptor:descriptor];
  if (texture == nil) return nil;
  [texture replaceRegion:MTLRegionMake2D(0, 0, image.width, image.height)
             mipmapLevel:0
               withBytes:image.pixels.bytes
             bytesPerRow:(NSUInteger)image.width * 4];
  [image.textures setObject:texture forKey:device];
  return texture;
}

int screenwide_gpu_register_background_image(uint32_t identifier,
                                             const uint8_t *rgba,
                                             uint32_t width, uint32_t height) {
  if (rgba == NULL || width == 0 || height == 0) return 0;
  @autoreleasepool {
    ensure_registry();
    [registry_lock lock];
    for (ScreenwideBackgroundImage *image in registry) {
      if (image.identifier == identifier) {
        [registry_lock unlock];
        return 1;
      }
    }
    ScreenwideBackgroundImage *image = [ScreenwideBackgroundImage new];
    image.identifier = identifier;
    image.width = width;
    image.height = height;
    image.pixels = [NSData dataWithBytes:rgba
                                  length:(NSUInteger)width * height * 4];
    image.textures = [NSMapTable weakToStrongObjectsMapTable];
    if (image.pixels == nil) {
      [registry_lock unlock];
      return 0;
    }
    while (registry.count >= kScreenwideBackgroundImageLimit)
      [registry removeObjectAtIndex:0];
    [registry addObject:image];
    [registry_lock unlock];
    return 1;
  }
}

static id<MTLTexture> background_image_texture(id<MTLDevice> device,
                                              ScreenwideCanvas *canvas) {
  if (device == nil) return nil;
  ensure_registry();
  [registry_lock lock];
  id<MTLTexture> texture = nil;
  if (canvas != NULL && canvas->has_background_image != 0) {
    for (ScreenwideBackgroundImage *image in registry) {
      if (image.identifier != canvas->background_image_id) continue;
      texture = image_texture(image, device);
      break;
    }
  }
  // A picture that is gone must never break a canvas: drop back to the mesh
  // or solid colour the uniforms already carry.
  if (texture == nil) {
    if (canvas != NULL) canvas->has_background_image = 0;
    texture = placeholder_texture(device);
  }
  [registry_lock unlock];
  return texture;
}

void screenwide_gpu_bind_canvas(id<MTLComputeCommandEncoder> encoder,
                                id<MTLDevice> device,
                                const ScreenwideCanvas *canvas,
                                NSUInteger buffer_index,
                                NSUInteger texture_index) {
  if (encoder == nil || canvas == NULL) return;
  // The uniforms are copied so the fallback can be stamped into them without
  // the caller's canvas, which several draws share, ever changing.
  ScreenwideCanvas uniforms = *canvas;
  id<MTLTexture> picture = background_image_texture(device, &uniforms);
  [encoder setBytes:&uniforms length:sizeof(uniforms) atIndex:buffer_index];
  [encoder setTexture:picture atIndex:texture_index];
}
