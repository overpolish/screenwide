// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

/// The shade a layer's spotlights cast. Every spotlight showing cuts its hole
/// in one shared shade, which darkens what is under it by a fixed share.
/// Every kernel runs it after the picture and the cursor and before the
/// composite pass, so what other annotations draw stays at full brightness.
/// The arithmetic is Rust's `spotlight::geometry::{light, shade}`, which the
/// tests hold this to; the blur a spotlight can add is applied to the source
/// with the redactions, in `..._redact.h`.
#define GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_SPOTLIGHT @R"METAL(
constant uint annotation_spotlight_kind = 6u;

/// How much of the light the shade takes away. The twin of Rust's
/// `spotlight::model::SPOTLIGHT_DIM`.
constant float annotation_spotlight_dim = 0.4;
/// The flag a spotlight that blurs carries: `flags::BLUR`.
constant uint annotation_spotlight_blur = 1u << 3;

/// How much of one spotlight's light reaches `point`: all of it well inside
/// the box, none outside it, and a smooth fall over its softness in from the
/// edge. `feather` is one drawn pixel, which is all a spotlight with no
/// softness fades over.
static float annotation_spotlight_light(const device AnnotationUniforms &annotation,
                                        float2 point, float feather) {
  float2 low = float2(annotation.arrow.a);
  float2 high = float2(annotation.arrow.b);
  float rounding = min(annotation.arrow.rounding, min(high.x - low.x, high.y - low.y) * 0.5);
  float2 q = abs(point - (low + high) * 0.5) - ((high - low) * 0.5 - rounding);
  float distance = length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - rounding;
  float soft = max(annotation.arrow.width, 0.0);
  float fall = saturate((distance + soft + feather * 0.5) / (soft + feather));
  return 1.0 - fall * fall * (3.0 - 2.0 * fall);
}

/// How much of a spotlight layer reaches `point`, from every spotlight on the
/// layer `above_camera` names: as strong as the most present spotlight that
/// counts - every one for the shade, those that blur for the blur - lifted by
/// each one's light as far as it is present. A spotlight's presence rides in
/// its colour's alpha, which the binding scales by its reveal. The editor's
/// passes blur the source with the redactions; only the live overlay, which
/// has no source, asks for the blur here.
static float annotation_spotlight_cover(
    const device AnnotationUniforms *annotations, uint count, uint above_camera,
    float2 point, float pixel_scale, bool blurring) {
  float feather = max(pixel_scale, 1e-4);
  float layer = 0.0, lit = 0.0;
  for (uint index = 0; index < count; ++index) {
    const device AnnotationUniforms &annotation = annotations[index];
    if (annotation.kind != annotation_spotlight_kind ||
        annotation.above_camera != above_camera)
      continue;
    float presence = saturate(float(annotation.color.a));
    if (presence <= 0.0) continue;
    if (!blurring || (annotation.flags & annotation_spotlight_blur) != 0u)
      layer = max(layer, presence);
    // Only a point inside the box can be lit; everywhere else is shade.
    float2 reach = float2(feather + 1.0);
    if (any(point < float2(annotation.arrow.a) - reach) ||
        any(point > float2(annotation.arrow.b) + reach))
      continue;
    lit = max(lit, presence * annotation_spotlight_light(annotation, point, feather));
  }
  return max(layer - lit, 0.0);
}

/// `rgba`, premultiplied, under the shade the spotlights on the layer
/// `above_camera` names cast at `point`.
static float4 composite_spotlights(
    float4 rgba, const device AnnotationUniforms *annotations, uint count,
    uint above_camera, float2 point, float pixel_scale) {
  float shade = annotation_spotlight_cover(annotations, count, above_camera, point,
                                           pixel_scale, false);
  rgba.rgb *= 1.0 - annotation_spotlight_dim * shade;
  return rgba;
}
)METAL"
