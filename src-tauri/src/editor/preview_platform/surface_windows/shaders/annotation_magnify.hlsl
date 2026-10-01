// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The magnifier: a loupe showing a zoom area enlarged, the line joining the
// two, the loupe's rim and its shadow. The HLSL twin of
// `gpu_compositor_macos_shader_source_annotation_magnify.h`; every number it
// reads was prepared by `magnify::geometry::prepare_magnify`, which documents
// where the record keeps each part. Included by `annotations.hlsl`, after the
// pieces every kind shares.
//
// The loupe reads the picture through the two functions declared below, which
// each shader that draws annotations defines over its own source: the
// preview's is its source texture after the redactions and the spotlights'
// blur were applied to it, and the live overlay's, which offers no magnifier,
// the desktop its highlights recolour. Highlights read them too, to find each
// glyph's full ink. Each texel is drawn as a crisp square, with only the one
// drawn pixel across its edge blended. The cursor lies on the picture, under
// the loupe, so the loupe shows it too, enlarged, through
// `annotation_cursor_seen`.

static const uint annotation_magnify_kind = 8u;
// The flag a loupe that casts a shadow carries: `flags::SHADOW`.
static const uint annotation_magnify_shadow_flag = 1u << 7;

// Where the source lies on the canvas: the placed image's rectangle, the
// texels the canvas shows of it, first and last inclusive, and its size.
struct AnnotationMagnifyPlacement {
  float4 image;
  float4 texels;
  float2 size;
};

AnnotationMagnifyPlacement annotation_magnify_placement();
float4 annotation_magnify_fetch(int2 texel);

// The picture at canvas point `q`, where one drawn pixel spans `span` canvas
// pixels there.
float4 annotation_magnify_sample(float2 q, float2 span) {
  AnnotationMagnifyPlacement at = annotation_magnify_placement();
  if (any(at.image.zw <= 0.0) || any(at.size < 1.0)) return 0.0;
  float2 per = at.size / at.image.zw;
  float2 centred = (q - at.image.xy) * per - 0.5;
  float2 base = floor(centred);
  // Enlarged, a texel spans many drawn pixels, and only the one across its
  // edge is blended; reduced, this is plain bilinear filtering.
  float2 footprint = min(max(span * per, 1e-4), 1.0);
  float2 blend = saturate((centred - base - 0.5) / footprint + 0.5);
  // Past what the canvas shows, the edge the crop left carries on.
  int2 first = int2(clamp(base, at.texels.xy, at.texels.zw));
  int2 second = int2(clamp(base + 1.0, at.texels.xy, at.texels.zw));
  float4 top = lerp(annotation_magnify_fetch(first),
                    annotation_magnify_fetch(int2(second.x, first.y)), blend.x);
  float4 bottom = lerp(annotation_magnify_fetch(int2(first.x, second.y)),
                       annotation_magnify_fetch(second), blend.x);
  return lerp(top, bottom, blend.y);
}

// A rounded box's signed distance, by its centre and half size.
float annotation_magnify_box(float2 probe, float2 centre, float2 half_size, float radius) {
  float rounding = min(radius, min(half_size.x, half_size.y));
  float2 q = abs(probe - centre) - (half_size - rounding);
  return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - rounding;
}

float annotation_magnify_segment(float2 probe, float2 a, float2 b) {
  float2 along = b - a;
  float t = saturate(dot(probe - a, along) / max(dot(along, along), 1e-6));
  return distance(probe, a + along * t);
}

float4 annotation_magnify_over(float4 rgba, float3 color, float alpha) {
  if (alpha <= 0.0) return rgba;
  rgba.rgb = color * alpha + rgba.rgb * (1.0 - alpha);
  rgba.a = alpha + rgba.a * (1.0 - alpha);
  return rgba;
}

