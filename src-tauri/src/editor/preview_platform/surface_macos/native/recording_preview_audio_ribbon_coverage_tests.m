// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import <Foundation/Foundation.h>
#import <Metal/Metal.h>
#include <float.h>

#import "recording_preview_audio_ribbon_macos.h"
#import "recording_preview_audio_ribbon_shader.h"

static id<MTLTexture> screenwide_audio_coverage_texture(id<MTLDevice> device,
                                                        NSUInteger width,
                                                        NSUInteger height) {
  MTLTextureDescriptor *descriptor = [MTLTextureDescriptor
      texture2DDescriptorWithPixelFormat:MTLPixelFormatBGRA8Unorm
                                   width:width
                                  height:height
                               mipmapped:NO];
  descriptor.usage = MTLTextureUsageRenderTarget | MTLTextureUsageShaderRead;
  descriptor.storageMode = MTLStorageModeShared;
  return [device newTextureWithDescriptor:descriptor];
}

static float screenwide_audio_coverage_sample(id<MTLDevice> device,
                                              id<MTLCommandQueue> queue,
                                              id<MTLRenderPipelineState> pipeline,
                                              id<MTLTexture> bars,
                                              float centre_x,
                                              float half_height) {
  enum { width = 32, height = 16 };
  id<MTLTexture> target =
      screenwide_audio_coverage_texture(device, width, height);
  if (target == nil) return -1.0f;

  ScreenwideAudioBarsUniforms uniforms = {
      .color = {1.0f, 1.0f, 1.0f, 1.0f},
      .viewport = {(float)width, (float)height},
      .origin_x = centre_x - 2.0f,
      .bar_pitch = 100.0f,
      .bar_width = 4.0f,
      .maximum = half_height,
      .scale = 0.0f,
      .bar_count = 1,
  };

  MTLRenderPassDescriptor *pass = [MTLRenderPassDescriptor renderPassDescriptor];
  pass.colorAttachments[0].texture = target;
  pass.colorAttachments[0].loadAction = MTLLoadActionClear;
  pass.colorAttachments[0].storeAction = MTLStoreActionStore;
  pass.colorAttachments[0].clearColor = MTLClearColorMake(0, 0, 0, 0);

  id<MTLCommandBuffer> command = [queue commandBuffer];
  if (command == nil) return -1.0f;
  id<MTLRenderCommandEncoder> encoder = [command renderCommandEncoderWithDescriptor:pass];
  [encoder setRenderPipelineState:pipeline];
  [encoder setFragmentBytes:&uniforms
                     length:sizeof(uniforms)
                    atIndex:0];
  [encoder setFragmentTexture:bars atIndex:0];
  [encoder drawPrimitives:MTLPrimitiveTypeTriangle vertexStart:0 vertexCount:3];
  [encoder endEncoding];
  [command commit];
  [command waitUntilCompleted];
  if (command.status != MTLCommandBufferStatusCompleted) return -1.0f;

  uint8_t pixels[width * height * 4];
  [target getBytes:pixels
        bytesPerRow:width * 4
         fromRegion:MTLRegionMake2D(0, 0, width, height)
        mipmapLevel:0];
  NSUInteger alpha_sum = 0;
  for (NSUInteger offset = 3; offset < sizeof(pixels); offset += 4)
    alpha_sum += pixels[offset];
  return (float)alpha_sum;
}

static BOOL screenwide_audio_coverage_case(id<MTLDevice> device,
                                           id<MTLCommandQueue> queue,
                                           id<MTLRenderPipelineState> pipeline,
                                           id<MTLTexture> bars,
                                           float half_height) {
  float minimum = FLT_MAX;
  float maximum = -FLT_MAX;
  for (NSUInteger phase = 0; phase < 16; phase++) {
    float sum = screenwide_audio_coverage_sample(
        device, queue, pipeline, bars, 16.0f + (float)phase / 16.0f,
        half_height);
    if (sum < 0.0f) return NO;
    minimum = MIN(minimum, sum);
    maximum = MAX(maximum, sum);
  }
  float mean = (minimum + maximum) * 0.5f;
  return mean > 0.0f && (maximum - minimum) / mean < 0.005f;
}

int screenwide_audio_ribbon_coverage_is_stable(void) {
  @autoreleasepool {
    id<MTLDevice> device = MTLCreateSystemDefaultDevice();
    if (device == nil) return 0;
    id<MTLCommandQueue> queue = [device newCommandQueue];
    NSError *error = nil;
    id<MTLLibrary> library = [device newLibraryWithSource:screenwide_audio_bars_shader
                                                  options:nil
                                                    error:&error];
    if (queue == nil || library == nil) return 0;
    MTLRenderPipelineDescriptor *descriptor = [MTLRenderPipelineDescriptor new];
    descriptor.vertexFunction = [library newFunctionWithName:@"audio_bars_vertex"];
    descriptor.fragmentFunction = [library newFunctionWithName:@"audio_bars"];
    descriptor.colorAttachments[0].pixelFormat = MTLPixelFormatBGRA8Unorm;
    id<MTLRenderPipelineState> pipeline =
        [device newRenderPipelineStateWithDescriptor:descriptor error:&error];
    if (pipeline == nil) return 0;

    MTLTextureDescriptor *bars_descriptor = [MTLTextureDescriptor
        texture2DDescriptorWithPixelFormat:MTLPixelFormatR32Float
                                     width:1
                                    height:1
                                 mipmapped:NO];
    bars_descriptor.usage = MTLTextureUsageShaderRead;
    bars_descriptor.storageMode = MTLStorageModeShared;
    id<MTLTexture> bars = [device newTextureWithDescriptor:bars_descriptor];
    if (bars == nil) return 0;
    float amplitude = 1.0f;
    [bars replaceRegion:MTLRegionMake2D(0, 0, 1, 1)
            mipmapLevel:0
              withBytes:&amplitude
            bytesPerRow:sizeof(amplitude)];

    return screenwide_audio_coverage_case(device, queue, pipeline, bars, 2.0f) &&
           screenwide_audio_coverage_case(device, queue, pipeline, bars, 6.0f);
  }
}
