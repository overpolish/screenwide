// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/// The audio-only preview's bars: the layer they draw into, the pipeline that
/// draws them, and the display link that advances playback. The per-frame work is
/// in `+audio_ribbon_draw.m` and the inputs from Rust in `+audio_ribbon_state.m`.

#import "recording_preview_audio_ribbon_shader.h"
#import "recording_preview_surface_macos_private.h"

@implementation ScreenwideAudioRibbon
- (void)displayFrame:(CADisplayLink *)link {
  if (self.drawInFlight) return;
  if (self.clock != nil) {
    double ahead = MAX(link.targetTimestamp - CACurrentMediaTime(), 0.0);
    self.playheadRatio = self.clock.read(self.clock.context, ahead);
  }
  screenwide_audio_ribbon_draw(self.surface);
}
@end

/// Match the host view's display cadence, including when it moves between
/// screens. A run-loop timer can miss alternating refreshes and visibly judder.
static void update_audio_bars_display_link(ScreenwidePreviewSurface *surface) {
  ScreenwideAudioRibbon *bars = surface.audioRibbon;
  if (bars == nil) return;
  BOOL wanted = bars.trackCount > 0 && bars.pipeline != nil && !bars.layer.hidden;
  if (wanted == (bars.displayLink != nil)) return;
  if (!wanted) {
    [bars.displayLink invalidate];
    bars.displayLink = nil;
    return;
  }
  bars.surface = surface;
  bars.displayLink = [surface.host displayLinkWithTarget:bars
                                               selector:@selector(displayFrame:)];
  [bars.displayLink addToRunLoop:NSRunLoop.mainRunLoop forMode:NSRunLoopCommonModes];
}

/// Main thread only. Hidden the moment there is nothing to draw, so a
/// recording with video never pays for a transparent full-viewport pass.
///
/// While they are showing, the bars own the container's visibility. The
/// container is otherwise shown by a viewport layout and hidden by
/// `screenwide_preview_surface_hide`, which exists to take away video frames
/// between playback and a still - and the bars are not a video frame. The
/// layout that would show it again is also deduplicated by the webview when
/// nothing about it changed, so waiting for one can mean waiting forever.
SCREENWIDE_PREVIEW_PRIVATE void screenwide_audio_ribbon_update_visibility(
    ScreenwidePreviewSurface *surface) {
  ScreenwideAudioRibbon *bars = surface.audioRibbon;
  if (bars == nil) return;
  BOOL showing = bars.trackCount > 0 && bars.pointCount > 0;
  bars.layer.hidden = !showing;
  NSSize viewport = surface.container.bounds.size;
  if (showing && viewport.width > 0.0 && viewport.height > 0.0)
    surface.container.hidden = NO;
  update_audio_bars_display_link(surface);
}

SCREENWIDE_PREVIEW_PRIVATE void screenwide_audio_ribbon_attach(
    ScreenwidePreviewSurface *surface, id<MTLLibrary> library) {
  if (surface.device == nil || library == nil) return;
  ScreenwideAudioRibbon *bars = [ScreenwideAudioRibbon new];
  MTLRenderPipelineDescriptor *descriptor = [MTLRenderPipelineDescriptor new];
  descriptor.vertexFunction = [library newFunctionWithName:@"audio_bars_vertex"];
  descriptor.fragmentFunction = [library newFunctionWithName:@"audio_bars"];
  MTLRenderPipelineColorAttachmentDescriptor *attachment =
      descriptor.colorAttachments[0];
  attachment.pixelFormat = MTLPixelFormatBGRA8Unorm;
  // The shader returns premultiplied alpha, which is what Core Animation
  // composites this layer over the container's backdrop with.
  attachment.blendingEnabled = YES;
  attachment.sourceRGBBlendFactor = MTLBlendFactorOne;
  attachment.sourceAlphaBlendFactor = MTLBlendFactorOne;
  attachment.destinationRGBBlendFactor = MTLBlendFactorOneMinusSourceAlpha;
  attachment.destinationAlphaBlendFactor = MTLBlendFactorOneMinusSourceAlpha;
  NSError *error = nil;
  bars.pipeline = [surface.device newRenderPipelineStateWithDescriptor:descriptor
                                                                 error:&error];
  CAMetalLayer *layer = [CAMetalLayer layer];
  layer.device = surface.device;
  layer.pixelFormat = MTLPixelFormatBGRA8Unorm;
  layer.framebufferOnly = YES;
  layer.opaque = NO;
  layer.presentsWithTransaction = NO;
  NSNull *noAction = [NSNull null];
  layer.actions = @{
    @"bounds": noAction,
    @"position": noAction,
    @"hidden": noAction,
    @"opacity": noAction,
    @"contents": noAction,
  };
  layer.hidden = YES;
  bars.layer = layer;
  surface.audioRibbon = bars;
  // Index zero keeps it under the pane views' own layers; only an audio-only
  // layout leaves those hidden, which is exactly when the bars show.
  [surface.container.layer insertSublayer:layer atIndex:0];
  screenwide_audio_ribbon_layout(surface);
}

SCREENWIDE_PREVIEW_PRIVATE void screenwide_audio_ribbon_layout(
    ScreenwidePreviewSurface *surface) {
  ScreenwideAudioRibbon *bars = surface.audioRibbon;
  if (bars == nil) return;
  NSRect bounds = surface.container.bounds;
  CGFloat scale = surface.host.window.backingScaleFactor ?: 1.0;
  bars.layer.frame = bounds;
  bars.layer.contentsScale = scale;
  bars.layer.drawableSize = CGSizeMake(MAX(round(bounds.size.width * scale), 2.0),
                                        MAX(round(bounds.size.height * scale), 2.0));
}

SCREENWIDE_PREVIEW_PRIVATE void screenwide_audio_ribbon_detach(
    ScreenwidePreviewSurface *surface) {
  ScreenwideAudioRibbon *bars = surface.audioRibbon;
  if (bars == nil) return;
  [bars.displayLink invalidate];
  bars.displayLink = nil;
  bars.clock = nil;
  [bars.layer removeFromSuperlayer];
  surface.audioRibbon = nil;
}

/// Compiles the bars' Metal source on its own, so a shader mistake fails a
/// unit test rather than the first audio-only preview. The surface's library
/// needs an editor window; this needs only a device.
int screenwide_audio_ribbon_shader_compiles(void) {
  @autoreleasepool {
    id<MTLDevice> device = MTLCreateSystemDefaultDevice();
    if (device == nil) return 0;
    NSError *error = nil;
    id<MTLLibrary> library = [device newLibraryWithSource:screenwide_audio_bars_shader
                                                  options:nil
                                                    error:&error];
    if (library == nil) return 0;
    return [library newFunctionWithName:@"audio_bars_vertex"] != nil &&
           [library newFunctionWithName:@"audio_bars"] != nil;
  }
}
