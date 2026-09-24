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

/// How much of a pixel lies inside an edge `distance` canvas pixels away,
/// where `feather` is half a drawn pixel. A linear ramp across that one pixel
/// is the area a straight edge actually covers; `smoothstep` is half again as
/// steep at the edge and leaves visible steps on a 1x display.
static float annotation_edge(float distance, float feather) {
  return saturate(0.5 - distance / (2.0 * feather));
}

/// Coverage of one prepared arrow, feathered over `feather` canvas pixels.
static float annotation_coverage(
    float2 point, const device AnnotationArrowGeometry &arrow, float feather) {
  float2 distances = annotation_arrow_distance(point, arrow);
  return max(annotation_edge(distances.x, feather), annotation_edge(distances.y, feather));
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

/// How many atlas pixels the type is rasterised at per canvas pixel.
constant float annotation_type_supersample = 2.0;

static float annotation_atlas_texel(const device uchar4 *pixels, uint2 atlas, int2 at) {
  uint2 clamped = uint2(clamp(at, int2(0), int2(atlas) - 1));
  return float(pixels[clamped.y * atlas.x + clamped.x].a) / 255.0;
}

/// The atlas's coverage at `position`, in atlas pixels, blended between the
/// four nearest texels.
static float annotation_atlas_bilinear(
    const device uchar4 *pixels, uint2 atlas, float2 position) {
  float2 at = position - 0.5;
  float2 base = floor(at);
  float2 fraction = at - base;
  int2 low = int2(base);
  float a = annotation_atlas_texel(pixels, atlas, low);
  float b = annotation_atlas_texel(pixels, atlas, low + int2(1, 0));
  float c = annotation_atlas_texel(pixels, atlas, low + int2(0, 1));
  float d = annotation_atlas_texel(pixels, atlas, low + int2(1, 1));
  return mix(mix(a, b, fraction.x), mix(c, d, fraction.x), fraction.y);
}

/// How much of the type in one atlas cell covers a drawn pixel centred on
/// `texel`, both in atlas pixels. A drawn pixel spans `feather * 2` canvas
/// pixels, so the whole of that footprint is averaged: a single sample would
/// pick one atlas pixel out of many where the canvas is shown smaller than
/// its resolution, and the type's edges would come out stepped. The samples
/// stay inside the cell so a neighbour's type never bleeds in. The twin of
/// `annotation_atlas_coverage` in `annotations.hlsl`.
static float annotation_atlas_coverage(
    const device uchar4 *pixels, uint2 atlas, float2 texel, float feather,
    float2 cell_origin, float2 cell_size) {
  float span = max(feather * 2.0 * annotation_type_supersample, 1e-3);
  // Samples no more than an atlas pixel apart, and never fewer than two per
  // axis: one bilinear sample only averages the footprint when it lands on a
  // texel corner, and type set off the pixel grid would otherwise come out
  // stepped. Capped for a canvas shown very small.
  uint taps = clamp(uint(ceil(span)), 2u, 6u);
  float step = span / float(taps);
  float2 first = texel - span * 0.5 + step * 0.5;
  float2 low = cell_origin + 0.5;
  float2 high = max(cell_origin + cell_size - 0.5, low);
  float total = 0.0;
  for (uint row = 0u; row < taps; ++row) {
    for (uint column = 0u; column < taps; ++column) {
      float2 position = clamp(first + float2(float(column), float(row)) * step, low, high);
      total += annotation_atlas_bilinear(pixels, atlas, position);
    }
  }
  return total / float(taps * taps);
}

)METAL"
