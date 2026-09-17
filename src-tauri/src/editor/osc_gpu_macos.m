// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "osc_gpu_macos.h"
#import "osc_gpu_macos_shader.h"

_Static_assert(sizeof(ScreenwideRegionOscVertex) == 24,
               "Region OSC vertices must match the Metal struct stride");
_Static_assert(sizeof(ScreenwideOscControlPalette) == 32,
               "OSC palette must match the Rust FFI layout");

NSString *screenwide_region_osc_shader_source(void) {
  return ScreenwideRegionOscMetalSource;
}

id<MTLTexture> screenwide_region_osc_make_placeholder(id<MTLDevice> device) {
  MTLTextureDescriptor *descriptor = [MTLTextureDescriptor
      texture2DDescriptorWithPixelFormat:MTLPixelFormatRGBA8Unorm
                                   width:1
                                  height:1
                               mipmapped:NO];
  descriptor.usage = MTLTextureUsageShaderRead;
  id<MTLTexture> texture = [device newTextureWithDescriptor:descriptor];
  const uint8_t transparent[4] = {0, 0, 0, 0};
  [texture replaceRegion:MTLRegionMake2D(0, 0, 1, 1)
              mipmapLevel:0
                withBytes:transparent
              bytesPerRow:4];
  return texture;
}

static void encode(
    id<MTLRenderCommandEncoder> encoder,
    id<MTLRenderPipelineState> pipeline, id<MTLBuffer> vertices,
    NSUInteger vertex_count, ScreenwideRegionOscRenderState state,
    id<MTLTexture> label, id<MTLTexture> secondary_label,
    id<MTLTexture> snapshot) {
  [encoder setRenderPipelineState:pipeline];
  [encoder setVertexBuffer:vertices offset:0 atIndex:0];
  [encoder setFragmentBytes:&state.light_mode
                     length:sizeof(state.light_mode)
                    atIndex:0];
  [encoder setFragmentBytes:state.magnifier_box
                     length:sizeof(state.magnifier_box)
                    atIndex:1];
  [encoder setFragmentBytes:state.action_fills
                     length:sizeof(state.action_fills)
                    atIndex:2];
  float control_colors[8];
  memcpy(control_colors, state.control_fill, sizeof(state.control_fill));
  memcpy(control_colors + 4, state.control_outline,
         sizeof(state.control_outline));
  [encoder setFragmentBytes:control_colors
                     length:sizeof(control_colors)
                    atIndex:3];
  [encoder setFragmentBytes:state.ocr_colors
                     length:sizeof(state.ocr_colors)
                    atIndex:4];
  [encoder setFragmentBytes:state.overlay_shade
                     length:sizeof(state.overlay_shade)
                    atIndex:5];
  [encoder setFragmentBytes:state.ruler_colors
                     length:sizeof(state.ruler_colors)
                    atIndex:6];
  [encoder setFragmentBytes:state.ruler_sample
                     length:sizeof(state.ruler_sample)
                    atIndex:7];
  [encoder setFragmentBytes:state.ruler_animation
                     length:sizeof(state.ruler_animation)
                    atIndex:8];
  [encoder setFragmentTexture:label atIndex:0];
  [encoder setFragmentTexture:secondary_label atIndex:1];
  [encoder setFragmentTexture:screenwide_osc_icon_texture(pipeline.device)
                        atIndex:2];
  [encoder setFragmentTexture:snapshot atIndex:3];
  [encoder drawPrimitives:MTLPrimitiveTypeTriangle
              vertexStart:0
              vertexCount:vertex_count];
}

void screenwide_region_osc_encode(
    id<MTLRenderCommandEncoder> encoder,
    id<MTLRenderPipelineState> pipeline, id<MTLBuffer> vertices,
    NSUInteger vertex_count, ScreenwideRegionOscRenderState state,
    id<MTLTexture> label, id<MTLTexture> secondary_label) {
  // Non-snapshot surfaces never emit kind 33, so any valid texture keeps the
  // shared fragment interface fully bound without allocating another asset.
  encode(encoder, pipeline, vertices, vertex_count, state, label,
         secondary_label, label);
}

void screenwide_region_osc_encode_with_snapshot(
    id<MTLRenderCommandEncoder> encoder,
    id<MTLRenderPipelineState> pipeline, id<MTLBuffer> vertices,
    NSUInteger vertex_count, ScreenwideRegionOscRenderState state,
    id<MTLTexture> label, id<MTLTexture> secondary_label,
    id<MTLTexture> snapshot) {
  encode(encoder, pipeline, vertices, vertex_count, state, label,
         secondary_label, snapshot);
}
