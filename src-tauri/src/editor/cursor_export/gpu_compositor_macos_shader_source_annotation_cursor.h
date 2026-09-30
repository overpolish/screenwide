// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

/// The cursor as the annotation layers see it. `composite_annotation_layers`
/// draws it over every mark but treats it as lying on the picture: the
/// spotlights' shade darkens it and a loupe hides it and shows it enlarged.
/// The spotlights' blur is applied to the source before the cursor is drawn,
/// so the cursor is softened here by the same deviation and the same share.
///
/// A kernel names its cursor with a type of its own, as it names its picture
/// with a magnifier's tap: `AnnotationCursorNone` where the layer carries no
/// cursor, and one that samples the kernel's artwork beside it. Each has an
/// `annotation_cursor_sample` overload and carries the spotlights its blur is
/// lifted by.
#define GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_CURSOR @R"METAL(
/// The spotlights' blur as the canvas draws it: its standard deviation in
/// canvas pixels, and how far it has arrived - as present as the most present
/// spotlight that blurs. Both zero where no spotlight showing blurs. The twin
/// of the blur record `screenwide_redactions` appends.
struct AnnotationCursorBlur {
  float deviation;
  float strength;
};

/// What lifts the cursor's blur: the list the spotlights are in, and the
/// blur they cast.
struct AnnotationCursorSpotlights {
  const device AnnotationUniforms *annotations;
  uint count;
  AnnotationCursorBlur blur;
};

static float annotation_spotlight_light(const device AnnotationUniforms &annotation,
                                        float2 point, float feather);

/// How much of the pixel at `point` the spotlights' blur takes: all of it,
/// lifted by each spotlight's light as far as its presence against the
/// blur's. The twin of `redact_spotlight_share`, which blurs the source.
static float annotation_spotlight_blur_share(AnnotationCursorSpotlights spotlights,
                                             float2 point, float feather) {
  float strength = spotlights.blur.strength;
  if (strength <= 0.0) return 0.0;
  float lit = 0.0;
  for (uint index = 0; index < spotlights.count; ++index) {
    const device AnnotationUniforms &annotation = spotlights.annotations[index];
    if (annotation.kind != 6u) continue;
    float presence = min(saturate(float(annotation.color.a)) / strength, 1.0);
    if (presence <= 0.0) continue;
    float2 reach = float2(feather + 1.0);
    if (any(point < float2(annotation.arrow.a) - reach) ||
        any(point > float2(annotation.arrow.b) + reach))
      continue;
    lit = max(lit, presence * annotation_spotlight_light(annotation, point, feather));
  }
  return saturate(1.0 - lit);
}

/// A layer that carries no cursor: the camera's, and the live overlay's.
struct AnnotationCursorNone {
  AnnotationCursorSpotlights spotlights;
};

static float4 annotation_cursor_sample(AnnotationCursorNone, float2) {
  return float4(0.0);
}

/// How many taps the cursor's blur takes, spiralling out to two deviations.
constant uint annotation_cursor_blur_taps = 16u;

/// The cursor at `point`, straight alpha, as the layer shows it: sharp, or
/// softened by the spotlights' blur where that reaches. The blur is a spiral
/// of taps evenly spread over a disc two deviations wide and weighted as a
/// Gaussian, which the small cursor can afford at every pixel it covers.
/// `feather` is one drawn pixel, which a spotlight's hard edge fades over.
template <typename Cursor>
static float4 annotation_cursor_seen(Cursor cursor, float2 point, float feather) {
  float4 sharp = annotation_cursor_sample(cursor, point);
  AnnotationCursorBlur blur = cursor.spotlights.blur;
  if (!(blur.deviation >= 0.5) || blur.strength <= 0.0) return sharp;
  float4 still = float4(sharp.rgb * sharp.a, sharp.a);
  float4 soft = still;
  float weights = 1.0;
  for (uint tap = 0u; tap < annotation_cursor_blur_taps; ++tap) {
    float share = (float(tap) + 0.5) / float(annotation_cursor_blur_taps);
    float radius = 2.0 * blur.deviation * sqrt(share);
    float angle = float(tap) * 2.39996323;
    float weight = exp(-2.0 * share);
    float4 sample = annotation_cursor_sample(cursor, point + radius * float2(cos(angle), sin(angle)));
    soft += weight * float4(sample.rgb * sample.a, sample.a);
    weights += weight;
  }
  soft /= weights;
  if (soft.a <= 0.0 && still.a <= 0.0) return float4(0.0);
  float4 seen = mix(still, soft, annotation_spotlight_blur_share(cursor.spotlights, point, feather));
  return seen.a > 0.0 ? float4(seen.rgb / seen.a, seen.a) : float4(0.0);
}
)METAL"
