// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "region_osc_renderer_macos.h"

ScreenwideRegionOscRenderState screenwide_region_osc_render_state(
    uint32_t light_mode) {
  ScreenwideOscControlPalette palette =
      screenwide_osc_control_palette(light_mode);
  ScreenwideOscOverlayPalette overlay = screenwide_osc_overlay_palette();
  ScreenwideOscOcrPalette ocr = screenwide_osc_ocr_palette(light_mode);
  ScreenwideRegionOscRenderState state = {.light_mode = light_mode};
  memcpy(state.overlay_shade, overlay.shade, sizeof(state.overlay_shade));
  memcpy(state.control_fill, palette.fill, sizeof(state.control_fill));
  memcpy(state.control_outline, palette.outline,
         sizeof(state.control_outline));
  memcpy(state.ocr_colors, &ocr, sizeof(state.ocr_colors));
  return state;
}

id<MTLRenderPipelineState> screenwide_region_osc_make_pipeline(
    id<MTLDevice> device, id<MTLLibrary> library, NSError **error) {
  MTLRenderPipelineDescriptor *descriptor =
      [MTLRenderPipelineDescriptor new];
  descriptor.vertexFunction =
      [library newFunctionWithName:@"region_osc_vertex_main"];
  descriptor.fragmentFunction =
      [library newFunctionWithName:@"region_osc_fragment"];
  descriptor.colorAttachments[0].pixelFormat = MTLPixelFormatBGRA8Unorm;
  descriptor.colorAttachments[0].blendingEnabled = YES;
  descriptor.colorAttachments[0].sourceRGBBlendFactor =
      MTLBlendFactorSourceAlpha;
  descriptor.colorAttachments[0].destinationRGBBlendFactor =
      MTLBlendFactorOneMinusSourceAlpha;
  descriptor.colorAttachments[0].sourceAlphaBlendFactor =
      MTLBlendFactorSourceAlpha;
  descriptor.colorAttachments[0].destinationAlphaBlendFactor =
      MTLBlendFactorOneMinusSourceAlpha;
  return [device newRenderPipelineStateWithDescriptor:descriptor error:error];
}
