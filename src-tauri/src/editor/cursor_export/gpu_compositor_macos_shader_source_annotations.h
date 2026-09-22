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
struct AnnotationSample {
  AnnotationArrowGeometry arrow;
  float opacity;
};
struct AnnotationUniforms {
  uint kind, above_camera, flags;
  float params[3];
  packed_float4 color;
  float hover;
  AnnotationArrowGeometry arrow;
  uint sample_offset, sample_count;
  uint data_offset, data_count;
};
static_assert(sizeof(AnnotationUniforms) == 152,
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
static float2 annotation_arrow_distance(
    float2 point, const device AnnotationArrowGeometry &arrow) {
  // An empty window is an annotation that has not started, or one whose head
  // has eaten what was left of its shaft. Either way there is no shaft to draw.
  float2 result = float2(arrow.high > arrow.low
      ? annotation_curve_distance(point, float2(arrow.a), float2(arrow.b),
            float2(arrow.c), arrow.low, arrow.high) - arrow.width * 0.5
      : 1e20, 1e20);
  if (arrow.head != 0u)
    result.y = min(result.y, annotation_triangle_distance(point, float2(arrow.end_head.a),
        float2(arrow.end_head.b), float2(arrow.end_head.c)) - arrow.rounding);
  if (arrow.head == 2u)
    result.y = min(result.y, annotation_triangle_distance(point, float2(arrow.start_head.a),
        float2(arrow.start_head.b), float2(arrow.start_head.c)) - arrow.rounding);
  return result;
}

/// Coverage of one prepared arrow, feathered over `feather` canvas pixels.
static float annotation_coverage(
    float2 point, const device AnnotationArrowGeometry &arrow, float feather) {
  float2 distances = annotation_arrow_distance(point, arrow);
  return max(1.0 - smoothstep(-feather, feather, distances.x),
             1.0 - smoothstep(-feather, feather, distances.y));
}

/// Accumulated exposure coverage: the annotation is drawn at every prepared
/// sample between the shutter start and now, so a moving shaft and its head
/// smear along the path they actually travelled while a held end stays sharp.
static float annotation_exposure(
    float2 point, const device AnnotationUniforms &annotation,
    const device AnnotationSample *samples, float feather) {
  float total = 0.0;
  for (uint tap = 0; tap < annotation.sample_count; ++tap) {
    const device AnnotationSample &sample = samples[annotation.sample_offset + tap];
    total += annotation_coverage(point, sample.arrow, feather) * sample.opacity;
  }
  return total / float(annotation.sample_count);
}

)METAL"
