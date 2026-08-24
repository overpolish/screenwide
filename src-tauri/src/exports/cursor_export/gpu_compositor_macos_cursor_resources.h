// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#import <Metal/Metal.h>

#import "gpu_compositor_macos.h"

@interface ScreenwideCursorResources : NSObject
@property(nonatomic, strong) id<MTLTexture> texture;
@property(nonatomic, strong) NSData *uniforms;
@property(nonatomic) uint32_t count;
@end

ScreenwideCursorResources *screenwide_cursor_resources(
    id<MTLDevice> device, const ScreenwideCursorArtwork *artworks,
    uint32_t artwork_count);
ScreenwideOverlayUniforms screenwide_workspace_cursor_uniforms(
    ScreenwideCursorResources *resources,
    const ScreenwideWorkspaceLayer *layer);
ScreenwideOverlayUniforms screenwide_canvas_cursor_uniforms(
    ScreenwideCursorResources *resources, const ScreenwideGpuCursor *cursor,
    const ScreenwideCanvas *canvas, uint32_t output_width,
    uint32_t output_height);
