// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#ifndef SCREENWIDE_RECORDING_PREVIEW_AUDIO_RIBBON_MACOS_H
#define SCREENWIDE_RECORDING_PREVIEW_AUDIO_RIBBON_MACOS_H

#import <AppKit/AppKit.h>
#import <Metal/Metal.h>
#import <QuartzCore/CAMetalLayer.h>
#import <QuartzCore/CADisplayLink.h>
#include <stdint.h>

/// Four tracks and 2048 envelope points are what the bars read; Rust clamps to
/// the same numbers before it uploads, so the payload is bounded whatever the
/// recording holds. The bar count follows the viewport and stops here.
#define SCREENWIDE_AUDIO_RIBBON_MAX_TRACKS 4u
#define SCREENWIDE_AUDIO_RIBBON_MAX_POINTS 2048u
#define SCREENWIDE_AUDIO_RIBBON_MAX_BARS 512u

/// Mirrors `ScreenwideAudioBarsUniforms` in the bars' Metal source. Every
/// measurement is in drawable pixels: `scale` is there so the shader can keep
/// its hairline a point tall rather than a pixel.
typedef struct {
  /// A sounding bar's colour, and a silent bar's.
  float color[4];
  float flat_color[4];
  float viewport[2];
  float origin_x;
  float bar_pitch;
  float bar_width;
  /// The tallest half-height a bar may reach.
  float maximum;
  float scale;
  uint32_t bar_count;
} ScreenwideAudioBarsUniforms;
_Static_assert(sizeof(ScreenwideAudioBarsUniforms) == 64,
               "Metal/C audio bars uniform layout mismatch");

typedef double (*ScreenwideAudioClockRead)(void *, double);
typedef void (*ScreenwideAudioClockRelease)(void *);

@interface ScreenwideAudioRibbonClock : NSObject
@property(nonatomic) void *context;
@property(nonatomic) ScreenwideAudioClockRead read;
@property(nonatomic) ScreenwideAudioClockRelease releaseContext;
@end

@class ScreenwidePreviewSurface;

/// Everything the bars own: their layer, pipeline, the envelopes they sample
/// and the display link that follows the playhead. One per preview surface.
@interface ScreenwideAudioRibbon : NSObject {
@public
  /// A C array cannot be a property, and this is read for every bar drawn.
  float gains[SCREENWIDE_AUDIO_RIBBON_MAX_TRACKS];
  /// The colours the last frame was drawn in, so a change redraws at once.
  float drawnColor[8];
}
@property(nonatomic, strong) CAMetalLayer *layer;
@property(nonatomic, strong) id<MTLRenderPipelineState> pipeline;
/// One R32Float row of per-bar amplitudes, rewritten each drawn frame.
@property(nonatomic, strong) id<MTLTexture> bars;
@property(nonatomic, strong) NSMutableData *levels;
@property(nonatomic, strong) NSData *samples;
@property(nonatomic, strong) CADisplayLink *displayLink;
@property(nonatomic, weak) ScreenwidePreviewSurface *surface;
- (void)displayFrame:(CADisplayLink *)link;
@property(nonatomic) uint32_t trackCount;
@property(nonatomic) uint32_t pointCount;
@property(nonatomic) double playheadRatio;
@property(nonatomic, strong) ScreenwideAudioRibbonClock *clock;
/// What the last drawn frame was made of. A frame identical to it is skipped,
/// so a parked playhead over silence costs nothing at all.
@property(nonatomic) double drawnPlayhead;
@property(nonatomic) BOOL hasDrawn;
@property(nonatomic) BOOL drawInFlight;
@end

void screenwide_audio_ribbon_attach(ScreenwidePreviewSurface *surface,
                                    id<MTLLibrary> library);
/// Follows the viewport: the bars fill whatever the container covers.
void screenwide_audio_ribbon_layout(ScreenwidePreviewSurface *surface);
void screenwide_audio_ribbon_detach(ScreenwidePreviewSurface *surface);
/// Draws one frame now, unless it would be identical to the last one.
void screenwide_audio_ribbon_draw(ScreenwidePreviewSurface *surface);
/// Shows the bars only while they have envelopes, and runs the display link to match.
void screenwide_audio_ribbon_update_visibility(ScreenwidePreviewSurface *surface);

#endif
