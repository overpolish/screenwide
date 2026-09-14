// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#define GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATIONS @R"METAL(
struct AnnotationUniforms {
  uint kind;
  uint head;
  uint above_camera;
  float width;
  packed_float4 color;
  packed_float2 p0, p1, p2;
  float hover;
};
static_assert(sizeof(AnnotationUniforms) == 60,
              "Annotation uniforms must match their native layout");

// Head geometry as multiples of the stroke width. The shaft stops where the
// head begins so a translucent colour does not double up under the tip.
constant float annotation_head_length = 4.0;
constant float annotation_head_half_base = 2.0;
// How many parameters the curve is sampled at before the best one is
// refined, how many golden-section steps refine it, and how finely the head's
// base is bisected along the curve.
constant uint annotation_curve_samples = 16u;
constant uint annotation_refine_steps = 10u;
constant uint annotation_trim_steps = 10u;
// The hover halo's alpha, matching the ruler's.
constant float annotation_hover_alpha = 0.24;

static float2 annotation_curve_point(
    float2 a, float2 leg, float2 bend, float t) {
  return a + (leg * 2.0 + bend * t) * t;
}

static float annotation_curve_squared(
    float2 a, float2 leg, float2 bend, float2 point, float t) {
  float2 offset = annotation_curve_point(a, leg, bend, t) - point;
  return dot(offset, offset);
}

/// The distance from `point` to the quadratic Bézier `a, b, c`, with the
/// curve parameter restricted to `[low, high]`.
///
/// Two things are hopeless here. The closed form - the cubic solve on the
/// derivative of the squared distance - scales its coefficients by
/// 1 / |a - 2b + c|², which vanishes for the nearly straight arrow a plain
/// drag makes, so in fp32 at pixel scale whole stretches of shaft resolve to
/// the wrong parameter. Newton on f(t) = dot(B(t) - p, B'(t)) is no better at
/// the other extreme: f'(t) = |B'|² + 2·dot(B(t) - p, bend), and on a tight
/// bend `bend` is thousands of pixels, so the second term rivals the first,
/// the step walks towards a maximum, and the sample it fell back on sags away
/// from the curve by half a sample step - a gap with two round caps.
///
/// So the refinement never trusts a derivative: golden-section on the squared
/// distance itself, inside one sample step of the best sample, and Newton
/// only as a polish that has to prove it shortened the distance.
static float annotation_curve_distance(
    float2 point, float2 a, float2 b, float2 c, float low, float high) {
  float2 leg = b - a;
  float2 bend = a - 2.0 * b + c;
  float span = max(high - low, 0.0);
  float step = span / float(annotation_curve_samples);
  float best = low;
  float best_squared = 1e30;
  for (uint index = 0u; index <= annotation_curve_samples; ++index) {
    float t = low + step * float(index);
    float squared = annotation_curve_squared(a, leg, bend, point, t);
    if (squared < best_squared) {
      best_squared = squared;
      best = t;
    }
  }
  float left = max(best - step, low);
  float right = min(best + step, high);
  const float golden = 0.6180339887;
  float inner_left = right - (right - left) * golden;
  float inner_right = left + (right - left) * golden;
  float left_squared = annotation_curve_squared(a, leg, bend, point, inner_left);
  float right_squared =
      annotation_curve_squared(a, leg, bend, point, inner_right);
  for (uint index = 0u; index < annotation_refine_steps; ++index) {
    if (left_squared < right_squared) {
      right = inner_right;
      inner_right = inner_left;
      right_squared = left_squared;
      inner_left = right - (right - left) * golden;
      left_squared = annotation_curve_squared(a, leg, bend, point, inner_left);
    } else {
      left = inner_left;
      inner_left = inner_right;
      left_squared = right_squared;
      inner_right = left + (right - left) * golden;
      right_squared = annotation_curve_squared(a, leg, bend, point, inner_right);
    }
  }
  float t = left_squared < right_squared ? inner_left : inner_right;
  float refined = min(left_squared, right_squared);
  for (uint polish = 0u; polish < 2u; ++polish) {
    float2 offset = annotation_curve_point(a, leg, bend, t) - point;
    float2 tangent = leg * 2.0 + bend * (2.0 * t);
    float slope = dot(tangent, tangent) + 2.0 * dot(offset, bend);
    if (abs(slope) < 1e-6) break;
    float next = t - dot(offset, tangent) / slope;
    if (next < left || next > right) break;
    float squared = annotation_curve_squared(a, leg, bend, point, next);
    if (squared >= refined) break;
    refined = squared;
    t = next;
  }
  return sqrt(min(best_squared, refined));
}