// Draws one magnifier, as `shape` places it and in `color`, over `rgba`: the
// loupe's shadow, the zoom area's outline and the line to the loupe in a pen
// half the rim's, the picture the zoom area covers enlarged into the loupe -
// and the cursor, which lies on the picture, where `cursor` says the layer
// carries it - and the rim. The picture is solid throughout: a loupe setting
// off from its zoom area shows exactly what that covers, at its own size, so
// it lifts off the picture rather than fading in over it. The shadow is a
// clip's, and comes with the magnifier's presence; the rest comes with its
// colour, which carries the presence already. The hover halo hugs the zoom
// area, the box the chrome holds.
float4 annotation_magnify_draw(float4 rgba, PreviewGeometry shape, float4 color, uint flags,
                               float2 probe, float feather, float halo, bool cursor) {
  float2 centre = float2(shape.ax, shape.ay);
  float2 half_size = float2(shape.bx, shape.by);
  float2 area = float2(shape.cx, shape.cy);
  float2 area_half = float2(shape.start_head_cx, shape.start_head_cy);
  if (any(half_size <= 0.0) || any(area_half <= 0.0)) return rgba;
  float presence = saturate(shape.low);
  float stroke = max(shape.width, 0.0);
  float pen = max(stroke * 0.5, feather * 2.0);
  bool shadowed = (flags & annotation_magnify_shadow_flag) != 0u;
  // A clip's own shadow: the same spread for its size, lifted the same way.
  float sigma = clamp(min(half_size.x, half_size.y) * 0.11, 2.0, 110.0);
  float lift = sigma * 0.35;
  float reach = stroke + feather * 2.0 + (shadowed ? sigma * 4.0 + lift : 0.0);
  bool near_loupe = all(abs(probe - centre) <= half_size + reach);
  bool near_area = all(abs(probe - area) <= area_half + pen + halo + feather * 2.0);
  float2 from = float2(shape.start_head_ax, shape.start_head_ay);
  float2 to = float2(shape.start_head_bx, shape.start_head_by);
  bool near_line = shape.head != 0u && all(probe >= min(from, to) - pen - feather * 2.0) &&
                   all(probe <= max(from, to) + pen + feather * 2.0);
  if (!near_loupe && !near_area && !near_line) return rgba;
  float loupe = annotation_magnify_box(probe, centre, half_size, shape.rounding);
  float inside = annotation_edge(loupe, feather);
  if (shadowed && near_loupe) {
    float fall = max(annotation_magnify_box(probe - float2(0.0, lift), centre, half_size,
                                            shape.rounding), 0.0);
    float shadow = 0.14 * presence * exp(-0.5 * fall * fall / (sigma * sigma));
    rgba.rgb *= 1.0 - shadow * (1.0 - inside);
  }
  float outline = 1e20;
  if (near_area) outline = annotation_magnify_box(probe, area, area_half, shape.high);
  if (halo > 0.0 && near_area) {
    float outer = outline - pen * 0.5;
    float band = (1.0 - annotation_edge(outer, feather)) * annotation_edge(outer - halo, feather);
    rgba = annotation_magnify_over(rgba, color.rgb, band * color.a * annotation_hover_alpha);
  }
  float lines = abs(outline) - pen * 0.5;
  if (near_line) lines = min(lines, annotation_magnify_segment(probe, from, to) - pen * 0.5);
  rgba = annotation_magnify_over(rgba, color.rgb, annotation_edge(lines, feather) * color.a);
  if (inside > 0.0) {
    float2 shrink = area_half / half_size;
    float2 seen_at = area + (probe - centre) * shrink;
    float4 picture = annotation_magnify_sample(seen_at, shrink * feather * 2.0);
    rgba = annotation_magnify_over(rgba, picture.rgb, inside * picture.a);
    if (cursor) {
      float4 pointer = annotation_cursor_seen(seen_at, feather * 2.0 * min(shrink.x, shrink.y));
      rgba = annotation_magnify_over(rgba, pointer.rgb, inside * pointer.a);
    }
  }
  if (stroke > 0.0 && near_loupe)
    rgba = annotation_magnify_over(
        rgba, color.rgb, annotation_edge(abs(loupe) - stroke * 0.5, feather) * color.a);
  return rgba;
}

// Draws one magnifier over `rgba`. One that moved while the shutter was open
// is drawn whole at every exposure sample, each at the opacity it had then,
// and the results averaged: what an open shutter records of a loupe flying
// out of its zoom area, picture and all.
float4 annotation_magnify_layer(float4 rgba, PreviewArrow magnifier, float2 probe,
                                float feather, float halo, bool cursor) {
  float4 color = float4(magnifier.red, magnifier.green, magnifier.blue, magnifier.alpha);
  if (magnifier.sample_count == 0u)
    return annotation_magnify_draw(rgba, magnifier.geometry, color, magnifier.flags, probe,
                                   feather, halo, cursor);
  float4 total = 0.0;
  [loop] for (uint tap = 0u; tap < magnifier.sample_count; ++tap) {
    PreviewSample sample = annotation_samples[magnifier.sample_first + tap];
    total += annotation_magnify_draw(rgba, sample.geometry,
                                     float4(color.rgb, color.a * sample.opacity),
                                     magnifier.flags, probe, feather, halo, cursor);
  }
  return total / (float)magnifier.sample_count;
}

// How much of `probe` one prepared loupe covers.
float annotation_magnify_inside(PreviewGeometry shape, float2 probe, float feather) {
  float2 centre = float2(shape.ax, shape.ay);
  float2 half_size = float2(shape.bx, shape.by);
  if (any(half_size <= 0.0) || shape.start_head_cx <= 0.0 || shape.start_head_cy <= 0.0 ||
      any(abs(probe - centre) > half_size + feather * 2.0))
    return 0.0;
  return annotation_edge(annotation_magnify_box(probe, centre, half_size, shape.rounding),
                         feather);
}

// How much of `probe` a magnifier's loupe covers, over its exposure as
// `annotation_magnify_layer` draws it: what a cursor lying under the loupe is
// hidden by.
float annotation_magnify_cover(PreviewArrow magnifier, float2 probe, float feather) {
  if (magnifier.sample_count == 0u)
    return annotation_magnify_inside(magnifier.geometry, probe, feather);
  float total = 0.0;
  [loop] for (uint tap = 0u; tap < magnifier.sample_count; ++tap)
    total += annotation_magnify_inside(annotation_samples[magnifier.sample_first + tap].geometry,
                                       probe, feather);
  return total / (float)magnifier.sample_count;
}
