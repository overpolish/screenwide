// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#define GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATIONS @R"METAL(
struct AnnotationTriangle {
  packed_float2 a, b, c;
};
struct AnnotationArrowGeometry {
  packed_float2 a, b, c;
  float width, low, high;
  AnnotationTriangle start_head, end_head;
  float rounding;
  uint head;
};
struct AnnotationUniforms {
  uint kind, above_camera;
  packed_float4 color;
  float hover;
  AnnotationArrowGeometry arrow;
};
static_assert(sizeof(AnnotationUniforms) == 120,
              "Prepared annotations must match their native layout");

constant float annotation_hover_alpha = 0.24;

static float annotation_triangle_distance(
    float2 point, float2 a, float2 b, float2 c) {
  float2 edge_ab = b - a, edge_bc = c - b, edge_ca = a - c;
  float2 to_a = point - a, to_b = point - b, to_c = point - c;
  float2 nearest_ab = to_a - edge_ab *
      clamp(dot(to_a, edge_ab) / max(dot(edge_ab, edge_ab), 1e-6), 0.0, 1.0);
  float2 nearest_bc = to_b - edge_bc *
      clamp(dot(to_b, edge_bc) / max(dot(edge_bc, edge_bc), 1e-6), 0.0, 1.0);
  float2 nearest_ca = to_c - edge_ca *
      clamp(dot(to_c, edge_ca) / max(dot(edge_ca, edge_ca), 1e-6), 0.0, 1.0);
  float winding = sign(edge_ab.x * edge_ca.y - edge_ab.y * edge_ca.x);
  float2 distances = min(
      min(float2(dot(nearest_ab, nearest_ab),
                 winding * (to_a.x * edge_ab.y - to_a.y * edge_ab.x)),
          float2(dot(nearest_bc, nearest_bc),
                 winding * (to_b.x * edge_bc.y - to_b.y * edge_bc.x))),
      float2(dot(nearest_ca, nearest_ca),
             winding * (to_c.x * edge_ca.y - to_c.y * edge_ca.x)));
  return -sqrt(distances.x) * sign(distances.y);
}

/// Head vertices and shaft limits were prepared once before this dispatch.
/// The shader only evaluates distances; picking uses the same prepared heads.
static float annotation_arrow_distance(
    float2 point, const device AnnotationArrowGeometry &arrow) {
  float result = annotation_curve_distance(point, float2(arrow.a), float2(arrow.b),
      float2(arrow.c), arrow.low, arrow.high) - arrow.width * 0.5;
  if (arrow.head != 0u)
    result = min(result, annotation_triangle_distance(point, float2(arrow.end_head.a),
        float2(arrow.end_head.b), float2(arrow.end_head.c)) - arrow.rounding);
  if (arrow.head == 2u)
    result = min(result, annotation_triangle_distance(point, float2(arrow.start_head.a),
        float2(arrow.start_head.b), float2(arrow.start_head.c)) - arrow.rounding);
  return result;
}

/// Draws every mark whose layer matches `above_camera`.
///
/// Geometry is already in canvas pixels, prepared using the current image
/// placement before dispatch. Marks are deliberately not
/// clipped to the crop: an arrow may point in from the padding.
///
/// `pixel_scale` is how many canvas pixels one drawn pixel covers. Every edge
/// is feathered over that, not over one canvas pixel: a workspace layer drawn
/// smaller than its canvas would otherwise take its whole antialiasing band
/// from inside a single drawn pixel and come out jagged.
static float4 composite_annotations(
    float4 rgba, const device AnnotationUniforms *annotations, uint count,
    uint above_camera, float2 canvas_point, constant CanvasUniforms &u,
    float2 source_dimensions, float pixel_scale) {
  if (count == 0u || any(source_dimensions <= 0.0)) return rgba;
  float feather = max(pixel_scale, 1e-4) * 0.5;
  for (uint index = 0; index < count; ++index) {
    const device AnnotationUniforms &mark = annotations[index];
    if (mark.kind != 0u || mark.above_camera != above_camera) continue;
    float4 color = float4(mark.color);
    if (color.a <= 0.0 || mark.arrow.width <= 0.0) continue;
    float2 a = float2(mark.arrow.a);
    float2 b = float2(mark.arrow.b);
    float2 c = float2(mark.arrow.c);
    // The curve lies inside the hull of its three points; the heads reach
    // `head_length` back from a tip and `half_base` across. One rectangle
    // covers all of it, and keeps a long list cheap for most of the canvas.
    float halo = max(mark.hover, 0.0);
    float radius = mark.arrow.width * 0.5;
    float reach = radius * (mark.arrow.head != 0u
        ? 9.0 : 1.0) + halo + feather + 1.0;
    if (any(canvas_point < min(a, min(b, c)) - reach) ||
        any(canvas_point > max(a, max(b, c)) + reach))
      continue;
    float distance = annotation_arrow_distance(
        canvas_point, mark.arrow);
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
    float coverage = 1.0 - smoothstep(-feather, feather, distance);
    if (coverage <= 0.0) continue;
    float alpha = coverage * color.a;
    rgba.rgb = color.rgb * alpha + rgba.rgb * (1.0 - alpha);
    rgba.a = alpha + rgba.a * (1.0 - alpha);
  }
  return rgba;
}
)METAL"
