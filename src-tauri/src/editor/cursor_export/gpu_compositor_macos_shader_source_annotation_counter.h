// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

/// The counter's own half of the annotation shader: the shape read out of the
/// slots an arrow fills with its curve, the distance to its silhouette, its
/// number sampled from the text atlas, and the layer that draws all three.
#define GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_COUNTER @R"METAL(
/// A counter read out of the slots an arrow fills with its curve: the disc's
/// centre and radius, the tail's tip and the radius it is rounded to, and where
/// the number was rasterised in the text atlas. Prepared once per annotation by
/// `annotation_prepare_counter`.
struct AnnotationCounter {
  float2 center, tip;
  float radius, tip_radius;
  float2 text_origin, text_size;
};

static AnnotationCounter annotation_counter(
    const device AnnotationArrowGeometry &prepared) {
  AnnotationCounter counter;
  counter.center = float2(prepared.a);
  counter.tip = float2(prepared.b);
  counter.radius = prepared.rounding;
  counter.tip_radius = prepared.low;
  counter.text_origin = float2(prepared.start_head.a);
  counter.text_size = float2(prepared.start_head.b);
  return counter;
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
/// The twin of `annotation_counter_silhouette_distance` in
/// `annotations/geometry.h`, which picks the same shape.
static float annotation_counter_distance(float2 point, AnnotationCounter counter) {
  float2 local = point - counter.center;
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

/// How much of the number covers this pixel, from the atlas the numbers were
/// rasterised into. The atlas is drawn at `supersample` pixels to the drawn
/// pixel and read with four taps, so a counter still reads while it is growing
/// into place.
///
/// The number is centred on the disc and carried by the disc's own radius, so
/// it grows and shrinks with the annotation without a second scale to keep in
/// step.
static float annotation_number_coverage(
    float2 point, AnnotationCounter counter, const device uchar4 *numbers,
    uint2 atlas) {
  if (atlas.x == 0u || atlas.y == 0u ||
      counter.text_size.x <= 0.0 || counter.text_size.y <= 0.0)
    return 0.0;
  const float supersample = 2.0;
  float2 drawn = counter.text_size / supersample;
  float2 local = point - counter.center + drawn * 0.5;
  if (any(local < 0.0) || any(local > drawn)) return 0.0;
  // The atlas rows run top-down from the buffer's first pixel, which is the
  // space the rasteriser reports its rectangles in.
  float2 texel = counter.text_origin + local * supersample;
  float total = 0.0;
  for (uint tap = 0u; tap < 4u; ++tap) {
    float2 offset = float2(float(tap & 1u), float(tap >> 1u)) * 0.5;
    uint2 at = uint2(clamp(texel + offset, float2(0.0), float2(atlas) - 1.0));
    total += float(numbers[at.y * atlas.x + at.x].a) / 255.0;
  }
  return total * 0.25;
}

/// Accumulated exposure coverage for a counter: the disc is drawn at every
/// prepared sample between the shutter start and now, so one that grew
/// through the frame smears over the sizes it covered rather than jumping
/// between them. Each sample carries its own opacity, which is what fades a
/// counter in and out of an exported frame.
static float annotation_counter_exposure(
    float2 point, const device AnnotationUniforms &annotation,
    const device AnnotationSample *samples, float feather) {
  float total = 0.0;
  for (uint tap = 0; tap < annotation.sample_count; ++tap) {
    const device AnnotationSample &sample = samples[annotation.sample_offset + tap];
    float distance = annotation_counter_distance(
        point, annotation_counter(sample.arrow));
    total += (1.0 - smoothstep(-feather, feather, distance)) * sample.opacity;
  }
  return total / float(annotation.sample_count);
}

/// One counter: its silhouette in the annotation's own colour, and its number
/// in whichever of black or white reads on that colour. The number is clipped
/// to the silhouette's own coverage, so the two share one antialiased edge.
static float4 annotation_counter_layer(
    float4 rgba, const device AnnotationUniforms &annotation, float4 color,
    float2 canvas_point, float feather, float halo,
    const device AnnotationSample *samples, const device uchar4 *numbers,
    uint2 number_atlas) {
  AnnotationCounter counter = annotation_counter(annotation.arrow);
  float reach = counter.radius * 2.0 + halo + feather + 1.0;
  if (any(canvas_point < counter.center - reach) ||
      any(canvas_point > counter.center + reach))
    return rgba;
  float distance = annotation_counter_distance(canvas_point, counter);
  if (halo > 0.0) {
    // The halo hugs the silhouette from the edge outwards, the ruler's way.
    float band = smoothstep(-feather, feather, distance) *
        (1.0 - smoothstep(halo - feather, halo + feather, distance));
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
      ? 1.0 - smoothstep(-feather, feather, distance)
      : annotation_counter_exposure(canvas_point, annotation, samples, feather);
  if (coverage <= 0.0) return rgba;
  float alpha = coverage * color.a;
  rgba.rgb = color.rgb * alpha + rgba.rgb * (1.0 - alpha);
  rgba.a = alpha + rgba.a * (1.0 - alpha);
  float ink = annotation_number_coverage(canvas_point, counter, numbers,
                                         number_atlas) * alpha;
  if (ink <= 0.0) return rgba;
  // Luminance rather than a fixed white: the palette runs from yellow to
  // near-black, and a number has to read on all of it.
  float luminance = dot(color.rgb, float3(0.2126, 0.7152, 0.0722));
  float3 tint = luminance > 0.6 ? float3(0.0) : float3(1.0);
  rgba.rgb = tint * ink + rgba.rgb * (1.0 - ink);
  rgba.a = ink + rgba.a * (1.0 - ink);
  return rgba;
}

)METAL"
