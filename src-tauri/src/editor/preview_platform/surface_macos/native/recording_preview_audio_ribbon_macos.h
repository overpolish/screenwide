// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#ifndef SCREENWIDE_RECORDING_PREVIEW_AUDIO_RIBBON_MACOS_H
#define SCREENWIDE_RECORDING_PREVIEW_AUDIO_RIBBON_MACOS_H

#import <AppKit/AppKit.h>
#import <QuartzCore/CAMetalLayer.h>
#import <QuartzCore/CADisplayLink.h>
#include <stdint.h>

/// Four tracks and 2048 envelope points are what the bars read; Rust clamps to
/// the same numbers before it uploads, so the payload is bounded whatever the
/// recording holds.
#define SCREENWIDE_AUDIO_RIBBON_MAX_TRACKS 4u
#define SCREENWIDE_AUDIO_RIBBON_MAX_POINTS 2048u

/// The shared wgpu renderer, in `surface_macos/audio_ribbon/renderer.rs`.
typedef struct NativeAudioRibbon NativeAudioRibbon;
NativeAudioRibbon *screenwide_audio_ribbon_renderer_create(void *layer);
void screenwide_audio_ribbon_renderer_destroy(NativeAudioRibbon *ribbon);
void screenwide_audio_ribbon_renderer_set_envelopes(NativeAudioRibbon *ribbon,
                                                    const float *samples,
                                                    uint32_t tracks,
                                                    uint32_t points,
                                                    const float *gains);
/// `colors` is the recording's RGBA, then the outside's. Returns 1 when a
/// frame was submitted.
int32_t screenwide_audio_ribbon_renderer_draw(NativeAudioRibbon *ribbon,
                                              uint32_t width, uint32_t height,
                                              float scale, float playhead,
                                              const float *colors);

typedef double (*ScreenwideAudioClockRead)(void *, double);
typedef void (*ScreenwideAudioClockRelease)(void *);

@interface ScreenwideAudioRibbonClock : NSObject
@property(nonatomic) void *context;
@property(nonatomic) ScreenwideAudioClockRead read;
@property(nonatomic) ScreenwideAudioClockRelease releaseContext;
@end

@class ScreenwidePreviewSurface;

/// Everything the bars own: their layer, the renderer that draws into it and
/// the display link that follows the playhead. One per preview surface.
@interface ScreenwideAudioRibbon : NSObject
@property(nonatomic, strong) CAMetalLayer *layer;
@property(nonatomic) NativeAudioRibbon *renderer;
@property(nonatomic, strong) CADisplayLink *displayLink;
@property(nonatomic, weak) ScreenwidePreviewSurface *surface;
- (void)displayFrame:(CADisplayLink *)link;
@property(nonatomic) uint32_t trackCount;
@property(nonatomic) uint32_t pointCount;
@property(nonatomic) double playheadRatio;
@property(nonatomic, strong) ScreenwideAudioRibbonClock *clock;
@end

void screenwide_audio_ribbon_attach(ScreenwidePreviewSurface *surface);
/// Follows the viewport: the bars fill whatever the container covers.
void screenwide_audio_ribbon_layout(ScreenwidePreviewSurface *surface);
void screenwide_audio_ribbon_detach(ScreenwidePreviewSurface *surface);
/// Draws one frame now, unless it would be identical to the last one or the
/// last one is still on the GPU.
void screenwide_audio_ribbon_draw(ScreenwidePreviewSurface *surface);
/// Shows the bars only while they have envelopes, and runs the display link to match.
void screenwide_audio_ribbon_update_visibility(ScreenwidePreviewSurface *surface);

#endif
