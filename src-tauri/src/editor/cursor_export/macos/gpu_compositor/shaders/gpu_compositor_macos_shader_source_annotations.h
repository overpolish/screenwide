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

/// Where the annotations' type was rasterised: the atlas's size in pixels and
/// how many atlas pixels it holds per canvas pixel. The twin of
/// `ScreenwideAnnotationTextUniforms`.
struct AnnotationTextAtlas {
  uint width, height;
  float scale;
};

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

/// The atlas's coverage at `texel`, bilinearly filtered and held to the
/// cell from `low` to `high`, so a tap at the cell's edge reads its own
/// transparent margin rather than the next cell over.
static float annotation_atlas_bilinear(
    const device uchar4 *pixels, uint width, float2 texel, float2 low, float2 high) {
  float2 base = texel - 0.5;
  float2 first = floor(base);
  float2 blend = base - first;
  uint2 from = uint2(clamp(first, low, high));
  uint2 to = uint2(clamp(first + 1.0, low, high));
  float top = mix(float(pixels[from.y * width + from.x].a),
                  float(pixels[from.y * width + to.x].a), blend.x);
  float bottom = mix(float(pixels[to.y * width + from.x].a),
                     float(pixels[to.y * width + to.x].a), blend.x);
  return mix(top, bottom, blend.y) / 255.0;
}

/// How much of the type in the atlas cell at `origin`, `size` atlas pixels
/// across, covers one drawn pixel centred on `texel` and spanning `footprint`
/// atlas pixels: four filtered taps spread over that span. The atlas holds
/// two to four atlas pixels per drawn pixel, so the taps take in the whole
/// pixel whether the canvas is drawn larger or smaller than its resolution.
static float annotation_atlas_coverage(
    const device uchar4 *pixels, AnnotationTextAtlas atlas, float2 origin, float2 size,
    float2 texel, float footprint) {
  float2 low = origin;
  float2 high = min(origin + size, float2(atlas.width, atlas.height)) - 1.0;
  float spread = footprint * 0.25;
  float total = 0.0;
  for (uint tap = 0u; tap < 4u; ++tap) {
    float2 offset = float2(tap & 1u ? spread : -spread, tap >> 1u ? spread : -spread);
    total += annotation_atlas_bilinear(pixels, atlas.width, texel + offset, low, high);
  }
  return total * 0.25;
}

)METAL"
