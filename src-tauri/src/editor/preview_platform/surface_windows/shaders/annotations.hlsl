// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Arrow annotations, the HLSL twin of
// `gpu_compositor_macos_shader_source_annotation_curve.h` and
// `gpu_compositor_macos_shader_source_annotations.h`. Geometry arrives already
// prepared in canvas pixels - `annotations::arrow::geometry::prepare_arrow` on the
// Rust side is the twin of the C header Metal prepares through - so both
// backends evaluate the same distances against the same numbers.

// Every member is a scalar on purpose. HLSL refuses to straddle a vector
// across a 16-byte boundary and pads to avoid it, which would silently
// disagree with the tightly packed Rust struct; scalars pack at four bytes
// with no such rule, so every element matches its Rust twin exactly.
struct PreviewGeometry {
  float ax, ay;
  float bx, by;
  float cx, cy;
  float width, low, high;
  float start_head_ax, start_head_ay;
  float start_head_bx, start_head_by;
  float start_head_cx, start_head_cy;
  float end_head_ax, end_head_ay;
  float end_head_bx, end_head_by;
  float end_head_cx, end_head_cy;
  float rounding;
  uint head;
};

struct PreviewArrow {
  PreviewGeometry geometry;
  float red, green, blue, alpha;
  float hover;
  uint sample_first, sample_count;
  uint kind;
  uint flags;
  float params[3];
  uint data_offset, data_count;
};

/// A counter read out of the slots an arrow fills with its curve: the disc's
/// centre and radius, the tail's tip and the radius it is rounded to, and where
/// its number was rasterised in the text atlas. Prepared once per annotation by
/// `counter::geometry::prepare_counter`.
struct PreviewCounter {
  float2 center, tip;
  float radius, tip_radius;
  float2 text_origin, text_size;
};

PreviewCounter annotation_counter(PreviewGeometry prepared) {
  PreviewCounter counter;
  counter.center = float2(prepared.ax, prepared.ay);
  counter.tip = float2(prepared.bx, prepared.by);
  counter.radius = prepared.rounding;
  counter.tip_radius = prepared.low;
  counter.text_origin = float2(prepared.start_head_ax, prepared.start_head_ay);
  counter.text_size = float2(prepared.start_head_bx, prepared.start_head_by);
  return counter;
}

/// The annotation part way through the interval this frame covers, and how
/// solid it was then.
struct PreviewSample {
  PreviewGeometry geometry;
  float opacity;
};

StructuredBuffer<PreviewArrow> annotation_arrows : register(t5);
StructuredBuffer<PreviewSample> annotation_samples : register(t6);
/// The counters' numbers, rasterised at the size they are drawn and stacked
/// into one texture. The twin of the Metal kernels' number buffer.
Texture2D<float4> annotation_numbers : register(t7);
StructuredBuffer<float2> annotation_points : register(t8);
StructuredBuffer<uint> annotation_text : register(t9);

/// The halo's opacity, matching the ruler's.
static const float annotation_hover_alpha = 0.24;

// How many parameters the curve is sampled at before the best one is
// refined, and how many golden-section steps refine it.
static const uint annotation_curve_samples = 16u;
static const uint annotation_refine_steps = 10u;

float2 annotation_curve_point(float2 a, float2 leg, float2 bend, float t) {
  return a + (leg * 2.0 + bend * t) * t;
}

float annotation_curve_squared(float2 a, float2 leg, float2 bend, float2 probe, float t) {
  float2 offset = annotation_curve_point(a, leg, bend, t) - probe;
  return dot(offset, offset);
}

