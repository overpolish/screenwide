// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/// One drawn frame of the audio-only preview's bars.
///
/// Each bar belongs to a fixed envelope bucket. The shader receives its
/// amplitude and draws the capsule and anti-aliasing; only the row moves.

#import "recording_preview_surface_macos_private.h"

#include <math.h>

/// Two points of bar with a three-point gap.
static const double SCREENWIDE_BARS_WIDTH_POINTS = 2.0;
static const double SCREENWIDE_BARS_PITCH_POINTS = 5.0;
/// A bar reaches this share of the viewport's height at most, half of it
/// either side of the centre line.
static const double SCREENWIDE_BARS_HEIGHT_SHARE = 0.30;
/// Envelope points averaged into one bar. One point per bar flickered as the
/// playhead moved between points; a bucket of a few reads as a waveform.
static const double SCREENWIDE_BARS_POINTS_PER_BAR = 4.0;
/// Below this a bar has settled, and a frame of only settled bars is skipped.
static const float SCREENWIDE_BARS_SETTLED = 0.001f;
/// The recording is the accent, opaque, and what lies outside it the neutral
/// at the tertiary label's translucency.
static const float SCREENWIDE_BARS_ACCENT_ALPHA = 1.0f;
static const float SCREENWIDE_BARS_FLAT_ALPHA = 0.25f;

/// A bar that lies before the recording starts or after it ends. It is drawn
/// in the neutral so the row says where the audio actually is.
static const float SCREENWIDE_BARS_OUTSIDE = -1.0f;

/// The loudest enabled track over one bucket of envelope points, as the mean
/// of the bucket, or `SCREENWIDE_BARS_OUTSIDE` for a bucket the recording
/// does not reach.
static float audio_bars_amplitude(ScreenwideAudioRibbon *bars, double bucket) {
  if (bars.pointCount == 0 || bars.samples.length == 0 || bucket < 0.0)
    return SCREENWIDE_BARS_OUTSIDE;
  const float *samples = bars.samples.bytes;
  NSUInteger points = bars.pointCount;
  NSUInteger first = (NSUInteger)(bucket * SCREENWIDE_BARS_POINTS_PER_BAR);
  if (first >= points) return SCREENWIDE_BARS_OUTSIDE;
  NSUInteger last = MIN(first + (NSUInteger)SCREENWIDE_BARS_POINTS_PER_BAR, points);
  float level = 0.0f;
  for (uint32_t track = 0; track < bars.trackCount; track++) {
    NSUInteger row = (NSUInteger)track * points;
    if ((row + points) * sizeof(float) > bars.samples.length) break;
    float sum = 0.0f;
    for (NSUInteger index = first; index < last; index++) sum += samples[row + index];
    level = MAX(level, sum / (float)(last - first) * bars->gains[track]);
  }
  return level;
}

/// How many bars cover the viewport edge to edge, with a spare either side so
/// the row can slide by up to a pitch, and where the run starts once the
/// middle bar is centred and the row is slid back by the playhead's progress
/// through its bucket. Both are in drawable pixels.
static uint32_t audio_bars_run(double width, double scale, double fraction,
                               double *origin_x) {
  double pitch = SCREENWIDE_BARS_PITCH_POINTS * scale;
  double count = ceil(width / pitch) + 2.0;
  uint32_t bars = (uint32_t)MIN(MAX(count, 3.0), (double)SCREENWIDE_AUDIO_RIBBON_MAX_BARS);
  double middle = (double)(bars / 2u);
  double radius = SCREENWIDE_BARS_WIDTH_POINTS * scale * 0.5;
  *origin_x = width * 0.5 - middle * pitch - radius - fraction * pitch;
  return bars;
}

