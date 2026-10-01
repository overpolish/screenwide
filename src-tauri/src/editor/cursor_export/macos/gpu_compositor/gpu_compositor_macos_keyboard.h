// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#import <Metal/Metal.h>

#import "gpu_compositor_macos.h"

@interface ScreenwideKeyboardArtwork : NSObject
@property(nonatomic, strong) id<MTLBuffer> pixels;
@property(nonatomic) ScreenwideKeyboardUniforms uniforms;
@end

ScreenwideKeyboardArtwork *screenwide_keyboard_artwork(
    id<MTLDevice> device,
    NSMutableDictionary<NSString *, ScreenwideKeyboardArtwork *> *cache,
    ScreenwideKeyboardOverlay overlay, uint32_t output_height);

void screenwide_bind_keyboard(
    id<MTLComputeCommandEncoder> encoder, id<MTLDevice> device,
    NSMutableDictionary<NSString *, ScreenwideKeyboardArtwork *> *cache,
    ScreenwideKeyboardOverlay overlay, uint32_t output_height);

const ScreenwideKeyboardOverlay *screenwide_keyboard_at(
    const ScreenwideKeyboardOverlay *keyboards, uint32_t count, CMTime pts);

id<MTLComputePipelineState> screenwide_keyboard_pipeline(
    id<MTLDevice> device, id<MTLLibrary> library, NSString *name,
    NSError **error);

void screenwide_encode_keyboard_overlay(
    id<MTLCommandBuffer> command, id<MTLDevice> device,
    id<MTLComputePipelineState> luma_pipeline,
    id<MTLComputePipelineState> chroma_pipeline,
    id<MTLTexture> destination_y, id<MTLTexture> destination_uv,
    NSMutableDictionary<NSString *, ScreenwideKeyboardArtwork *> *cache,
    const ScreenwideKeyboardOverlay *keyboard, uint32_t output_width,
    uint32_t output_height);
