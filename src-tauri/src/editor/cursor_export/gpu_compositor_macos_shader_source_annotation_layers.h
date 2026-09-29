// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

/// The editor's annotations, drawn in the order the document keeps them. It
/// comes after every kind's own source, the spotlight's shade included.
#define GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_LAYERS @R"METAL(
/// Draws every annotation whose layer matches `above_camera` over `rgba`, the
/// first in the document at the bottom. A highlight recolours whatever is
/// under it by then. The spotlights share one shade, laid where the topmost of
/// them sits: it darkens the picture and every annotation below that
/// spotlight, and nothing above it. `shade_gate` scales the shade, for a pass
/// redrawn over a picture that already carries it outside the gate.
///
/// `pixel_scale` is how many canvas pixels one drawn pixel covers. Every edge
/// is feathered over that, not over one canvas pixel: a workspace layer drawn
/// smaller than its canvas would otherwise take its whole antialiasing band
/// from inside a single drawn pixel and come out jagged.
static float4 composite_annotation_layers(
    float4 rgba, const device AnnotationUniforms *annotations, uint count,
    uint above_camera, float2 point, float2 source_dimensions, float pixel_scale,
    float shade_gate, const device AnnotationSample *samples, const device uchar4 *numbers,
    AnnotationTextAtlas number_atlas, const device packed_float2 *points) {
  if (count == 0u || any(source_dimensions <= 0.0)) return rgba;
  float feather = max(pixel_scale, 1e-4) * 0.5;
  uint shade_at = count;
  for (uint index = 0; index < count; ++index)
    if (annotations[index].above_camera == above_camera &&
        annotations[index].kind == annotation_spotlight_kind)
      shade_at = index;
  for (uint index = 0; index < count; ++index) {
    const device AnnotationUniforms &annotation = annotations[index];
    if (annotation.above_camera != above_camera) continue;
    if (index == shade_at) {
      float shade = annotation_spotlight_cover(annotations, count, above_camera, point,
                                               pixel_scale, false);
      rgba.rgb *= 1.0 - annotation_spotlight_dim * shade * shade_gate;
    }
    rgba = annotation.kind == annotation_highlight_kind
        ? annotation_highlight_layer(rgba, rgba, annotation, point, feather, points, samples)
        : annotation_layer(rgba, annotation, point, feather, samples, numbers, number_atlas,
                           points);
  }
  return rgba;
}
)METAL"
