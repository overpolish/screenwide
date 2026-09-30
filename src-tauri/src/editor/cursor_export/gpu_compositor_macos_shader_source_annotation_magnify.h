// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

/// The magnifier's half of the annotation shader: a loupe showing a zoom
/// area enlarged, the line joining the two, the loupe's rim and its shadow.
/// Everything is placed by Rust's `magnify::geometry`, which
/// documents where the record keeps each part; this only draws.
///
/// The loupe reads the picture through a tap each kernel builds over its own
/// source: the retained RGBA copy the canvas is drawn from, or a video
/// frame's two planes, in both cases after the redactions and the
/// spotlights' blur were applied to it. Where the tap is looked up is the
/// canvas placement's own inverse, held to the texels the canvas shows, so a
/// loupe never shows what a crop cut away. Each texel is drawn as a crisp square,
/// with only the one drawn pixel across its edge blended, which keeps an
/// enlarged capture sharp without the stair steps of plain nearest sampling.
/// The cursor lies on the picture, under the loupe, so the loupe shows it
/// too, enlarged, where the zoom area covers it. The twin of
/// `annotation_magnify.hlsl`.
#define GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_MAGNIFY @R"METAL(
constant uint annotation_magnify_kind = 8u;
constant uint annotation_magnify_shadow_flag = 1u << 7;

/// Where a kernel's source lies on its canvas: the placed image's rectangle,
/// the texels the canvas shows of it, first and last inclusive, and its size
/// in pixels.
struct AnnotationMagnifyPlacement {
  float4 image;
  float4 texels;
  float2 size;
};

/// The placement a kernel's canvas uniforms describe. What shows is where the
/// canvas's crop and the source's own crop, both in canvas pixels, overlap.
static AnnotationMagnifyPlacement annotation_magnify_placement(
    constant CanvasUniforms &u, uint2 size) {
  float4 image = float4(u.image_x, u.image_y, float(u.image_width), float(u.image_height));
  float2 dimensions = float2(size);
  float2 per = dimensions / max(image.zw, float2(1.0));
  float2 low = max(float2(u.crop_x, u.crop_y), float2(u.source_crop_x, u.source_crop_y));
  float2 high = min(float2(u.crop_x, u.crop_y) + float2(u.crop_width, u.crop_height),
                    float2(u.source_crop_x, u.source_crop_y) +
                        float2(u.source_crop_width, u.source_crop_height));
  float2 last = max(dimensions - 1.0, float2(0.0));
  float2 first_texel = clamp(floor((low - image.xy) * per + 1e-3), float2(0.0), last);
  float2 last_texel = clamp(ceil((high - image.xy) * per - 1e-3) - 1.0, first_texel, last);
  return {image, float4(first_texel, last_texel), dimensions};
}

/// A kernel with no picture to enlarge: the live overlay's, which offers no
/// magnifier. Its loupe would be empty.
struct AnnotationMagnifyNone {
  AnnotationMagnifyPlacement at;
};

static float4 annotation_magnify_fetch(AnnotationMagnifyNone, int2) {
  return float4(0.0);
}

/// A canvas drawn from a retained RGBA source.
struct AnnotationMagnifyRgba {
  AnnotationMagnifyPlacement at;
  const device uchar4 *pixels;
};

static float4 annotation_magnify_fetch(AnnotationMagnifyRgba tap, int2 texel) {
  uint width = uint(tap.at.size.x);
  return float4(tap.pixels[uint(texel.y) * width + uint(texel.x)]) / 255.0;
}

