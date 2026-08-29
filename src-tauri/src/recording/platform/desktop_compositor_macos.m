// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import <CoreVideo/CoreVideo.h>
#import <Foundation/Foundation.h>
#import <Metal/Metal.h>

static NSString *const shader_source = @R"METAL(
#include <metal_stdlib>
using namespace metal;

struct Piece {
  uint source_x;
  uint source_y;
  uint source_width;
  uint source_height;
  uint destination_x;
  uint destination_y;
  uint destination_width;
  uint destination_height;
};

constexpr sampler linear_sampler(coord::pixel, address::clamp_to_edge,
                                 filter::linear);

kernel void clear_canvas(texture2d<float, access::write> output [[texture(0)]],
                         uint2 position [[thread_position_in_grid]]) {
  if (position.x < output.get_width() && position.y < output.get_height())
    output.write(float4(0.0, 0.0, 0.0, 1.0), position);
}

kernel void place_piece(texture2d<float, access::sample> source [[texture(0)]],
                        texture2d<float, access::write> output [[texture(1)]],
                        constant Piece &piece [[buffer(0)]],
                        uint2 local [[thread_position_in_grid]]) {
  if (local.x >= piece.destination_width || local.y >= piece.destination_height)
    return;
  float2 ratio = (float2(local) + 0.5) /
                 float2(piece.destination_width, piece.destination_height);
  float2 coordinate = float2(piece.source_x, piece.source_y) +
                      ratio * float2(piece.source_width, piece.source_height);
  output.write(source.sample(linear_sampler, coordinate),
               uint2(piece.destination_x, piece.destination_y) + local);
}
)METAL";

typedef struct {
  uint32_t source_x, source_y, source_width, source_height;
  uint32_t destination_x, destination_y, destination_width, destination_height;
} ScreenwideDesktopPiece;

typedef struct {
  CVPixelBufferRef pixels;
  ScreenwideDesktopPiece piece;
} ScreenwideDesktopFrame;

@interface ScreenwideDesktopCompositor : NSObject {
@public
  id<MTLDevice> device;
  id<MTLCommandQueue> queue;
  id<MTLComputePipelineState> clear_pipeline;
  id<MTLComputePipelineState> place_pipeline;
  CVMetalTextureCacheRef texture_cache;
  size_t width;
  size_t height;
}
@end

@implementation ScreenwideDesktopCompositor
- (void)dealloc {
  if (texture_cache != NULL) {
    CVMetalTextureCacheFlush(texture_cache, 0);
    CFRelease(texture_cache);
  }
}
@end

static void report(char *text, size_t capacity, NSString *message) {
  if (text != NULL && capacity > 0)
    snprintf(text, capacity, "%s", (message ?: @"Desktop composition failed").UTF8String);
}

static id<MTLTexture> texture(CVMetalTextureCacheRef cache,
                              CVPixelBufferRef pixels,
                              CVMetalTextureRef *reference) {
  CVReturn result = CVMetalTextureCacheCreateTextureFromImage(
      kCFAllocatorDefault, cache, pixels, NULL, MTLPixelFormatBGRA8Unorm,
      CVPixelBufferGetWidth(pixels), CVPixelBufferGetHeight(pixels), 0,
      reference);
  return result == kCVReturnSuccess && *reference != NULL
             ? CVMetalTextureGetTexture(*reference)
             : nil;
}

void *screenwide_desktop_compositor_create(uint32_t width, uint32_t height,
                                           char *error_text,
                                           size_t error_capacity) {
  @autoreleasepool {
    if (width == 0 || height == 0) {
      report(error_text, error_capacity, @"The desktop canvas is empty");
      return NULL;
    }
    ScreenwideDesktopCompositor *compositor = [ScreenwideDesktopCompositor new];
    compositor->width = width;
    compositor->height = height;
    compositor->device = MTLCreateSystemDefaultDevice();
    NSError *error = nil;
    id<MTLLibrary> library = [compositor->device newLibraryWithSource:shader_source
                                                              options:nil
                                                                error:&error];
    compositor->clear_pipeline = [compositor->device
        newComputePipelineStateWithFunction:[library newFunctionWithName:@"clear_canvas"]
                                      error:&error];
    compositor->place_pipeline = [compositor->device
        newComputePipelineStateWithFunction:[library newFunctionWithName:@"place_piece"]
                                      error:&error];
    compositor->queue = [compositor->device newCommandQueue];
    CVReturn cache = CVMetalTextureCacheCreate(kCFAllocatorDefault, NULL,
                                                compositor->device, NULL,
                                                &compositor->texture_cache);
    if (compositor->device == nil || library == nil ||
        compositor->clear_pipeline == nil || compositor->place_pipeline == nil ||
        compositor->queue == nil || cache != kCVReturnSuccess ||
        compositor->texture_cache == NULL) {
      report(error_text, error_capacity,
             error.localizedDescription ?: @"The desktop Metal pipeline is unavailable");
      return NULL;
    }
    return (__bridge_retained void *)compositor;
  }
}

