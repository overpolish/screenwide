// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

/// The editor's annotations and the cursor over them. It comes after every
/// kind's own source, the spotlight's shade included.
#define GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_LAYERS @R"METAL(
/// Whether a kind acts on the picture rather than marking it: a spotlight
/// shades and blurs it, and a magnifier enlarges it. They lie under every
/// mark, and over the cursor.
static bool annotation_acts_on_picture(uint kind) {
  return kind == annotation_spotlight_kind || kind == annotation_magnify_kind;
}

/// Draws every annotation whose layer matches `above_camera`, and the
/// cursor, over `rgba`. First what acts on the picture, in document order:
/// the spotlights share one shade, laid where the topmost of them sits among
/// the magnifiers, and a magnifier shows the picture and the cursor
/// enlarged. Then every mark, in document order, the first at the bottom,
/// none of them shaded; a highlight recolours whatever is under it by then.
/// Last the cursor, over every mark, which is where a viewer looks for it,
/// but treated as lying on the picture: the spotlights' shade darkens it and
/// their blur softens it, and a loupe hides it. `shade_gate` scales the shade
/// on the picture, for a pass redrawn over a picture that already carries it
/// outside the gate; the cursor is drawn afresh, and takes all of it.
///
/// `pixel_scale` is how many canvas pixels one drawn pixel covers. Every edge
/// is feathered over that, not over one canvas pixel: a workspace layer drawn
/// smaller than its canvas would otherwise take its whole antialiasing band
/// from inside a single drawn pixel and come out jagged. `tap` is where a
/// magnifier reads the picture it enlarges, and `cursor` the layer's cursor,
/// `AnnotationCursorNone` on a layer that carries none.
template <typename Tap, typename Cursor>
static float4 composite_annotation_layers(
    float4 rgba, const device AnnotationUniforms *annotations, uint count,
    uint above_camera, float2 point, float2 source_dimensions, float pixel_scale,
    float shade_gate, const device AnnotationSample *samples, const device uchar4 *numbers,
    AnnotationTextAtlas number_atlas, const device packed_float2 *points, Tap tap,
    Cursor cursor) {
  float drawn_pixel = max(pixel_scale, 1e-4);
  float feather = drawn_pixel * 0.5;
  bool annotated = count > 0u && all(source_dimensions > 0.0);
  if (annotated) {
    uint shade_at = count;
    for (uint index = 0; index < count; ++index)
      if (annotations[index].above_camera == above_camera &&
          annotations[index].kind == annotation_spotlight_kind)
        shade_at = index;
    for (uint index = 0; index < count; ++index) {
      const device AnnotationUniforms &annotation = annotations[index];
      if (annotation.above_camera != above_camera ||
          !annotation_acts_on_picture(annotation.kind))
        continue;
      if (index == shade_at) {
        float shade = annotation_spotlight_cover(annotations, count, above_camera, point,
                                                 pixel_scale, false);
        rgba.rgb *= 1.0 - annotation_spotlight_dim * shade * shade_gate;
      }
      rgba = annotation_layer(rgba, annotation, point, feather, samples, numbers,
                              number_atlas, points, tap, cursor);
    }
    for (uint index = 0; index < count; ++index) {
      const device AnnotationUniforms &annotation = annotations[index];
      if (annotation.above_camera != above_camera ||
          annotation_acts_on_picture(annotation.kind))
        continue;
      rgba = annotation.kind == annotation_highlight_kind
          ? annotation_highlight_layer(rgba, rgba, annotation, point, feather, points, samples)
          : annotation_layer(rgba, annotation, point, feather, samples, numbers, number_atlas,
                             points, tap, cursor);
    }
  }
  float4 pointer = annotation_cursor_seen(cursor, point, drawn_pixel);
  if (pointer.a <= 0.0) return rgba;
  if (annotated) {
    pointer.rgb *= 1.0 - annotation_spotlight_dim *
                             annotation_spotlight_cover(annotations, count, above_camera,
                                                        point, pixel_scale, false);
    float hidden = 0.0;
    for (uint index = 0; index < count; ++index) {
      const device AnnotationUniforms &annotation = annotations[index];
      if (annotation.above_camera == above_camera &&
          annotation.kind == annotation_magnify_kind)
        hidden = max(hidden, annotation_magnify_cover(annotation, samples, point, feather));
    }
    pointer.a *= 1.0 - hidden;
  }
  return mix(rgba, pointer, pointer.a);
}
)METAL"