/// Samples every bar. Returns whether the row changed at all: an unchanged
/// row is not worth a drawable.
static BOOL update_audio_bars_levels(ScreenwideAudioRibbon *bars,
                                     double centre_bucket, uint32_t count) {
  if (bars.levels.length != sizeof(float) * count) {
    bars.levels = [NSMutableData dataWithLength:sizeof(float) * count];
    bars.hasDrawn = NO;
  }
  float *levels = bars.levels.mutableBytes;
  double middle = (double)(count / 2u);
  BOOL changed = NO;
  for (uint32_t index = 0; index < count; index++) {
    // A waveform passing under the playhead: the bars are fixed to the
    // recording's buckets, the middle bar holds the bucket the playhead is
    // in, the past to its left and what is still to come to its right. The
    // row slides rather than the bars changing, so nothing flickers.
    double bucket = floor(centre_bucket) + ((double)index - middle);
    float next = audio_bars_amplitude(bars, bucket);
    if (fabsf(next - levels[index]) > SCREENWIDE_BARS_SETTLED) changed = YES;
    levels[index] = next;
  }
  return changed;
}

/// Resolve dynamic colours against the host, independent of whether a frame
/// came from a display-link callback or an IPC block on the main queue.
static NSColor *audio_bars_accent(NSAppearance *appearance) {
  __block NSColor *accent;
  [appearance performAsCurrentDrawingAppearance:^{
    accent = [NSColor.controlAccentColor colorUsingColorSpace:NSColorSpace.sRGBColorSpace];
  }];
  return accent;
}

