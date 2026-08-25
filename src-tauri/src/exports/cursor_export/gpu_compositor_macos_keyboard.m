// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#include <math.h>

#import "gpu_compositor_macos_keyboard.h"

static float keyboard_effective_scale(ScreenwideKeyboardOverlay keyboard,
                                      uint32_t outputWidth,
                                      uint32_t outputHeight) {
  float requested = keyboard.requested_scale > 0.0f
      ? keyboard.requested_scale : keyboard.scale;
  if (!(keyboard.maximum_width > 0.0f) || outputWidth == 0 || outputHeight == 0)
    return requested;
  const float availableWidth = (float)outputWidth * (1.0f - 0.055f * 2.0f);
  const float widthAtUnitScale = (float)outputHeight * (60.0f / 1080.0f) *
      keyboard.maximum_width / 20.0f;
  const float fitted = availableWidth / MAX(widthAtUnitScale * 1.12f, 0.0001f);
  return MIN(requested, fitted);
}

void screenwide_bind_keyboard(
    id<MTLComputeCommandEncoder> encoder, id<MTLDevice> device,
    NSMutableDictionary<NSString *, ScreenwideKeyboardArtwork *> *cache,
    ScreenwideKeyboardOverlay overlay, uint32_t outputHeight) {
  ScreenwideKeyboardArtwork *artwork = screenwide_keyboard_artwork(
      device, cache, overlay, outputHeight);
  id<MTLBuffer> pixels = artwork.pixels ?:
      [device newBufferWithLength:4 options:MTLResourceStorageModeShared];
  ScreenwideKeyboardUniforms uniforms = artwork != nil
      ? artwork.uniforms : (ScreenwideKeyboardUniforms){0};
  [encoder setBuffer:pixels offset:0 atIndex:10];
  [encoder setBytes:&uniforms length:sizeof(uniforms) atIndex:11];
}

const ScreenwideKeyboardOverlay *screenwide_keyboard_at(
    const ScreenwideKeyboardOverlay *keyboards, uint32_t count, CMTime pts) {
  if (keyboards == NULL || count == 0) return NULL;
  double seconds = CMTimeGetSeconds(pts);
  if (!isfinite(seconds) || seconds < 0.0) seconds = 0.0;
  int64_t index = (int64_t)floor(seconds * 60.0 + 1e-6);
  if (index > (int64_t)count - 1) index = (int64_t)count - 1;
  return &keyboards[index];
}

id<MTLComputePipelineState> screenwide_keyboard_pipeline(
    id<MTLDevice> device, id<MTLLibrary> library, NSString *name,
    NSError **error) {
  return [device newComputePipelineStateWithFunction:
                     [library newFunctionWithName:name] error:error];
}

void screenwide_encode_keyboard_overlay(
    id<MTLCommandBuffer> command, id<MTLDevice> device,
    id<MTLComputePipelineState> luma_pipeline,
    id<MTLComputePipelineState> chroma_pipeline,
    id<MTLTexture> destination_y, id<MTLTexture> destination_uv,
    NSMutableDictionary<NSString *, ScreenwideKeyboardArtwork *> *cache,
    const ScreenwideKeyboardOverlay *keyboard, uint32_t output_width,
    uint32_t output_height) {
  if (keyboard == NULL || keyboard->key_count == 0) return;
  ScreenwideKeyboardArtwork *artwork =
      screenwide_keyboard_artwork(device, cache, *keyboard, output_height);
  if (artwork == nil || artwork.pixels == nil || artwork.uniforms.height == 0)
    return;
  ScreenwideKeyboardUniforms uniforms = artwork.uniforms;
  float height = (float)output_height * (60.0f / 1080.0f) *
      keyboard_effective_scale(*keyboard, output_width, output_height);
  float width = height * (float)uniforms.width / (float)uniforms.height;
  float center_x = (float)output_width * 0.5f;
  float center_y = (float)output_height * (1.0f - 0.055f) - height * 0.5f;
  float margin = height;
  int32_t origin[2] = {
      (int32_t)floorf(center_x - width * 0.5f - margin),
      (int32_t)floorf(center_y - height * 0.5f - margin),
  };
  origin[0] = MAX(origin[0], 0) & ~1;
  origin[1] = MAX(origin[1], 0) & ~1;
  uint32_t box_width = (uint32_t)MIN(
      ceilf(width + margin * 2.0f), (float)output_width - origin[0]);
  uint32_t box_height = (uint32_t)MIN(
      ceilf(height + margin * 2.0f), (float)output_height - origin[1]);
  if (box_width == 0 || box_height == 0) return;
  MTLSize group = MTLSizeMake(16, 16, 1);
  id<MTLComputeCommandEncoder> encoder = [command computeCommandEncoder];
  [encoder setComputePipelineState:luma_pipeline];
  [encoder setBuffer:artwork.pixels offset:0 atIndex:0];
  [encoder setBytes:&uniforms length:sizeof(uniforms) atIndex:1];
  [encoder setBytes:origin length:sizeof(origin) atIndex:2];
  [encoder setTexture:destination_y atIndex:0];
  [encoder dispatchThreads:MTLSizeMake(box_width, box_height, 1)
         threadsPerThreadgroup:group];
  [encoder endEncoding];

  uint32_t dimensions[2] = {output_width, output_height};
  encoder = [command computeCommandEncoder];
  [encoder setComputePipelineState:chroma_pipeline];
  [encoder setBuffer:artwork.pixels offset:0 atIndex:0];
  [encoder setBytes:&uniforms length:sizeof(uniforms) atIndex:1];
  [encoder setBytes:origin length:sizeof(origin) atIndex:2];
  [encoder setBytes:dimensions length:sizeof(dimensions) atIndex:3];
  [encoder setTexture:destination_uv atIndex:0];
  [encoder dispatchThreads:MTLSizeMake((box_width + 1) / 2,
                                       (box_height + 1) / 2, 1)
         threadsPerThreadgroup:group];
  [encoder endEncoding];
}
