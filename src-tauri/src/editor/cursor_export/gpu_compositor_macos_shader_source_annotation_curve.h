// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

/// The distance from a point to an arrow's quadratic Bézier, which is the
/// whole of what makes a shaft. Kept apart from the shapes it draws because
/// it is the one piece here that is a numerical method rather than geometry,
/// and its two failure modes are worth reading in one sitting.
#define GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_CURVE @R"METAL(
// How many parameters the curve is sampled at before the best one is
// refined, and how many golden-section steps refine it.
constant uint annotation_curve_samples = 16u;
constant uint annotation_refine_steps = 10u;
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
)METAL"
