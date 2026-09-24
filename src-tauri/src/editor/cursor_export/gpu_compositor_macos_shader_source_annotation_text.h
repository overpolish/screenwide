// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

/// The text box's own half of the annotation shader: the box and its pointer
/// read out of the slots an arrow fills with its curve, the distance to them,
/// its text sampled from the atlas, and the layer that draws them.
#define GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_TEXT @R"METAL(
/// A text box read out of the slots an arrow fills with its curve. Prepared
/// once per annotation by Rust's `prepare_text`, over
/// `screenwide_annotation_prepare`, which documents every slot.
struct AnnotationTextBox {
  float2 low, high;
  float radius;
  float2 base, tip;
  float base_radius, tip_radius, blend;
  float2 text_origin, text_size, text_centre;
};

static AnnotationTextBox annotation_text_box(
    const device AnnotationArrowGeometry &prepared) {
  AnnotationTextBox box;
  box.low = float2(prepared.a);
  box.high = float2(prepared.b);
  box.radius = prepared.rounding;
  box.base = float2(prepared.end_head.a);
  box.tip = float2(prepared.end_head.b);
  box.base_radius = prepared.high;
  box.tip_radius = prepared.low;
  box.blend = float2(prepared.end_head.c).x;
  box.text_origin = float2(prepared.start_head.a);
  box.text_size = float2(prepared.start_head.b);
  box.text_centre = float2(prepared.start_head.c);
  return box;
}

static float annotation_rounded_box_distance(
    float2 point, float2 low, float2 high, float radius) {
  float2 half_size = (high - low) * 0.5;
  float2 q = abs(point - (low + high) * 0.5) - half_size + radius;
  return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - radius;
}

/// A stroke tapering from a circle at `from` to a smaller one at `to`, its
/// sides tangent to both. The per-pixel twin of `tapered_distance` in
/// `annotations/text/geometry.rs`.
static float annotation_tapered_distance(
    float2 point, float2 from, float2 to, float from_radius, float to_radius) {
  float2 axis = to - from;
  float span = dot(axis, axis);
  float taper = from_radius - to_radius;
  float2 local = point - from;
  if (span <= taper * taper)
    return min(length(local) - from_radius, length(point - to) - to_radius);
  float2 q = float2(abs(local.x * axis.y - local.y * axis.x),
                    dot(local, axis)) / span;
  float2 side = float2(sqrt(span - taper * taper), taper);
  float cross_value = side.x * q.y - side.y * q.x;
  if (cross_value < 0.0) return sqrt(span * dot(q, q)) - from_radius;
  if (cross_value > side.x)
    return sqrt(span * (dot(q, q) + 1.0 - 2.0 * q.y)) - to_radius;
  return dot(side, q) - from_radius;
}

/// How far a point falls outside a text box, pointer included: the box and
/// the pointer blended over `blend`, so the pointer leaves the edge in a
/// curve. The per-pixel twin of `text_distance`.
static float annotation_text_distance(float2 point, AnnotationTextBox box) {
  float body = annotation_rounded_box_distance(point, box.low, box.high, box.radius);
  if (box.base_radius <= 0.0) return body;
  float pointer = annotation_tapered_distance(point, box.base, box.tip,
                                              box.base_radius, box.tip_radius);
  if (box.blend <= 0.0) return min(body, pointer);
  float mix_amount = max(box.blend - abs(body - pointer), 0.0) / box.blend;
  return min(body, pointer) - mix_amount * mix_amount * box.blend * 0.25;
}

/// How much of the text covers this pixel, from the atlas it was rasterised
/// into, centred on the box.
static float annotation_text_coverage(
    float2 point, AnnotationTextBox box, float feather,
    const device uchar4 *atlas_pixels, uint2 atlas) {
  if (atlas.x == 0u || atlas.y == 0u || box.text_size.x <= 0.0 ||
      box.text_size.y <= 0.0)
    return 0.0;
  float2 drawn = box.text_size / annotation_type_supersample;
  float2 local = point - box.text_centre + drawn * 0.5;
  if (any(local < 0.0) || any(local > drawn)) return 0.0;
  float2 texel = box.text_origin + local * annotation_type_supersample;
  return annotation_atlas_coverage(atlas_pixels, atlas, texel, feather,
                                   box.text_origin, box.text_size);
}

/// A text box averaged over its exposure samples, as a counter is while it
/// grows into place.
static float annotation_text_exposure(
    float2 point, const device AnnotationUniforms &annotation,
    const device AnnotationSample *samples, float feather) {
  float total = 0.0;
  for (uint tap = 0; tap < annotation.sample_count; ++tap) {
    const device AnnotationSample &sample = samples[annotation.sample_offset + tap];
    float distance = annotation_text_distance(point, annotation_text_box(sample.arrow));
    total += annotation_edge(distance, feather) * sample.opacity;
  }
  return total / float(annotation.sample_count);
}

/// One text box: its box and pointer in the annotation's own colour, and its
/// text in whichever of black or white reads on that colour, clipped to the
/// box's own coverage so the two share one antialiased edge.
static float4 annotation_text_layer(
    float4 rgba, const device AnnotationUniforms &annotation, float4 color,
    float2 canvas_point, float feather, float halo,
    const device AnnotationSample *samples, const device uchar4 *atlas_pixels,
    uint2 atlas) {
  AnnotationTextBox box = annotation_text_box(annotation.arrow);
  float reach = halo + feather + 1.0;
  float2 low = box.low - reach;
  float2 high = box.high + reach;
  if (box.base_radius > 0.0) {
    low = min(low, box.tip - box.tip_radius - reach);
    high = max(high, box.tip + box.tip_radius + reach);
  }
  if (any(canvas_point < low) || any(canvas_point > high)) return rgba;
  float distance = annotation_text_distance(canvas_point, box);
  if (halo > 0.0) {
    float band = (1.0 - annotation_edge(distance, feather)) *
        annotation_edge(distance - halo, feather);
    float alpha = band * color.a * annotation_hover_alpha;
    if (alpha > 0.0) {
      rgba.rgb = color.rgb * alpha + rgba.rgb * (1.0 - alpha);
      rgba.a = alpha + rgba.a * (1.0 - alpha);
    }
  }
  float coverage = annotation.sample_count == 0u
      ? annotation_edge(distance, feather)
      : annotation_text_exposure(canvas_point, annotation, samples, feather);
  if (coverage <= 0.0) return rgba;
  float alpha = coverage * color.a;
  rgba.rgb = color.rgb * alpha + rgba.rgb * (1.0 - alpha);
  rgba.a = alpha + rgba.a * (1.0 - alpha);
  float ink = annotation_text_coverage(canvas_point, box, feather, atlas_pixels, atlas) * alpha;
  if (ink <= 0.0) return rgba;
  // The counter's rule, so a text box and a counter in one colour carry the
  // same ink.
  float luminance = dot(color.rgb, float3(0.2126, 0.7152, 0.0722));
  float3 tint = luminance > 0.6 ? float3(0.0) : float3(1.0);
  rgba.rgb = tint * ink + rgba.rgb * (1.0 - ink);
  rgba.a = ink + rgba.a * (1.0 - ink);
  return rgba;
}

)METAL"