SCREENWIDE_PREVIEW_PRIVATE void screenwide_audio_ribbon_draw(
    ScreenwidePreviewSurface *surface) {
  ScreenwideAudioRibbon *bars = surface.audioRibbon;
  if (bars == nil || bars.pipeline == nil) return;
  if (bars.trackCount == 0 || bars.layer.hidden) return;
  if (surface.container.hidden || bars.drawInFlight) return;
  CGSize size = bars.layer.drawableSize;
  if (size.width < 2.0 || size.height < 2.0) return;
  double scale = surface.host.window.backingScaleFactor ?: 1.0;
  double playhead = bars.playheadRatio;
  // Where the playhead stands in bucket units: the whole part picks the
  // middle bar's bucket, the fraction slides the row.
  double centre_bucket = playhead * (double)(MAX(bars.pointCount, 1u) - 1u) /
                         SCREENWIDE_BARS_POINTS_PER_BAR;
  double origin_x = 0.0;
  uint32_t count = audio_bars_run(size.width, scale,
                                  centre_bucket - floor(centre_bucket), &origin_x);
  BOOL changed = update_audio_bars_levels(bars, centre_bucket, count);
  // The accent is read every frame so a change in System Settings shows at
  // once, not on the next move of the playhead.
  NSAppearance *appearance = surface.host.effectiveAppearance;
  NSColor *accent = audio_bars_accent(appearance);
  // The accent stands in for the UI's translucent tint, and silence takes the
  // neutral the UI uses for its quietest content: black or white at 25%,
  // depending on the appearance, the same as the tertiary label colour.
  BOOL light = [[appearance bestMatchFromAppearancesWithNames:@[
    NSAppearanceNameAqua, NSAppearanceNameDarkAqua
  ]] isEqualToString:NSAppearanceNameAqua];
  float neutral = light ? 0.0f : 1.0f;
  float color[8] = {accent != nil ? (float)accent.redComponent : 0.35f,
                    accent != nil ? (float)accent.greenComponent : 0.55f,
                    accent != nil ? (float)accent.blueComponent : 0.95f,
                    SCREENWIDE_BARS_ACCENT_ALPHA,
                    neutral, neutral, neutral, SCREENWIDE_BARS_FLAT_ALPHA};
  BOOL recolored = memcmp(color, bars->drawnColor, sizeof(color)) != 0;
  // Nothing moves on its own: a parked playhead in the same colour produces
  // the same picture, so no frame is drawn at all.
  if (!changed && !recolored && bars.hasDrawn && bars.drawnPlayhead == playhead)
    return;
  if (bars.bars.width != count) {
    MTLTextureDescriptor *descriptor = [MTLTextureDescriptor
        texture2DDescriptorWithPixelFormat:MTLPixelFormatR32Float
                                     width:count
                                    height:1
                                 mipmapped:NO];
    descriptor.usage = MTLTextureUsageShaderRead;
    bars.bars = [surface.device newTextureWithDescriptor:descriptor];
  }
  if (bars.bars == nil) return;
  [bars.bars replaceRegion:MTLRegionMake2D(0, 0, count, 1)
               mipmapLevel:0
                 withBytes:bars.levels.bytes
               bytesPerRow:sizeof(float) * (size_t)count];
  id<CAMetalDrawable> drawable = [bars.layer nextDrawable];
  if (drawable == nil) return;

  ScreenwideAudioBarsUniforms uniforms = {0};
  memcpy(uniforms.color, color, sizeof(uniforms.color));
  memcpy(uniforms.flat_color, color + 4, sizeof(uniforms.flat_color));
  uniforms.viewport[0] = (float)size.width;
  uniforms.viewport[1] = (float)size.height;
  uniforms.origin_x = (float)origin_x;
  uniforms.bar_pitch = (float)(SCREENWIDE_BARS_PITCH_POINTS * scale);
  uniforms.bar_width = (float)(SCREENWIDE_BARS_WIDTH_POINTS * scale);
  uniforms.maximum = (float)(size.height * SCREENWIDE_BARS_HEIGHT_SHARE);
  uniforms.scale = (float)scale;
  uniforms.bar_count = count;

  MTLRenderPassDescriptor *pass = [MTLRenderPassDescriptor renderPassDescriptor];
  pass.colorAttachments[0].texture = drawable.texture;
  pass.colorAttachments[0].loadAction = MTLLoadActionClear;
  pass.colorAttachments[0].storeAction = MTLStoreActionStore;
  pass.colorAttachments[0].clearColor = MTLClearColorMake(0, 0, 0, 0);
  id<MTLCommandBuffer> command = [surface.queue commandBuffer];
  id<MTLRenderCommandEncoder> encoder =
      [command renderCommandEncoderWithDescriptor:pass];
  [encoder setRenderPipelineState:bars.pipeline];
  [encoder setFragmentBytes:&uniforms length:sizeof(uniforms) atIndex:0];
  [encoder setFragmentTexture:bars.bars atIndex:0];
  [encoder drawPrimitives:MTLPrimitiveTypeTriangle vertexStart:0 vertexCount:3];
  [encoder endEncoding];
  bars.drawInFlight = YES;
  bars.hasDrawn = YES;
  bars.drawnPlayhead = playhead;
  memcpy(bars->drawnColor, color, sizeof(color));
  [command presentDrawable:drawable];
  [command addCompletedHandler:^(__unused id<MTLCommandBuffer> completed) {
    dispatch_async(dispatch_get_main_queue(), ^{
      bars.drawInFlight = NO;
    });
  }];
  [command commit];
}

int screenwide_audio_ribbon_accent_is_stable(void) {
  @autoreleasepool {
    NSAppearance *light = [NSAppearance appearanceNamed:NSAppearanceNameAqua];
    NSAppearance *dark = [NSAppearance appearanceNamed:NSAppearanceNameDarkAqua];
    __block NSColor *fromLight;
    __block NSColor *fromDark;
    [light performAsCurrentDrawingAppearance:^{ fromLight = audio_bars_accent(dark); }];
    [dark performAsCurrentDrawingAppearance:^{ fromDark = audio_bars_accent(dark); }];
    return fromLight != nil && fromDark != nil &&
        fromLight.redComponent == fromDark.redComponent &&
        fromLight.greenComponent == fromDark.greenComponent &&
        fromLight.blueComponent == fromDark.blueComponent;
  }
}
