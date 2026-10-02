// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/// One drawn frame of the audio-only preview's bars. The shared renderer
/// places, sizes and paints them; this supplies what only AppKit knows: the
/// drawable size, the backing scale and the appearance's colours.

#import "recording_preview_surface_macos_private.h"

#include <math.h>

/// The recording is the accent, opaque, and what lies outside it the neutral
/// at the tertiary label's translucency.
static const float SCREENWIDE_BARS_ACCENT_ALPHA = 1.0f;
static const float SCREENWIDE_BARS_FLAT_ALPHA = 0.25f;

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
  if (bars == nil || bars.renderer == NULL) return;
  if (bars.trackCount == 0 || bars.layer.hidden) return;
  if (surface.container.hidden) return;
  double scale = surface.host.window.backingScaleFactor ?: 1.0;
  NSSize bounds = surface.container.bounds.size;
  double width = round(bounds.width * scale);
  double height = round(bounds.height * scale);
  if (width < 2.0 || height < 2.0) return;
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
  float colors[8] = {accent != nil ? (float)accent.redComponent : 0.35f,
                     accent != nil ? (float)accent.greenComponent : 0.55f,
                     accent != nil ? (float)accent.blueComponent : 0.95f,
                     SCREENWIDE_BARS_ACCENT_ALPHA,
                     neutral, neutral, neutral, SCREENWIDE_BARS_FLAT_ALPHA};
  screenwide_audio_ribbon_renderer_draw(bars.renderer, (uint32_t)width,
                                        (uint32_t)height, (float)scale,
                                        (float)bars.playheadRatio, colors);
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
