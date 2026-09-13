// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#ifndef SCREENWIDE_RECORDING_PREVIEW_AUDIO_RIBBON_SHADER_H
#define SCREENWIDE_RECORDING_PREVIEW_AUDIO_RIBBON_SHADER_H

#import <Foundation/Foundation.h>

/// The audio-only preview's bars, appended to the preview surface's Metal
/// library. One full-viewport quad draws the whole row: every bar is a pixel-covered
/// capsule, so there is no geometry per bar and no texture filtering
/// to go soft at a fractional backing scale.
///
/// The heights arrive already sampled, one value per bar in an
/// R32Float row. Nothing here moves on its own: with the playhead parked over
/// silence the same picture is produced every frame, and the draw is skipped.
static NSString *const screenwide_audio_bars_shader = @R"(
#include <metal_stdlib>
using namespace metal;

struct ScreenwideAudioBarsUniforms {
  float4 color;
  float4 flat_color;
  float2 viewport;
  float origin_x;
  float bar_pitch;
  float bar_width;
  float maximum;
  float scale;
  uint bar_count;
};

struct ScreenwideAudioBarsVertex {
  float4 position [[position]];
};

vertex ScreenwideAudioBarsVertex audio_bars_vertex(uint id [[vertex_id]]) {
  // One oversized triangle covers the viewport with no vertex buffer.
  float2 corner = float2(float((id << 1) & 2), float(id & 2));
  ScreenwideAudioBarsVertex out;
  out.position = float4(corner * 2.0 - 1.0, 0.0, 1.0);
  return out;
}

fragment float4 audio_bars(ScreenwideAudioBarsVertex in [[stage_in]],
                           constant ScreenwideAudioBarsUniforms &u [[buffer(0)]],
                           texture2d<float, access::read> bars [[texture(0)]]) {
  float2 pixel = in.position.xy;
  float pitch = max(u.bar_pitch, 1.0);
  float radius = u.bar_width * 0.5;
  // The nearest bar centre, not the column the pixel falls in: a pixel in the
  // gap belongs to the bar it is closest to, so both edges of every bar get
  // the same coverage treatment.
  float column = round((pixel.x - u.origin_x - radius) / pitch);
  if (column < 0.0 || column >= float(u.bar_count)) return float4(0.0);
  uint index = uint(column);

  // A negative sample marks a bar outside the recording.
  float sample = bars.read(uint2(index, 0)).r;
  bool outside = sample < 0.0;
  float amplitude = clamp(sample, 0.0, 1.0);
  // A hairline stays visible through silence, so the row never disappears.
  float half_height = max(u.scale, sqrt(amplitude) * u.maximum);

  float centre_x = u.origin_x + column * pitch + radius;
  float2 point = float2(pixel.x - centre_x, pixel.y - u.viewport.y * 0.5);
  // A capsule: a segment of the bar's straight part, thickened by its radius.
  float straight = max(half_height - radius, 0.0);
  float vertical_distance = max(abs(point.y) - straight, 0.0);
  float cross_section = sqrt(max(radius * radius -
                                     vertical_distance * vertical_distance,
                                 0.0));

  // Integrate the capsule's horizontal cross-section over this pixel instead
  // of feathering its signed distance. The interval overlap is exact for the
  // sampled row and therefore sums to the same area for every fractional x
  // translation. In particular, two partially covered edge pixels always add
  // up to the same coverage as one fully covered pixel.
  float pixel_width = max(fwidth(pixel.x), 1.0);
  float pixel_half = pixel_width * 0.5;
  float pixel_left = pixel.x - pixel_half;
  float pixel_right = pixel.x + pixel_half;
  float shape_left = centre_x - cross_section;
  float shape_right = centre_x + cross_section;
  float overlap = max(min(pixel_right, shape_right) -
                          max(pixel_left, shape_left),
                      0.0);
  float coverage = clamp(overlap / pixel_width, 0.0, 1.0);

  // The recording is drawn in the accent, quiet passages included; only the
  // bars before its start and after its end take the neutral, so the row
  // shows where the audio actually is.
  float4 color = outside ? u.flat_color : u.color;
  float alpha = color.a * coverage;
  return float4(color.rgb * alpha, alpha);
}
)";

#endif
