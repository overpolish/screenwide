// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/// The bars' inputs: enabled tracks' envelopes and gains, and the source
/// position selected by the audio playback clock. Both entry points are called
/// from Rust off the main thread, so each copies its arguments and applies
/// them on the main queue.

#import "recording_preview_surface_macos_private.h"

#include <math.h>
#include <string.h>

SCREENWIDE_PREVIEW_PRIVATE void on_main_async(dispatch_block_t block);

void screenwide_preview_surface_set_audio_ribbon(
    void *handle, const float *samples, uint32_t track_count,
    uint32_t point_count, const float *gains) {
  if (handle == NULL) return;
  ScreenwidePreviewSurface *surface = (__bridge ScreenwidePreviewSurface *)handle;
  // Extra rows can be dropped, but a row longer than the bars read cannot be
  // trimmed without mis-striding the rest: Rust clamps both, and anything past
  // the limit here clears rather than guesses.
  uint32_t tracks = MIN(track_count, SCREENWIDE_AUDIO_RIBBON_MAX_TRACKS);
  uint32_t points = point_count;
  if (samples == NULL || gains == NULL || tracks == 0 || points == 0 ||
      points > SCREENWIDE_AUDIO_RIBBON_MAX_POINTS) {
    tracks = 0;
    points = 0;
  }
  // The block outlives this call, so the caller's arrays are copied here.
  NSData *copied = tracks == 0
      ? [NSData data]
      : [NSData dataWithBytes:samples
                       length:sizeof(float) * (size_t)tracks * (size_t)points];
  NSData *copiedGains = tracks == 0
      ? [NSData data]
      : [NSData dataWithBytes:gains length:sizeof(float) * (size_t)tracks];
  on_main_async(^{
    ScreenwideAudioRibbon *ribbon = surface.audioRibbon;
    if (ribbon == nil) return;
    ribbon.trackCount = tracks;
    ribbon.pointCount = points;
    ribbon.samples = copied;
    float resolved[SCREENWIDE_AUDIO_RIBBON_MAX_TRACKS] = {0};
    if (copiedGains.length >= sizeof(float) * (size_t)tracks)
      memcpy(resolved, copiedGains.bytes, sizeof(float) * (size_t)tracks);
    for (uint32_t track = 0; track < SCREENWIDE_AUDIO_RIBBON_MAX_TRACKS; track++)
      ribbon->gains[track] = resolved[track];
    // A changed track selection or volume must be read again from scratch:
    // the cached heights belong to the envelopes that produced them.
    ribbon.levels = nil;
    ribbon.hasDrawn = NO;
    screenwide_audio_ribbon_update_visibility(surface);
    if (tracks > 0) screenwide_audio_ribbon_draw(surface);
  });
}

@implementation ScreenwideAudioRibbonClock
- (void)dealloc {
  if (_releaseContext != NULL) _releaseContext(_context);
}
@end

/// Transfer a timestamped device clock to the display link, with native lifetime.
void screenwide_preview_surface_set_audio_ribbon_clock(
    void *handle, void *context, ScreenwideAudioClockRead read,
    ScreenwideAudioClockRelease release_context) {
  ScreenwideAudioRibbonClock *clock = [ScreenwideAudioRibbonClock new];
  clock.context = context;
  clock.read = read;
  clock.releaseContext = release_context;
  if (handle == NULL) return;
  ScreenwidePreviewSurface *surface = (__bridge ScreenwidePreviewSurface *)handle;
  on_main_async(^{
    surface.audioRibbon.clock = clock;
  });
}

/// The worker supplies a position only after the audio clock reaches it.
/// No extrapolation or pause offset survives between presentations.
void screenwide_preview_surface_set_audio_ribbon_playhead(void *handle, double ratio) {
  if (handle == NULL) return;
  ScreenwidePreviewSurface *surface = (__bridge ScreenwidePreviewSurface *)handle;
  double safeRatio = isfinite(ratio) ? MIN(MAX(ratio, 0.0), 1.0) : 0.0;
  on_main_async(^{
    ScreenwideAudioRibbon *ribbon = surface.audioRibbon;
    if (ribbon == nil) return;
    ribbon.clock = nil;
    ribbon.playheadRatio = safeRatio;
    screenwide_audio_ribbon_draw(surface);
  });
}
