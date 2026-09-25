// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

/// The pass that draws a layer's annotations, whatever shape each one is. It
/// comes after every shape's own source: each kind is a branch here and a layer
/// function of its own beside it.
#define GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_COMPOSITE @R"METAL(
/// A hovered redaction's halo, drawn by the canvas pass like every other
/// kind's. The box itself was applied to the source and is not drawn here, so
/// this is the only thing that finds an erased box on its own surface: it is
/// white round a dark fill and black round a light one, and stronger than the
/// halo a coloured shape wears in its own colour.
constant float annotation_redact_halo_alpha = 0.5;

static float4 annotation_redact_halo(
    float4 rgba, const device AnnotationUniforms &annotation, float2 point,
    float feather) {
  float halo = max(annotation.hover, 0.0);
  if (halo <= 0.0) return rgba;
  float2 low = float2(annotation.arrow.a);
  float2 high = float2(annotation.arrow.b);
  float reach = halo + feather + 1.0;
  if (any(point < low - reach) || any(point > high + reach)) return rgba;
  float rounding = min(annotation.arrow.rounding, min(high.x - low.x, high.y - low.y) * 0.5);
  float2 q = abs(point - (low + high) * 0.5) - ((high - low) * 0.5 - rounding);
  float distance = length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - rounding;
  float band = (1.0 - annotation_edge(distance, feather)) *
      annotation_edge(distance - halo, feather);
  float luminance = dot(float4(annotation.color).rgb, float3(0.2126, 0.7152, 0.0722));
  float3 tone = luminance > 0.5 ? float3(0.0) : float3(1.0);
  float alpha = band * annotation_redact_halo_alpha;
  rgba.rgb = tone * alpha + rgba.rgb * (1.0 - alpha);
  rgba.a = alpha + rgba.a * (1.0 - alpha);
  return rgba;
}

/// Draws every annotation whose layer matches `above_camera`.
///
/// Geometry is already in canvas pixels, prepared using the current image
/// placement before dispatch. Annotations are deliberately not
/// clipped to the crop: an arrow may point in from the padding.
///
/// `pixel_scale` is how many canvas pixels one drawn pixel covers. Every edge
/// is feathered over that, not over one canvas pixel: a workspace layer drawn
/// smaller than its canvas would otherwise take its whole antialiasing band
/// from inside a single drawn pixel and come out jagged.
static float4 composite_annotations(
    float4 rgba, const device AnnotationUniforms *annotations, uint count,
    uint above_camera, float2 canvas_point, constant CanvasUniforms &u,
    float2 source_dimensions, float pixel_scale,
    const device AnnotationSample *samples, const device uchar4 *numbers,
    AnnotationTextAtlas number_atlas) {
  if (count == 0u || any(source_dimensions <= 0.0)) return rgba;
  float feather = max(pixel_scale, 1e-4) * 0.5;
  for (uint index = 0; index < count; ++index) {
    const device AnnotationUniforms &annotation = annotations[index];
    if (annotation.above_camera != above_camera) continue;
    float4 color = float4(annotation.color);
    if (annotation.kind == 3u) {
      rgba = annotation_redact_halo(rgba, annotation, canvas_point, feather);
      continue;
    }
    if (color.a <= 0.0 || annotation.arrow.width <= 0.0) continue;
    float halo = max(annotation.hover, 0.0);
    if (annotation.kind == 1u) {
      rgba = annotation_counter_layer(rgba, annotation, color, canvas_point, feather,
                                      halo, samples, numbers, number_atlas);
      continue;
    }
    if (annotation.kind == 2u) {
      rgba = annotation_text_layer(rgba, annotation, color, canvas_point, feather,
                                   halo, samples, numbers, number_atlas);
      continue;
    }
    float2 a = float2(annotation.arrow.a);
    float2 b = float2(annotation.arrow.b);
    float2 c = float2(annotation.arrow.c);
    // The curve lies inside the hull of its three points; the heads reach
    // `head_length` back from a tip and `half_base` across. One rectangle
    // covers all of it, and keeps a long list cheap for most of the canvas.
    float radius = annotation.arrow.width * 0.5;
    float reach = radius * (annotation.arrow.head != 0u
        ? 9.0 : 1.0) + halo + feather + 1.0;
    if (any(canvas_point < min(a, min(b, c)) - reach) ||
        any(canvas_point > max(a, max(b, c)) + reach))
      continue;
    float2 distances = annotation_arrow_distance(canvas_point, annotation.arrow);
    float distance = min(distances.x, distances.y);
    if (halo > 0.0) {
      // The ruler's hover halo: an outline stroke in the shape's own colour,
      // hugging it from the edge outwards.
      float band = (1.0 - annotation_edge(distance, feather)) *
          annotation_edge(distance - halo, feather);
      float alpha = band * color.a * annotation_hover_alpha;
      if (alpha > 0.0) {
        rgba.rgb = color.rgb * alpha + rgba.rgb * (1.0 - alpha);
        rgba.a = alpha + rgba.a * (1.0 - alpha);
      }
    }
    // A still frame draws the prepared arrow directly; a moving one averages
    // the arrow over the exposure, head and shaft together.
    float coverage = annotation.sample_count == 0u
        ? max(annotation_edge(distances.x, feather), annotation_edge(distances.y, feather))
        : annotation_exposure(canvas_point, annotation, samples, feather);
    if (coverage <= 0.0) continue;
    float alpha = coverage * color.a;
    rgba.rgb = color.rgb * alpha + rgba.rgb * (1.0 - alpha);
    rgba.a = alpha + rgba.a * (1.0 - alpha);
  }
  return rgba;
}
)METAL"