/// The picture at canvas point `q`, where one drawn pixel spans `span`
/// canvas pixels there.
template <typename Tap>
static float4 annotation_magnify_sample(Tap tap, float2 q, float2 span) {
  AnnotationMagnifyPlacement at = tap.at;
  if (any(at.image.zw <= 0.0) || any(at.size < 1.0)) return float4(0.0);
  float2 per = at.size / at.image.zw;
  float2 centred = (q - at.image.xy) * per - 0.5;
  float2 base = floor(centred);
  // Enlarged, a texel spans many drawn pixels, and only the one across its
  // edge is blended; reduced, this is plain bilinear filtering.
  float2 footprint = min(max(span * per, float2(1e-4)), float2(1.0));
  float2 blend = saturate((centred - base - 0.5) / footprint + 0.5);
  // Past what the canvas shows, the edge the crop left carries on.
  int2 first = int2(clamp(base, at.texels.xy, at.texels.zw));
  int2 second = int2(clamp(base + 1.0, at.texels.xy, at.texels.zw));
  float4 top = mix(annotation_magnify_fetch(tap, first),
                   annotation_magnify_fetch(tap, int2(second.x, first.y)), blend.x);
  float4 bottom = mix(annotation_magnify_fetch(tap, int2(first.x, second.y)),
                      annotation_magnify_fetch(tap, second), blend.x);
  return mix(top, bottom, blend.y);
}

/// A rounded box's signed distance, by its centre and half size.
static float annotation_magnify_box(float2 point, float2 centre, float2 half_size,
                                    float radius) {
  float rounding = min(radius, min(half_size.x, half_size.y));
  float2 q = abs(point - centre) - (half_size - rounding);
  return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - rounding;
}

static float annotation_magnify_segment(float2 point, float2 a, float2 b) {
  float2 along = b - a;
  float t = saturate(dot(point - a, along) / max(dot(along, along), 1e-6));
  return distance(point, a + along * t);
}

static float4 annotation_magnify_over(float4 rgba, float3 color, float alpha) {
  if (alpha <= 0.0) return rgba;
  rgba.rgb = color * alpha + rgba.rgb * (1.0 - alpha);
  rgba.a = alpha + rgba.a * (1.0 - alpha);
  return rgba;
}

/// Draws one magnifier, as `shape` places it and in `color`, over `rgba`:
/// the loupe's shadow, the zoom area's outline and the line to the loupe in a
/// pen half the rim's, the picture and the cursor the zoom area covers
/// enlarged into the loupe, and the rim. The picture is solid throughout: a
/// loupe setting off from its zoom area shows exactly what that covers, at
/// its own size, so it lifts off the picture rather than fading in over it.
/// The shadow is a clip's, and comes with the magnifier's presence; the rest
/// comes with its colour, which carries the presence already. The hover halo
/// hugs the zoom area, the box the chrome holds.
template <typename Tap, typename Cursor>
static float4 annotation_magnify_draw(
    float4 rgba, const device AnnotationArrowGeometry &shape, float4 color, uint flags,
    float2 point, float feather, float halo, Tap tap, Cursor cursor) {
  float2 centre = float2(shape.a);
  float2 half_size = float2(shape.b);
  float2 area = float2(shape.c);
  float2 area_half = float2(shape.start_head.c);
  if (any(half_size <= 0.0) || any(area_half <= 0.0)) return rgba;
  float presence = saturate(shape.low);
  float stroke = max(shape.width, 0.0);
  float pen = max(stroke * 0.5, feather * 2.0);
  bool shadowed = (flags & annotation_magnify_shadow_flag) != 0u;
  // A clip's own shadow: the same spread for its size, lifted the same way.
  float sigma = clamp(min(half_size.x, half_size.y) * 0.11, 2.0, 110.0);
  float lift = sigma * 0.35;
  float reach = stroke + feather * 2.0 + (shadowed ? sigma * 4.0 + lift : 0.0);
  bool near_loupe = all(abs(point - centre) <= half_size + reach);
  bool near_area = all(abs(point - area) <= area_half + pen + halo + feather * 2.0);
  float2 from = float2(shape.start_head.a);
  float2 to = float2(shape.start_head.b);
  bool near_line = shape.head != 0u &&
                   all(point >= min(from, to) - pen - feather * 2.0) &&
                   all(point <= max(from, to) + pen + feather * 2.0);
  if (!near_loupe && !near_area && !near_line) return rgba;
  float loupe = annotation_magnify_box(point, centre, half_size, shape.rounding);
  float inside = annotation_edge(loupe, feather);
  if (shadowed && near_loupe) {
    float fall = max(annotation_magnify_box(point - float2(0.0, lift), centre, half_size,
                                            shape.rounding), 0.0);
    float shadow = 0.14 * presence * exp(-0.5 * fall * fall / (sigma * sigma));
    rgba.rgb *= 1.0 - shadow * (1.0 - inside);
  }
  float outline = 1e20;
  if (near_area)
    outline = annotation_magnify_box(point, area, area_half, shape.high);
  if (halo > 0.0 && near_area) {
    float outer = outline - pen * 0.5;
    float band = (1.0 - annotation_edge(outer, feather)) * annotation_edge(outer - halo, feather);
    rgba = annotation_magnify_over(rgba, color.rgb, band * color.a * annotation_hover_alpha);
  }
  float lines = abs(outline) - pen * 0.5;
  if (near_line) lines = min(lines, annotation_magnify_segment(point, from, to) - pen * 0.5);
  rgba = annotation_magnify_over(rgba, color.rgb, annotation_edge(lines, feather) * color.a);
  if (inside > 0.0) {
    float2 shrink = area_half / half_size;
    float2 seen_at = area + (point - centre) * shrink;
    float4 picture = annotation_magnify_sample(tap, seen_at, shrink * feather * 2.0);
    rgba = annotation_magnify_over(rgba, picture.rgb, inside * picture.a);
    float4 pointer = annotation_cursor_seen(cursor, seen_at, feather * 2.0 * min(shrink.x, shrink.y));
    rgba = annotation_magnify_over(rgba, pointer.rgb, inside * pointer.a);
  }
  if (stroke > 0.0 && near_loupe)
    rgba = annotation_magnify_over(
        rgba, color.rgb, annotation_edge(abs(loupe) - stroke * 0.5, feather) * color.a);
  return rgba;
}