/// The curve parameter where the shaft meets a head: the point at which the
/// curve comes within `head_length` of `tip`. Trimming the shaft by parameter
/// rather than by the half-plane behind the head's base is what lets a
/// strongly bent arrow re-enter that half-plane without losing its shaft.
static float annotation_head_parameter(
    float2 a, float2 leg, float2 bend, float2 tip, float head_length,
    bool at_end) {
  float low = 0.0;
  float high = 1.0;
  float limit = head_length * head_length;
  for (uint step = 0u; step < annotation_trim_steps; ++step) {
    float middle = (low + high) * 0.5;
    float2 offset = annotation_curve_point(a, leg, bend, middle) - tip;
    // The end head covers the tail of the curve and the start head its head,
    // so the two walk the bracket in opposite directions.
    if ((dot(offset, offset) <= limit) == at_end) high = middle;
    else low = middle;
  }
  return at_end ? high : low;
}

/// A head's unit direction, falling back to the chord when the control point
/// sits on the tip it would otherwise aim from.
static float2 annotation_head_direction(float2 primary, float2 fallback) {
  float2 direction = dot(primary, primary) > 1e-8 ? primary : fallback;
  float squared = dot(direction, direction);
  return squared > 1e-12 ? direction * rsqrt(squared) : float2(1.0, 0.0);
}

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

/// The signed distance to one arrow, in canvas pixels. Negative is inside.
static float annotation_arrow_distance(
    float2 point, float2 a, float2 b, float2 c, float width, uint head) {
  float radius = max(width, 0.0) * 0.5;
  float head_length = radius * 2.0 * annotation_head_length;
  float half_base = radius * 2.0 * annotation_head_half_base;
  float2 leg = b - a;
  float2 bend = a - 2.0 * b + c;
  float low = 0.0;
  float high = 1.0;
  float result = 1e9;
  if (head != 0u) {
    high = annotation_head_parameter(a, leg, bend, c, head_length, true);
    float2 direction = annotation_head_direction(c - b, c - a);
    float2 base = c - direction * head_length;
    float2 across = float2(-direction.y, direction.x) * half_base;
    result = annotation_triangle_distance(
        point, c, base + across, base - across);
  }
  if (head == 2u) {
    low = annotation_head_parameter(a, leg, bend, a, head_length, false);
    float2 direction = annotation_head_direction(a - b, a - c);
    float2 base = a - direction * head_length;
    float2 across = float2(-direction.y, direction.x) * half_base;
    result = min(result, annotation_triangle_distance(
        point, a, base + across, base - across));
  }
  float shaft = annotation_curve_distance(
      point, a, b, c, min(low, high), high) - radius;
  return min(shaft, result);
}

/// Draws every mark whose layer matches `above_camera`.
///
/// Points are in the source's own pixel space, so they are mapped through the
/// same image placement the content layer is sampled with and travel with the
/// picture when the frame is moved or resized. Marks are deliberately not
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
  float2 scale = float2(u.image_width, u.image_height) / source_dimensions;
  float2 origin = float2(u.image_x, u.image_y);
  for (uint index = 0; index < count; ++index) {
    const device AnnotationUniforms &mark = annotations[index];
    if (mark.kind != 0u || mark.above_camera != above_camera) continue;
    float4 color = float4(mark.color);
    if (color.a <= 0.0 || mark.width <= 0.0) continue;
    float2 a = origin + float2(mark.p0) * scale;
    float2 b = origin + float2(mark.p1) * scale;
    float2 c = origin + float2(mark.p2) * scale;
    // The curve lies inside the hull of its three points; the heads reach
    // `head_length` back from a tip and `half_base` across. One rectangle
    // covers all of it, and keeps a long list cheap for most of the canvas.
    float halo = max(mark.hover, 0.0);
    float radius = mark.width * 0.5;
    float reach = radius * (mark.head != 0u
        ? 1.0 + 2.0 * annotation_head_length : 1.0) + halo + feather + 1.0;
    if (any(canvas_point < min(a, min(b, c)) - reach) ||
        any(canvas_point > max(a, max(b, c)) + reach))
      continue;
    float distance = annotation_arrow_distance(
        canvas_point, a, b, c, mark.width, mark.head);
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