/// The distance from `point` to the quadratic Bezier `a, b, c`, with the
/// curve parameter restricted to `[low, high]`.
///
/// The refinement never trusts a derivative: the closed form scales by
/// 1 / |a - 2b + c|², which vanishes for the nearly straight arrow a plain
/// drag makes, and Newton on the derivative walks towards a maximum on a
/// tight bend. So golden-section on the squared distance itself, inside one
/// sample step of the best sample, with Newton only as a polish that has to
/// prove it shortened the distance.
float annotation_curve_distance(float2 probe, float2 a, float2 b, float2 c, float low, float high) {
  float2 leg = b - a;
  float2 bend = a - 2.0 * b + c;
  float span = max(high - low, 0.0);
  float step_size = span / (float)annotation_curve_samples;
  float best = low;
  float best_squared = 1e30;
  for (uint index = 0u; index <= annotation_curve_samples; ++index) {
    float t = low + step_size * (float)index;
    float squared = annotation_curve_squared(a, leg, bend, probe, t);
    if (squared < best_squared) {
      best_squared = squared;
      best = t;
    }
  }
  float left = max(best - step_size, low);
  float right = min(best + step_size, high);
  const float golden = 0.6180339887;
  float inner_left = right - (right - left) * golden;
  float inner_right = left + (right - left) * golden;
  float left_squared = annotation_curve_squared(a, leg, bend, probe, inner_left);
  float right_squared = annotation_curve_squared(a, leg, bend, probe, inner_right);
  for (uint refine = 0u; refine < annotation_refine_steps; ++refine) {
    if (left_squared < right_squared) {
      right = inner_right;
      inner_right = inner_left;
      right_squared = left_squared;
      inner_left = right - (right - left) * golden;
      left_squared = annotation_curve_squared(a, leg, bend, probe, inner_left);
    } else {
      left = inner_left;
      inner_left = inner_right;
      left_squared = right_squared;
      inner_right = left + (right - left) * golden;
      right_squared = annotation_curve_squared(a, leg, bend, probe, inner_right);
    }
  }
  float t = left_squared < right_squared ? inner_left : inner_right;
  float refined = min(left_squared, right_squared);
  for (uint polish = 0u; polish < 2u; ++polish) {
    float2 offset = annotation_curve_point(a, leg, bend, t) - probe;
    float2 tangent = leg * 2.0 + bend * (2.0 * t);
    float slope = dot(tangent, tangent) + 2.0 * dot(offset, bend);
    if (abs(slope) < 1e-6) break;
    float next = t - dot(offset, tangent) / slope;
    if (next < left || next > right) break;
    float squared = annotation_curve_squared(a, leg, bend, probe, next);
    if (squared >= refined) break;
    refined = squared;
    t = next;
  }
  return sqrt(min(best_squared, refined));
}

