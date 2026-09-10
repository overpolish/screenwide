// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Offscreen Metal regression using the production OSC pipeline configuration.
// clang -fobjc-arc -framework AppKit -framework Metal scripts/test-osc-alpha.m
// -o /tmp/osc-alpha && /tmp/osc-alpha
#import "../src-tauri/src/editor/osc_gpu_pipeline_macos.m"
#include <assert.h>

// Palette resolution is not exercised by this pipeline-only test.
ScreenwideOscControlPalette screenwide_osc_control_palette(uint32_t light) {
  return (ScreenwideOscControlPalette){0};
}
ScreenwideOscOverlayPalette screenwide_osc_overlay_palette(void) {
  return (ScreenwideOscOverlayPalette){0};
}
ScreenwideOscOcrPalette screenwide_osc_ocr_palette(uint32_t light) {
  return (ScreenwideOscOcrPalette){0};
}
ScreenwideOscRulerPalette screenwide_osc_ruler_palette(uint32_t light) {
  return (ScreenwideOscRulerPalette){0};
}

int main(void) {
  @autoreleasepool {
    id<MTLDevice> device = MTLCreateSystemDefaultDevice();
    assert(device);
    NSError *error = nil;
    NSString *source = @"#include <metal_stdlib>\nusing namespace metal;\n"
      "vertex float4 region_osc_vertex_main(uint i [[vertex_id]]) {"
      "float2 p[3] = {float2(-1,-1),float2(3,-1),float2(-1,3)};"
      "return float4(p[i],0,1); }"
      "fragment float4 region_osc_fragment(constant float4 &c [[buffer(0)]]) { return c; }";
    id<MTLLibrary> library = [device newLibraryWithSource:source options:nil error:&error];
    assert(library);
    id<MTLCommandQueue> queue = [device newCommandQueue];
    for (int snapshot = 0; snapshot < 2; snapshot++) {
      id<MTLRenderPipelineState> pipeline = snapshot
        ? screenwide_region_osc_make_snapshot_pipeline(device, library, &error)
        : screenwide_region_osc_make_pipeline(device, library, &error);
      assert(pipeline);
      for (int white = 0; white < 2; white++) {
        for (int layers = 1; layers <= 2; layers++) {
          MTLTextureDescriptor *desc = [MTLTextureDescriptor
            texture2DDescriptorWithPixelFormat:MTLPixelFormatBGRA8Unorm
            width:1 height:1 mipmapped:NO];
          desc.storageMode = MTLStorageModeShared;
          desc.usage = MTLTextureUsageRenderTarget;
          id<MTLTexture> texture = [device newTextureWithDescriptor:desc];
          MTLRenderPassDescriptor *pass = [MTLRenderPassDescriptor renderPassDescriptor];
          pass.colorAttachments[0].texture = texture;
          pass.colorAttachments[0].loadAction = MTLLoadActionClear;
          pass.colorAttachments[0].storeAction = MTLStoreActionStore;
          pass.colorAttachments[0].clearColor = MTLClearColorMake(0,0,0,0);
          id<MTLCommandBuffer> command = [queue commandBuffer];
          id<MTLRenderCommandEncoder> encoder = [command renderCommandEncoderWithDescriptor:pass];
          [encoder setRenderPipelineState:pipeline];
          float color[4] = {white, white, white, 0.10f};
          [encoder setFragmentBytes:color length:sizeof(color) atIndex:0];
          for (int i = 0; i < layers; i++)
            [encoder drawPrimitives:MTLPrimitiveTypeTriangle vertexStart:0 vertexCount:3];
          [encoder endEncoding];
          [command commit];
          [command waitUntilCompleted];
          assert(command.status == MTLCommandBufferStatusCompleted);
          uint8_t pixel[4];
          [texture getBytes:pixel bytesPerRow:4 fromRegion:MTLRegionMake2D(0,0,1,1) mipmapLevel:0];
          int expected = (int)round((1.0 - pow(0.9, layers)) * 255.0);
          assert(abs(pixel[3] - expected) <= 1);
          for (int channel = 0; channel < 3; channel++)
            assert(abs(pixel[channel] - (white ? pixel[3] : 0)) <= 1);
        }
      }
    }
    puts("OSC alpha: 8 offscreen Metal blending cases passed.");
  }
}
