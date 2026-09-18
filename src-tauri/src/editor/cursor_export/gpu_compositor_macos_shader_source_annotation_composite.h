// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

/// The pass that draws a layer's annotations, whatever shape each one is. It
/// comes after every shape's own source: each kind is a branch here and a layer
/// function of its own beside it.
#define GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_COMPOSITE @R"METAL(
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
    uint2 number_atlas) {
  if (count == 0u || any(source_dimensions <= 0.0)) return rgba;
  float feather = max(pixel_scale, 1e-4) * 0.5;
  for (uint index = 0; index < count; ++index) {
    const device AnnotationUniforms &annotation = annotations[index];
    if (annotation.above_camera != above_camera) continue;
    float4 color = float4(annotation.color);
    if (color.a <= 0.0 || annotation.arrow.width <= 0.0) continue;
    float halo = max(annotation.hover, 0.0);
    if (annotation.kind == 1u) {
      rgba = annotation_counter_layer(rgba, annotation, color, canvas_point, feather,
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
      float band = smoothstep(-feather, feather, distance) *
          (1.0 - smoothstep(halo - feather, halo + feather, distance));
      float alpha = band * color.a * annotation_hover_alpha;
      if (alpha > 0.0) {
        rgba.rgb = color.rgb * alpha + rgba.rgb * (1.0 - alpha);
        rgba.a = alpha + rgba.a * (1.0 - alpha);
      }
    }
    // A still frame draws the prepared arrow directly; a moving one averages
    // the arrow over the exposure, head and shaft together.
    float coverage = annotation.sample_count == 0u
        ? max(1.0 - smoothstep(-feather, feather, distances.x),
              1.0 - smoothstep(-feather, feather, distances.y))
        : annotation_exposure(canvas_point, annotation, samples, feather);
    if (coverage <= 0.0) continue;
    float alpha = coverage * color.a;
    rgba.rgb = color.rgb * alpha + rgba.rgb * (1.0 - alpha);
    rgba.a = alpha + rgba.a * (1.0 - alpha);
  }
  return rgba;
}
)METAL"