/// Signed distance to a triangle: negative inside it.
float annotation_triangle_distance(float2 probe, float2 a, float2 b, float2 c) {
  float2 edge_ab = b - a, edge_bc = c - b, edge_ca = a - c;
  float2 to_a = probe - a, to_b = probe - b, to_c = probe - c;
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

/// Shaft distance in `x` and head distance in `y`. Head vertices and shaft
/// limits were prepared before the draw; this only evaluates distances, and
/// picking uses the same prepared heads.
float2 annotation_arrow_distance(float2 probe, PreviewGeometry arrow) {
  float2 a = float2(arrow.ax, arrow.ay);
  float2 b = float2(arrow.bx, arrow.by);
  float2 c = float2(arrow.cx, arrow.cy);
  // An empty window is an annotation that has not started, or one whose head
  // has eaten what was left of its shaft. Either way there is no shaft to draw.
  float2 result = float2(arrow.high > arrow.low
      ? annotation_curve_distance(probe, a, b, c, arrow.low, arrow.high) - arrow.width * 0.5
      : 1e20, 1e20);
  if (arrow.head != 0u)
    result.y = min(result.y, annotation_triangle_distance(probe,
        float2(arrow.end_head_ax, arrow.end_head_ay),
        float2(arrow.end_head_bx, arrow.end_head_by),
        float2(arrow.end_head_cx, arrow.end_head_cy)) - arrow.rounding);
  if (arrow.head == 2u)
    result.y = min(result.y, annotation_triangle_distance(probe,
        float2(arrow.start_head_ax, arrow.start_head_ay),
        float2(arrow.start_head_bx, arrow.start_head_by),
        float2(arrow.start_head_cx, arrow.start_head_cy)) - arrow.rounding);
  return result;
}

/// How much of a pixel lies inside an edge `distance` canvas pixels away,
/// where `feather` is half a drawn pixel. A linear ramp across that one pixel
/// is the area a straight edge actually covers; `smoothstep` is half again as
/// steep at the edge and leaves visible steps on a 1x display.
float annotation_edge(float distance, float feather) {
  return saturate(0.5 - distance / (2.0 * feather));
}

/// Coverage of one prepared arrow, feathered over `feather` canvas pixels.
float annotation_coverage(float2 probe, PreviewGeometry arrow, float feather) {
  float2 distances = annotation_arrow_distance(probe, arrow);
  return max(annotation_edge(distances.x, feather), annotation_edge(distances.y, feather));
}

/// Accumulated exposure coverage: the annotation is drawn at every prepared
/// sample between the shutter start and now, so a moving shaft and its head
/// smear along the path they actually travelled while a held end stays sharp.
float annotation_exposure(float2 probe, PreviewArrow annotation, float feather) {
  float total = 0.0;
  for (uint tap = 0u; tap < annotation.sample_count; ++tap) {
    PreviewSample sample = annotation_samples[annotation.sample_first + tap];
    total += annotation_coverage(probe, sample.geometry, feather) * sample.opacity;
  }
  return total / (float)annotation.sample_count;
}

/// How far a point falls outside a counter's silhouette: the disc, unioned
/// with the tail - the overlap of two circles, one either side of the axis,
/// each tangent to the disc and to the little circle the tip is rounded to,
/// their centres a radius behind the disc's own.
///
/// The side circles are solved here from the disc, the tip and the reach.
/// Their overlap is measured the way any lens is - to the near circle inside
/// its span, to the shared corner past it - and that corner is the tip's
/// centre, so taking the tip's radius off rounds the point without moving it.
/// The overlap runs back behind the disc too, so it is cut at the plane where
/// the circles touch the disc: a chord, inside the union, which never shows.
/// The per-pixel twin of `counter_silhouette_distance` in
/// `annotations/counter/silhouette.rs`. The CPU side prepares and picks in
/// Rust; the shaders are the only other copies.
float annotation_counter_distance(float2 probe, PreviewCounter counter) {
  float2 local = probe - counter.center;
  float2 reach = counter.tip - counter.center;
  float length_reach = length(reach);
  if (counter.radius <= 0.0) return length(local);
  if (length_reach <= 0.0) return length(local) - counter.radius;
  float2 axis = reach / length_reach;
  float along = dot(local, axis);
  float across = abs(local.x * -axis.y + local.y * axis.x);
  float disc = length(local) - counter.radius;
  float gap = counter.radius - counter.tip_radius;
  if (gap <= 0.0) return disc;
  float tip = length_reach - counter.tip_radius;
  float side = (tip * tip + 2.0 * tip * counter.radius + gap * gap) / (2.0 * gap);
  float apart = side + counter.tip_radius - counter.radius;
  float offset = apart * apart - counter.radius * counter.radius;
  if (offset <= 0.0) return disc;
  offset = sqrt(offset);
  float lens = (along - tip) * offset > across * (tip + counter.radius)
      ? length(float2(across, along - tip))
      : length(float2(across + offset, along + counter.radius)) - side;
  float touch = counter.radius * counter.radius / apart;
  return min(disc, max(lens - counter.tip_radius, touch - along));
}

/// How many atlas pixels the type is rasterised at per canvas pixel.
static const float annotation_type_supersample = 2.0;

/// The atlas's coverage at `position`, in atlas pixels, blended between the
/// four nearest texels.
float annotation_atlas_bilinear(float2 position, uint2 atlas) {
  float2 at = position - 0.5;
  float2 base = floor(at);
  float2 fraction = at - base;
  int2 limit = int2(atlas) - 1;
  int2 low = clamp(int2(base), int2(0, 0), limit);
  int2 high = clamp(int2(base) + 1, int2(0, 0), limit);
  float a = annotation_numbers.Load(int3(low.x, low.y, 0)).a;
  float b = annotation_numbers.Load(int3(high.x, low.y, 0)).a;
  float c = annotation_numbers.Load(int3(low.x, high.y, 0)).a;
  float d = annotation_numbers.Load(int3(high.x, high.y, 0)).a;
  return lerp(lerp(a, b, fraction.x), lerp(c, d, fraction.x), fraction.y);
}

/// How much of the type in one atlas cell covers a drawn pixel centred on
/// `texel`, both in atlas pixels. A drawn pixel spans `feather * 2` canvas
/// pixels, so the whole of that footprint is averaged: a single sample would
/// pick one atlas pixel out of many where the canvas is shown smaller than
/// its resolution, and the type's edges would come out stepped. The samples
/// stay inside the cell so a neighbour's type never bleeds in.
float annotation_atlas_coverage(
    float2 texel, float feather, float2 cell_origin, float2 cell_size, uint2 atlas) {
  float span = max(feather * 2.0 * annotation_type_supersample, 1e-3);
  // Samples no more than an atlas pixel apart, and never fewer than two per
  // axis: one bilinear sample only averages the footprint when it lands on a
  // texel corner, and type set off the pixel grid would otherwise come out
  // stepped. Capped for a canvas shown very small.
  uint taps = clamp((uint)ceil(span), 2u, 6u);
  float step = span / (float)taps;
  float2 first = texel - span * 0.5 + step * 0.5;
  float2 low = cell_origin + 0.5;
  float2 high = max(cell_origin + cell_size - 0.5, low);
  float total = 0.0;
  for (uint row = 0u; row < taps; ++row) {
    for (uint column = 0u; column < taps; ++column) {
      float2 position = clamp(first + float2((float)column, (float)row) * step, low, high);
      total += annotation_atlas_bilinear(position, atlas);
    }
  }
  return total / (float)(taps * taps);
}

/// How much of the number covers this pixel, from the atlas the numbers were
/// rasterised into. The number is centred on the disc and carried by the
/// disc's own radius, so it grows and shrinks with the annotation.
float annotation_number_coverage(
    float2 probe, PreviewCounter counter, float feather, uint2 atlas) {
  if (atlas.x == 0u || atlas.y == 0u || counter.text_size.x <= 0.0 ||
      counter.text_size.y <= 0.0)
    return 0.0;
  float2 drawn = counter.text_size / annotation_type_supersample;
  float2 local = probe - counter.center + drawn * 0.5;
  if (any(local < 0.0) || any(local > drawn)) return 0.0;
  // The atlas rows run top-down from its first pixel, which is the space the
  // rasteriser reports its rectangles in.
  float2 texel = counter.text_origin + local * annotation_type_supersample;
  return annotation_atlas_coverage(
      texel, feather, counter.text_origin, counter.text_size, atlas);
}

/// Accumulated exposure coverage for a counter: the disc is drawn at every
/// prepared sample between the shutter start and now, so one that grew
/// through the frame smears over the sizes it covered rather than jumping
/// between them. Each sample carries its own opacity, which is what fades a
/// counter in and out of an exported frame.
float annotation_counter_exposure(float2 probe, PreviewArrow annotation, float feather) {
  float total = 0.0;
  for (uint tap = 0u; tap < annotation.sample_count; ++tap) {
    PreviewSample sample = annotation_samples[annotation.sample_first + tap];
    float distance =
        annotation_counter_distance(probe, annotation_counter(sample.geometry));
    total += annotation_edge(distance, feather) * sample.opacity;
  }
  return total / (float)annotation.sample_count;
}

/// One counter: its silhouette in the annotation's own colour, and its number
/// in whichever of black or white reads on that colour. The number is clipped
/// to the silhouette's own coverage, so the two share one antialiased edge.
float4 annotation_counter_layer(
    float4 rgba, PreviewArrow annotation, float4 color, float2 canvas_point,
    float feather, float halo, uint2 atlas) {
  PreviewCounter counter = annotation_counter(annotation.geometry);
  // The disc and tail fit within twice the radius of the centre. The exposure
  // samples run steadily from last frame's reveal to this one's, so the first
  // and last hold the largest radius any of them is drawn at.
  float radius = counter.radius;
  if (annotation.sample_count > 0u) {
    uint last = annotation.sample_first + annotation.sample_count - 1u;
    radius = max(radius, max(
        annotation_counter(annotation_samples[annotation.sample_first].geometry).radius,
        annotation_counter(annotation_samples[last].geometry).radius));
  }
  float reach = radius * 2.0 + halo + feather + 1.0;
  if (any(canvas_point < counter.center - reach) ||
      any(canvas_point > counter.center + reach))
    return rgba;
  float distance = annotation_counter_distance(canvas_point, counter);
  if (halo > 0.0) {
    // The halo hugs the silhouette from the edge outwards, the ruler's way.
    float band = (1.0 - annotation_edge(distance, feather)) *
        annotation_edge(distance - halo, feather);
    float alpha = band * color.a * annotation_hover_alpha;
    if (alpha > 0.0) {
      rgba.rgb = color.rgb * alpha + rgba.rgb * (1.0 - alpha);
      rgba.a = alpha + rgba.a * (1.0 - alpha);
    }
  }
  // A still frame draws the prepared disc directly, its opacity already
  // folded into the colour; a moving one averages the disc over the
  // exposure, where each sample carries the opacity it had.
  float coverage = annotation.sample_count == 0u
      ? annotation_edge(distance, feather)
      : annotation_counter_exposure(canvas_point, annotation, feather);
  if (coverage <= 0.0) return rgba;
  float alpha = coverage * color.a;
  rgba.rgb = color.rgb * alpha + rgba.rgb * (1.0 - alpha);
  rgba.a = alpha + rgba.a * (1.0 - alpha);
  float ink = annotation_number_coverage(canvas_point, counter, feather, atlas) * alpha;
  if (ink <= 0.0) return rgba;
  // Luminance rather than a fixed white: the palette runs from yellow to
  // near-black, and a number has to read on all of it.
  float luminance = dot(color.rgb, float3(0.2126, 0.7152, 0.0722));
  float3 tint = luminance > 0.6 ? float3(0.0, 0.0, 0.0) : float3(1.0, 1.0, 1.0);
  rgba.rgb = tint * ink + rgba.rgb * (1.0 - ink);
  rgba.a = ink + rgba.a * (1.0 - ink);
  return rgba;
}

#include "annotation_text.hlsl"

/// Draws the prepared annotations in `[first, last)` over `rgba`.
///
/// The range is how the camera ordering is expressed: Rust sorts the
/// annotations that sit under the camera ahead of those above it, so each pass
/// draws one contiguous run rather than testing a flag per annotation per
/// pixel.
///
/// Annotations are deliberately not clipped to the crop: an arrow may point in
/// from the padding. `feather` is how wide an edge is smoothed, in canvas
/// pixels, and `number_atlas` is the size of the texture the counters' numbers
/// were rasterised into - zero where nothing rasterised one.
float4 composite_annotations(
    float4 rgba, float2 canvas_point, uint first, uint last, float feather,
    uint2 number_atlas) {
  for (uint index = first; index < last; ++index) {
    PreviewArrow annotation = annotation_arrows[index];
    PreviewGeometry arrow = annotation.geometry;
    float4 color = float4(annotation.red, annotation.green, annotation.blue, annotation.alpha);
    if (color.a <= 0.0 || arrow.width <= 0.0) continue;
    if (annotation.kind == 1u) {
      rgba = annotation_counter_layer(rgba, annotation, color, canvas_point, feather,
                                      max(annotation.hover, 0.0), number_atlas);
      continue;
    }
    if (annotation.kind == 2u) {
      rgba = annotation_text_layer(rgba, annotation, color, canvas_point, feather,
                                   max(annotation.hover, 0.0), number_atlas);
      continue;
    }
    float2 a = float2(arrow.ax, arrow.ay);
    float2 b = float2(arrow.bx, arrow.by);
    float2 c = float2(arrow.cx, arrow.cy);
    // The curve lies inside the hull of its three points; the heads reach
    // back from a tip and across it. One rectangle covers all of that, and
    // keeps a long list cheap over most of the canvas.
    float halo = max(annotation.hover, 0.0);
    float radius = arrow.width * 0.5;
    float reach = radius * (arrow.head != 0u ? 9.0 : 1.0) + halo + feather + 1.0;
    if (any(canvas_point < min(a, min(b, c)) - reach) ||
        any(canvas_point > max(a, max(b, c)) + reach))
      continue;
    float2 distances = annotation_arrow_distance(canvas_point, arrow);
    float distance = min(distances.x, distances.y);
    if (halo > 0.0) {
      // The ruler's hover halo: an outline stroke in the shape's own colour,
      // hugging it from the edge outwards.
      float band = (1.0 - annotation_edge(distance, feather)) *
          annotation_edge(distance - halo, feather);
      float alpha = band * color.a * annotation_hover_alpha;
      if (alpha > 0.0) {
        rgba.rgb = color.rgb * alpha + rgba.rgb * (1.0 - alpha);
        rgba.a = alpha + rgba.a * (1.0 - alpha);
      }
    }
    // A still frame draws the prepared arrow directly; a moving one averages
    // the arrow over the exposure, head and shaft together.
    float coverage = annotation.sample_count == 0u
        ? max(annotation_edge(distances.x, feather), annotation_edge(distances.y, feather))
        : annotation_exposure(canvas_point, annotation, feather);
    if (coverage <= 0.0) continue;
    float alpha = coverage * color.a;
    rgba.rgb = color.rgb * alpha + rgba.rgb * (1.0 - alpha);
    rgba.a = alpha + rgba.a * (1.0 - alpha);
  }
  return rgba;
}