CVPixelBufferRef screenwide_desktop_compositor_compose(
    void *handle, const ScreenwideDesktopFrame *frames, size_t frame_count,
    char *error_text, size_t error_capacity) {
  if (handle == NULL || frames == NULL || frame_count == 0)
    return NULL;
  @autoreleasepool {
    ScreenwideDesktopCompositor *compositor =
        (__bridge ScreenwideDesktopCompositor *)handle;
    NSDictionary *attributes = @{
      (id)kCVPixelBufferIOSurfacePropertiesKey : @{},
      (id)kCVPixelBufferMetalCompatibilityKey : @YES,
    };
    CVPixelBufferRef output = NULL;
    CVReturn created = CVPixelBufferCreate(
        kCFAllocatorDefault, compositor->width, compositor->height,
        kCVPixelFormatType_32BGRA, (__bridge CFDictionaryRef)attributes, &output);
    if (created != kCVReturnSuccess || output == NULL) {
      report(error_text, error_capacity, @"CoreVideo could not allocate the desktop canvas");
      return NULL;
    }
    CVMetalTextureRef output_ref = NULL;
    id<MTLTexture> output_texture = texture(compositor->texture_cache, output, &output_ref);
    id<MTLCommandBuffer> command = [compositor->queue commandBuffer];
    if (output_texture == nil || command == nil) {
      report(error_text, error_capacity, @"Metal could not wrap the desktop canvas");
      if (output_ref != NULL) CFRelease(output_ref);
      CFRelease(output);
      return NULL;
    }
    MTLSize group = MTLSizeMake(16, 16, 1);
    id<MTLComputeCommandEncoder> compute = [command computeCommandEncoder];
    [compute setComputePipelineState:compositor->clear_pipeline];
    [compute setTexture:output_texture atIndex:0];
    [compute dispatchThreads:MTLSizeMake(compositor->width, compositor->height, 1)
           threadsPerThreadgroup:group];
    [compute endEncoding];

    NSMutableArray *source_refs = [NSMutableArray arrayWithCapacity:frame_count];
    for (size_t index = 0; index < frame_count; index++) {
      CVPixelBufferRef pixels = frames[index].pixels;
      if (pixels == NULL || CVPixelBufferGetPixelFormatType(pixels) !=
                                kCVPixelFormatType_32BGRA)
        continue;
      const ScreenwideDesktopPiece piece = frames[index].piece;
      if (piece.source_width == 0 || piece.source_height == 0 ||
          piece.destination_width == 0 || piece.destination_height == 0 ||
          (uint64_t)piece.source_x + piece.source_width > CVPixelBufferGetWidth(pixels) ||
          (uint64_t)piece.source_y + piece.source_height > CVPixelBufferGetHeight(pixels) ||
          (uint64_t)piece.destination_x + piece.destination_width > compositor->width ||
          (uint64_t)piece.destination_y + piece.destination_height > compositor->height)
        continue;
      CVMetalTextureRef source_ref = NULL;
      id<MTLTexture> source = texture(compositor->texture_cache, pixels, &source_ref);
      if (source == nil) {
        if (source_ref != NULL) CFRelease(source_ref);
        continue;
      }
      [source_refs addObject:(__bridge id)source_ref];
      compute = [command computeCommandEncoder];
      [compute setComputePipelineState:compositor->place_pipeline];
      [compute setTexture:source atIndex:0];
      [compute setTexture:output_texture atIndex:1];
      [compute setBytes:&frames[index].piece
                 length:sizeof(frames[index].piece)
                atIndex:0];
      [compute dispatchThreads:MTLSizeMake(frames[index].piece.destination_width,
                                           frames[index].piece.destination_height, 1)
             threadsPerThreadgroup:group];
      [compute endEncoding];
    }
    [command commit];
    [command waitUntilCompleted];
    for (id reference in source_refs) CFRelease((__bridge CFTypeRef)reference);
    if (output_ref != NULL) CFRelease(output_ref);
    CVMetalTextureCacheFlush(compositor->texture_cache, 0);
    if (command.status != MTLCommandBufferStatusCompleted) {
      report(error_text, error_capacity,
             command.error.localizedDescription ?: @"Metal did not complete desktop composition");
      CFRelease(output);
      return NULL;
    }
    return output;
  }
}

void screenwide_desktop_compositor_destroy(void *handle) {
  if (handle != NULL) {
    @autoreleasepool {
      ScreenwideDesktopCompositor *compositor =
          (__bridge_transfer ScreenwideDesktopCompositor *)handle;
      (void)compositor;
    }
  }
}
