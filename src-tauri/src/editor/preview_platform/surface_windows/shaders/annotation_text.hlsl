// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The text box's own half of the annotation pass, the HLSL twin of
// `gpu_compositor_macos_shader_source_annotation_text.h`: the box and its
// pointer read out of the slots an arrow fills with its curve, the distance to
// them, its type sampled from the atlas, and the layer that draws them.
// Included by `annotations.hlsl`, after the pieces every kind shares.

/// A text box read out of the slots an arrow fills with its curve. Prepared
/// once per annotation by Rust's `text::geometry::prepare_text`, which
/// documents every slot.
struct PreviewTextBox {
  float2 low, high;
  float radius;
  float2 base, tip;
  float base_radius, tip_radius, blend;
  float2 text_origin, text_size, text_centre;
};

PreviewTextBox annotation_text_box(PreviewGeometry prepared) {
  PreviewTextBox box;
  box.low = float2(prepared.ax, prepared.ay);
  box.high = float2(prepared.bx, prepared.by);
  box.radius = prepared.rounding;
  box.base = float2(prepared.end_head_ax, prepared.end_head_ay);
  box.tip = float2(prepared.end_head_bx, prepared.end_head_by);
  box.base_radius = prepared.high;
  box.tip_radius = prepared.low;
  box.blend = prepared.end_head_cx;
  box.text_origin = float2(prepared.start_head_ax, prepared.start_head_ay);
  box.text_size = float2(prepared.start_head_bx, prepared.start_head_by);
  box.text_centre = float2(prepared.start_head_cx, prepared.start_head_cy);
  return box;
}

float annotation_rounded_box_distance(float2 probe, float2 low, float2 high, float radius) {
  float2 half_size = (high - low) * 0.5;
  float2 q = abs(probe - (low + high) * 0.5) - half_size + radius;
  return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - radius;
}

/// A stroke tapering from a circle at `from` to a smaller one at `to`, its
/// sides tangent to both. The per-pixel twin of `tapered_distance` in
/// `annotations/text/geometry.rs`.
float annotation_tapered_distance(
    float2 probe, float2 from, float2 to, float from_radius, float to_radius) {
  float2 axis = to - from;
  float span = dot(axis, axis);
  float taper = from_radius - to_radius;
  float2 local = probe - from;
  if (span <= taper * taper)
    return min(length(local) - from_radius, length(probe - to) - to_radius);
  float2 q = float2(abs(local.x * axis.y - local.y * axis.x), dot(local, axis)) / span;
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
float annotation_text_distance(float2 probe, PreviewTextBox box) {
  float body = annotation_rounded_box_distance(probe, box.low, box.high, box.radius);
  if (box.base_radius <= 0.0) return body;
  float pointer = annotation_tapered_distance(probe, box.base, box.tip,
                                              box.base_radius, box.tip_radius);
  if (box.blend <= 0.0) return min(body, pointer);
  float mix_amount = max(box.blend - abs(body - pointer), 0.0) / box.blend;
  return min(body, pointer) - mix_amount * mix_amount * box.blend * 0.25;
}

/// How much of the type covers this pixel, from the atlas it was rasterised
/// into at `atlas.scale` pixels to the canvas pixel, centred on the text
/// block. `feather` is half a drawn pixel, in canvas pixels, which is how far
/// the atlas read spreads.
float annotation_text_coverage(
    float2 probe, PreviewTextBox box, AnnotationTextAtlas atlas, float feather) {
  if (atlas.size.x == 0u || atlas.size.y == 0u || atlas.scale <= 0.0 ||
      box.text_size.x <= 0.0 || box.text_size.y <= 0.0)
    return 0.0;
  float2 drawn = box.text_size / atlas.scale;
  float2 local = probe - box.text_centre + drawn * 0.5;
  if (any(local < 0.0) || any(local > drawn)) return 0.0;
  return annotation_atlas_coverage(atlas, box.text_origin, box.text_size,
                                   box.text_origin + local * atlas.scale,
                                   2.0 * feather * atlas.scale);
}

/// A text box averaged over its exposure samples, as a counter is while it
/// grows into place.
float annotation_text_exposure(float2 probe, PreviewArrow annotation, float feather) {
  float total = 0.0;
  for (uint tap = 0u; tap < annotation.sample_count; ++tap) {
    PreviewSample sample = annotation_samples[annotation.sample_first + tap];
    float distance = annotation_text_distance(probe, annotation_text_box(sample.geometry));
    total += annotation_edge(distance, feather) * sample.opacity;
  }
  return total / (float)annotation.sample_count;
}

/// Grows `low` and `high` to hold a text box and its pointer, `reach` beyond.
void annotation_text_bounds(
    PreviewTextBox box, float reach, inout float2 low, inout float2 high) {
  low = min(low, box.low - reach);
  high = max(high, box.high + reach);
  if (box.base_radius > 0.0) {
    low = min(low, box.tip - box.tip_radius - reach);
    high = max(high, box.tip + box.tip_radius + reach);
  }
}

/// One text box: its box and pointer in the annotation's own colour, and its
/// type in whichever of black or white reads on that colour, clipped to the
/// box's own coverage so the two share one antialiased edge.
float4 annotation_text_layer(
    float4 rgba, PreviewArrow annotation, float4 color, float2 canvas_point,
    float feather, float halo, AnnotationTextAtlas atlas) {
  PreviewTextBox box = annotation_text_box(annotation.geometry);
  float reach = halo + feather + 1.0;
  float2 low = float2(1e20, 1e20);
  float2 high = float2(-1e20, -1e20);
  annotation_text_bounds(box, reach, low, high);
  if (annotation.sample_count > 0u) {
    // The exposure samples run steadily from last frame's reveal to this
    // one's, so the first and last hold every one between: a pointer drawing
    // back in trails past where its tip is now.
    uint last = annotation.sample_first + annotation.sample_count - 1u;
    annotation_text_bounds(annotation_text_box(annotation_samples[annotation.sample_first].geometry),
                           reach, low, high);
    annotation_text_bounds(annotation_text_box(annotation_samples[last].geometry), reach, low,
                           high);
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
      : annotation_text_exposure(canvas_point, annotation, feather);
  if (coverage <= 0.0) return rgba;
  float alpha = coverage * color.a;
  rgba.rgb = color.rgb * alpha + rgba.rgb * (1.0 - alpha);
  rgba.a = alpha + rgba.a * (1.0 - alpha);
  float ink = annotation_text_coverage(canvas_point, box, atlas, feather) * alpha;
  if (ink <= 0.0) return rgba;
  // The counter's rule, so a text box and a counter in one colour carry the
  // same ink.
  float luminance = dot(color.rgb, float3(0.2126, 0.7152, 0.0722));
  float3 tint = luminance > 0.6 ? float3(0.0, 0.0, 0.0) : float3(1.0, 1.0, 1.0);
  rgba.rgb = tint * ink + rgba.rgb * (1.0 - ink);
  rgba.a = ink + rgba.a * (1.0 - ink);
  return rgba;
}
