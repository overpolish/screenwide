// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/// The audio-only preview's bars: the layer they draw into, the wgpu renderer
/// that draws them, and the display link that advances playback. The
/// per-frame work is in `+audio_ribbon_draw.m` and the inputs from Rust in
/// `+audio_ribbon_state.m`.

#import "recording_preview_surface_macos_private.h"

@implementation ScreenwideAudioRibbon
- (void)displayFrame:(CADisplayLink *)link {
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
  BOOL wanted = bars.trackCount > 0 && bars.renderer != NULL && !bars.layer.hidden;
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
    ScreenwidePreviewSurface *surface) {
  ScreenwideAudioRibbon *bars = [ScreenwideAudioRibbon new];
  // The renderer's surface sets the drawable size, pixel format and
  // opacity; the premultiplied pixels it presents are what Core Animation
  // composites over the container's backdrop.
  CAMetalLayer *layer = [CAMetalLayer layer];
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
  bars.renderer = screenwide_audio_ribbon_renderer_create((__bridge void *)layer);
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
  bars.layer.frame = surface.container.bounds;
  bars.layer.contentsScale = surface.host.window.backingScaleFactor ?: 1.0;
}

SCREENWIDE_PREVIEW_PRIVATE void screenwide_audio_ribbon_detach(
    ScreenwidePreviewSurface *surface) {
  ScreenwideAudioRibbon *bars = surface.audioRibbon;
  if (bars == nil) return;
  [bars.displayLink invalidate];
  bars.displayLink = nil;
  bars.clock = nil;
  screenwide_audio_ribbon_renderer_destroy(bars.renderer);
  bars.renderer = NULL;
  [bars.layer removeFromSuperlayer];
  surface.audioRibbon = nil;
}