/// Draws one magnifier over `rgba`. One that moved while the shutter was
/// open is drawn whole at every exposure sample, each at the opacity it had
/// then, and the results averaged: what an open shutter records of a loupe
/// flying out of its zoom area, picture and all.
template <typename Tap, typename Cursor>
static float4 annotation_magnify_layer(
    float4 rgba, const device AnnotationUniforms &magnifier, float2 point, float feather,
    float halo, const device AnnotationSample *samples, Tap tap, Cursor cursor) {
  float4 color = float4(magnifier.color);
  if (magnifier.sample_count == 0u)
    return annotation_magnify_draw(rgba, magnifier.arrow, color, magnifier.flags, point,
                                   feather, halo, tap, cursor);
  float4 total = float4(0.0);
  for (uint index = 0; index < magnifier.sample_count; ++index) {
    const device AnnotationSample &sample = samples[magnifier.sample_offset + index];
    total += annotation_magnify_draw(rgba, sample.arrow,
                                     float4(color.rgb, color.a * sample.opacity),
                                     magnifier.flags, point, feather, halo, tap, cursor);
  }
  return total / float(magnifier.sample_count);
}

/// How much of `point` one prepared loupe covers.
static float annotation_magnify_inside(const device AnnotationArrowGeometry &shape,
                                       float2 point, float feather) {
  float2 centre = float2(shape.a);
  float2 half_size = float2(shape.b);
  if (any(half_size <= 0.0) || any(float2(shape.start_head.c) <= 0.0) ||
      any(abs(point - centre) > half_size + feather * 2.0))
    return 0.0;
  return annotation_edge(annotation_magnify_box(point, centre, half_size, shape.rounding),
                         feather);
}

/// How much of `point` a magnifier's loupe covers, over its exposure as
/// `annotation_magnify_layer` draws it: what a cursor lying under the loupe
/// is hidden by.
static float annotation_magnify_cover(const device AnnotationUniforms &magnifier,
                                      const device AnnotationSample *samples, float2 point,
                                      float feather) {
  if (magnifier.sample_count == 0u)
    return annotation_magnify_inside(magnifier.arrow, point, feather);
  float total = 0.0;
  for (uint index = 0; index < magnifier.sample_count; ++index)
    total += annotation_magnify_inside(samples[magnifier.sample_offset + index].arrow, point,
                                       feather);
  return total / float(magnifier.sample_count);
}
)METAL"
