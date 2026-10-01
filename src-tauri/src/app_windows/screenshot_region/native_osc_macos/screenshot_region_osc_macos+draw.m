// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "screenshot_region_osc_macos_private.h"
#include <stdlib.h>

static uint32_t light_mode(ScreenwideRegionOSC *s) {
  NSString *appearance = [s.host.effectiveAppearance
      bestMatchFromAppearancesWithNames:@[ NSAppearanceNameAqua,
                                           NSAppearanceNameDarkAqua ]];
  return [appearance isEqualToString:NSAppearanceNameAqua] ? 1 : 0;
}

void screenwide_region_osc_draw(ScreenwideRegionOSC *s) {
  s.drawRevision += 1;
  uint64_t revision = s.drawRevision;
  if (!s.visible) {
    // Hiding a CAMetalLayer keeps its full-screen drawable pool alive.
    // Keep the OSC session, but recreate its presentation layer on demand.
    s.drawPending = NO;
    s.drawInFlight = NO;
    s.layer.hidden = YES;
    [s.layer removeFromSuperlayer];
    s.layer = nil;
    return;
  }
  if (s.pipeline == nil)
    return;
  if (s.layer == nil) {
    s.layer = [CAMetalLayer layer];
    s.layer.device = s.device;
    s.layer.pixelFormat = MTLPixelFormatBGRA8Unorm;
    s.layer.framebufferOnly = NO;
    s.layer.opaque = NO;
    s.layer.hidden = YES;
    s.layer.autoresizingMask = kCALayerWidthSizable | kCALayerHeightSizable;
    [s.host.layer insertSublayer:s.layer above:s.snapshotLayer];
  }
  NSSize size = s.host.bounds.size;
  if (size.width <= 0 || size.height <= 0) {
    s.drawPending = NO;
    s.layer.hidden = YES;
    return;
  }
  // Match the export OSC: keep at most one CAMetalDrawable in flight so
  // pointer samples never block AppKit waiting for the display server.
  if (s.drawInFlight) {
    s.drawPending = YES;
    return;
  }
  s.drawInFlight = YES;
  s.drawPending = NO;
  CGFloat scale = s.host.window.backingScaleFactor ?: 1.0;
  s.layer.frame = s.host.bounds;
  s.layer.contentsScale = scale;
  s.layer.drawableSize =
      CGSizeMake(MAX(round(size.width * scale), 2.0), MAX(round(size.height * scale), 2.0));

  NSUInteger capacity = 262 + screenwide_region_osc_ocr_vertex_capacity(s) +
                        screenwide_region_osc_ruler_vertex_capacity(s);
  ScreenwideRegionOscVertex *vertices =
      calloc(capacity, sizeof(ScreenwideRegionOscVertex));
  if (!vertices) {
    s.drawInFlight = NO;
    return;
  }
  NSUInteger count = 0;
  NSRect canvas = NSMakeRect(0, 0, size.width, size.height);
  if (s.snapshotComposited && s.snapshotPresented && s.snapshotTexture) {
    CGFloat zoom = MAX(s.rulerViewportZoom, 1.0);
    NSRect source = NSMakeRect(
        s.rulerViewportOrigin.x / size.width,
        s.rulerViewportOrigin.y / size.height,
        1.0 / zoom, 1.0 / zoom);
    screenwide_region_osc_add_texture_quad(
        vertices, &count, size, canvas, source, 33);
  }
  if (NSIsEmptyRect(s.region))
    screenwide_region_osc_add_quad(vertices, &count, size, canvas, 6);
  else
    screenwide_region_osc_add_crop_with_handles(
        vertices, &count, size, s.region, canvas, scale, 0.0, s.showFrame,
        s.showHandles);
  screenwide_region_osc_ocr_add_vertices(s, vertices, &count, size, scale);
  screenwide_region_osc_ruler_add_vertices(s, vertices, &count, size, scale);

  id<CAMetalDrawable> drawable = [s.layer nextDrawable];
  if (drawable == nil) {
    free(vertices);
    s.drawInFlight = NO;
    return;
  }
  id<MTLCommandBuffer> command = [s.queue commandBuffer];
  MTLRenderPassDescriptor *clear =
      [MTLRenderPassDescriptor renderPassDescriptor];
  clear.colorAttachments[0].texture = drawable.texture;
  clear.colorAttachments[0].loadAction = MTLLoadActionClear;
  clear.colorAttachments[0].storeAction = MTLStoreActionStore;
  clear.colorAttachments[0].clearColor = MTLClearColorMake(0, 0, 0, 0);
  id<MTLRenderCommandEncoder> clearEncoder =
      [command renderCommandEncoderWithDescriptor:clear];
  [clearEncoder endEncoding];

  BOOL showMagnifier = s.showFrame && s.magnifier.active != 0 &&
                       s.magnifierSource != nil &&
                       s.magnifierPipeline != nil;
  if (showMagnifier) {
    uint32_t dimensions[2] = {s.magnifierSourceWidth,
                              s.magnifierSourceHeight};
    id<MTLComputeCommandEncoder> magnifierEncoder =
        [command computeCommandEncoder];
    screenwide_region_magnifier_encode(
        magnifierEncoder, s.magnifierPipeline, s.magnifierSource,
        drawable.texture, dimensions, s.magnifier);
    [magnifierEncoder endEncoding];
  }

  id<MTLBuffer> buffer = screenwide_osc_vertex_buffer(
      s.device, vertices, count, size, scale);
  free(vertices);
  MTLRenderPassDescriptor *pass =
      [MTLRenderPassDescriptor renderPassDescriptor];
  pass.colorAttachments[0].texture = drawable.texture;
  pass.colorAttachments[0].loadAction = MTLLoadActionLoad;
  pass.colorAttachments[0].storeAction = MTLStoreActionStore;
  id<MTLRenderCommandEncoder> encoder =
      [command renderCommandEncoderWithDescriptor:pass];
  id<MTLRenderPipelineState> renderPipeline =
      s.snapshotComposited ? s.snapshotPipeline : s.pipeline;
  ScreenwideRegionOscRenderState state =
      screenwide_region_osc_render_state(light_mode(s));
  screenwide_region_osc_ruler_apply_render_state(s, &state);
  state.ruler_sample[0] = ((s.rulerColor >> 24) & 0xFF) / 255.0;
  state.ruler_sample[1] = ((s.rulerColor >> 16) & 0xFF) / 255.0;
  state.ruler_sample[2] = ((s.rulerColor >> 8) & 0xFF) / 255.0;
  state.ruler_sample[3] = (s.rulerColor & 0xFF) / 255.0;
  if (showMagnifier) {
    state.magnifier_box[0] = s.magnifier.box_x;
    state.magnifier_box[1] = s.magnifier.box_y;
    state.magnifier_box[2] = s.magnifier.box_width;
    state.magnifier_box[3] = s.magnifier.box_height;
  }
  screenwide_region_osc_encode_with_snapshot(
      encoder, renderPipeline, buffer, count, state,
      s.rulerLabel.texture ?: s.placeholder, s.placeholder,
      s.snapshotTexture ?: s.placeholder);
  [encoder endEncoding];
  // Presentation can remain pending when the window closes. The drawable
  // must not keep its owning surface and all of its GPU resources alive.
  __weak ScreenwideRegionOSC *weakSurface = s;
  __weak CAMetalLayer *weakLayer = s.layer;
  [drawable addPresentedHandler:^(__unused id<MTLDrawable> presented) {
    dispatch_async(dispatch_get_main_queue(), ^{
      ScreenwideRegionOSC *s = weakSurface;
      // A dismissed layer may finish presenting after a new session starts.
      if (!s || !weakLayer || s.layer != weakLayer)
        return;
      s.drawInFlight = NO;
      BOOL redrawPending = s.drawPending;
      s.drawPending = NO;
      if (s.visible)
        s.layer.hidden = NO;
      if (redrawPending) {
        screenwide_region_osc_draw(s);
      } else if (s.visible && s.drawRevision == revision) {
        s.layer.hidden = NO;
      }
    });
  }];
  [command presentDrawable:drawable];
  [command commit];
}
