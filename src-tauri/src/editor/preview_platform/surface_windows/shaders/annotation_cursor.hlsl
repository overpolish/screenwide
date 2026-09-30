// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The cursor as the annotation layers see it. `composite_annotation_layers`
// draws it over every mark but treats it as lying on the picture: the
// spotlights' shade darkens it and a loupe hides it and shows it enlarged.
// The spotlights' blur is applied to the source before the cursor is drawn,
// so the cursor is softened here by the same deviation and the same share.
// The HLSL twin of `gpu_compositor_macos_shader_source_annotation_cursor.h`.
// Included by `annotations.hlsl`, after the spotlight.
//
// Each shader that draws annotations defines the two functions declared
// below over its own cursor: the preview's reads its cursor atlas, and the
// live overlay, which draws no cursor, reads nothing.

// The spotlights' blur as the canvas draws it: its standard deviation in
// canvas pixels and how far it has arrived - as present as the most present
// spotlight that blurs - both zero where none showing blurs; and how many
// prepared annotations the spotlights lifting it are among.
struct AnnotationCursorBlur {
  float deviation;
  float strength;
  uint count;
};

AnnotationCursorBlur annotation_cursor_blur();
// The cursor at `probe`, straight alpha, before any spotlight touches it.
float4 annotation_cursor_sample(float2 probe);

// How much of the pixel at `probe` the spotlights' blur takes: all of it,
// lifted by each spotlight's light as far as its presence against the
// blur's. The twin of `redact_spotlight_share`, which blurs the source.
float annotation_spotlight_blur_share(float2 probe, AnnotationCursorBlur blur, float pixel) {
  if (blur.strength <= 0.0) return 0.0;
  float lit = 0.0;
  for (uint index = 0; index < blur.count; ++index) {
    PreviewArrow annotation = annotation_arrows[index];
    if (annotation.kind != annotation_spotlight_kind) continue;
    float presence = min(saturate(annotation.alpha) / blur.strength, 1.0);
    if (presence <= 0.0) continue;
    PreviewGeometry shape = annotation.geometry;
    float reach = pixel + 1.0;
    if (probe.x < shape.ax - reach || probe.y < shape.ay - reach ||
        probe.x > shape.bx + reach || probe.y > shape.by + reach)
      continue;
    lit = max(lit, presence * annotation_spotlight_light(shape, probe, pixel));
  }
  return saturate(1.0 - lit);
}

// How many taps the cursor's blur takes, spiralling out to two deviations.
static const uint annotation_cursor_blur_taps = 16u;

// The cursor at `probe`, straight alpha, as the layer shows it: sharp, or
// softened by the spotlights' blur where that reaches. The blur is a spiral
// of taps evenly spread over a disc two deviations wide and weighted as a
// Gaussian. `pixel` is one drawn pixel, which a spotlight's hard edge fades
// over.
float4 annotation_cursor_seen(float2 probe, float pixel) {
  float4 sharp = annotation_cursor_sample(probe);
  AnnotationCursorBlur blur = annotation_cursor_blur();
  if (!(blur.deviation >= 0.5) || blur.strength <= 0.0) return sharp;
  float4 still = float4(sharp.rgb * sharp.a, sharp.a);
  float4 soft = still;
  float weights = 1.0;
  [loop] for (uint tap = 0u; tap < annotation_cursor_blur_taps; ++tap) {
    float share = ((float)tap + 0.5) / (float)annotation_cursor_blur_taps;
    float radius = 2.0 * blur.deviation * sqrt(share);
    float angle = (float)tap * 2.39996323;
    float weight = exp(-2.0 * share);
    float4 sample = annotation_cursor_sample(probe + radius * float2(cos(angle), sin(angle)));
    soft += weight * float4(sample.rgb * sample.a, sample.a);
    weights += weight;
  }
  soft /= weights;
  if (soft.a <= 0.0 && still.a <= 0.0) return 0.0;
  float4 seen = lerp(still, soft, annotation_spotlight_blur_share(probe, blur, pixel));
  return seen.a > 0.0 ? float4(seen.rgb / seen.a, seen.a) : 0.0;
}
