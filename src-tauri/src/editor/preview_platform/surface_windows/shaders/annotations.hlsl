// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Arrow annotations, the HLSL twin of
// `gpu_compositor_macos_shader_source_annotation_curve.h` and
// `gpu_compositor_macos_shader_source_annotations.h`. Geometry arrives already
// prepared in canvas pixels - `annotations::geometry::prepare_arrow` on the
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
  /// The hover halo's width in canvas pixels; zero when nothing is hovered.
  float hover;
  /// This mark's run in `annotation_samples`. A count of zero draws the
  /// prepared geometry directly, as a still always does.
  uint sample_first, sample_count;
};

/// The mark part way through the interval this frame covers, and how solid
/// it was then.
struct PreviewSample {
  PreviewGeometry geometry;
  float opacity;
};

StructuredBuffer<PreviewArrow> annotation_arrows : register(t5);
StructuredBuffer<PreviewSample> annotation_samples : register(t6);

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
  // An empty window is a mark that has not started, or one whose head has
  // eaten what was left of its shaft. Either way there is no shaft to draw.
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

/// Coverage of one prepared arrow, feathered over `feather` canvas pixels.
float annotation_coverage(float2 probe, PreviewGeometry arrow, float feather) {
  float2 distances = annotation_arrow_distance(probe, arrow);
  return max(1.0 - smoothstep(-feather, feather, distances.x),
             1.0 - smoothstep(-feather, feather, distances.y));
}

/// Accumulated exposure coverage: the mark is drawn at every prepared sample
/// between the shutter start and now, so a moving shaft and its head smear
/// along the path they actually travelled while a held end stays sharp.
float annotation_exposure(float2 probe, PreviewArrow mark, float feather) {
  float total = 0.0;
  for (uint tap = 0u; tap < mark.sample_count; ++tap) {
    PreviewSample sample = annotation_samples[mark.sample_first + tap];
    total += annotation_coverage(probe, sample.geometry, feather) * sample.opacity;
  }
  return total / (float)mark.sample_count;
}

/// Draws the prepared marks in `[first, last)` over `rgba`.
///
/// The range is how the camera ordering is expressed: Rust sorts the marks
/// that sit under the camera ahead of those above it, so each pass draws one
/// contiguous run rather than testing a flag per mark per pixel.
///
/// Marks are deliberately not clipped to the crop: an arrow may point in from
/// the padding. `feather` is how wide an edge is smoothed, in canvas pixels.
float4 composite_annotations(
    float4 rgba, float2 canvas_point, uint first, uint last, float feather) {
  for (uint index = first; index < last; ++index) {
    PreviewArrow mark = annotation_arrows[index];
    PreviewGeometry arrow = mark.geometry;
    float4 color = float4(mark.red, mark.green, mark.blue, mark.alpha);
    if (color.a <= 0.0 || arrow.width <= 0.0) continue;
    float2 a = float2(arrow.ax, arrow.ay);
    float2 b = float2(arrow.bx, arrow.by);
    float2 c = float2(arrow.cx, arrow.cy);
    // The curve lies inside the hull of its three points; the heads reach
    // back from a tip and across it. One rectangle covers all of that, and
    // keeps a long list cheap over most of the canvas.
    float halo = max(mark.hover, 0.0);
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
      float band = smoothstep(-feather, feather, distance) *
          (1.0 - smoothstep(halo - feather, halo + feather, distance));
      float alpha = band * color.a * annotation_hover_alpha;
      if (alpha > 0.0) {
        rgba.rgb = color.rgb * alpha + rgba.rgb * (1.0 - alpha);
        rgba.a = alpha + rgba.a * (1.0 - alpha);
      }
    }
    // A still frame draws the prepared arrow directly; a moving one averages
    // the arrow over the exposure, head and shaft together.
    float coverage = mark.sample_count == 0u
        ? max(1.0 - smoothstep(-feather, feather, distances.x),
              1.0 - smoothstep(-feather, feather, distances.y))
        : annotation_exposure(canvas_point, mark, feather);
    if (coverage <= 0.0) continue;
    float alpha = coverage * color.a;
    rgba.rgb = color.rgb * alpha + rgba.rgb * (1.0 - alpha);
    rgba.a = alpha + rgba.a * (1.0 - alpha);
  }
  return rgba;
}
