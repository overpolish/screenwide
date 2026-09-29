// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The shape: an outline round a rounded box, walked clockwise from the top
// side's left end - four sides and four corners - and the pen's stroke along
// a window of that walk with a round pen at each end. Each piece of the walk
// is measured on its own and the nearest kept, so wherever the pen went is
// drawn. The HLSL twin of
// `gpu_compositor_macos_shader_source_annotation_shape.h`. Every number it
// reads was prepared by Rust's `outline::geometry::prepare_shape`, which also
// decides a hand-drawn stroke's wander, and `shape_distance` there picks the
// same stroke. Included by `annotations.hlsl`, after the pieces every kind
// shares.

static const uint annotation_shape_kind = 5u;
static const float annotation_shape_quarter = 1.5707963268;
static const float annotation_shape_turn = 6.2831853072;
/// How many points along the pen's line each search for the nearest one
/// tries before its ends, as `TRIES` in `annotations/outline/geometry.rs`.
static const int annotation_shape_tries = 4;

float annotation_shape_perimeter(float2 half_size, float rounding) {
  return 4.0 * (half_size.x + half_size.y - 2.0 * rounding) + annotation_shape_turn * rounding;
}

/// How far the stroke sits outside the outline `along` into it in `x`, how
/// fast that changes along it in `y`, and how fast that change changes in
/// `z`: a bow all the way round, a lead-in easing off from the start and a
/// trail-off easing in towards the end. Nowhere for a clean stroke, whose
/// wander is all zero. The twin of `wander`, `slope` and `curl` in
/// `annotations/outline/wander.rs`.
float3 annotation_shape_wander(PreviewGeometry shape, float along) {
  float2 ease = max(float2(shape.end_head_bx, shape.end_head_by), 1e-6);
  // The lead-in settles in an S from its start; the trail-off flicks off,
  // steepest at the tip. Each is gone past its own span.
  float lead_x = saturate(along / ease.x);
  float trail_x = saturate((shape.cy - along) / ease.y);
  float lead_bend = lead_x > 0.0 && lead_x < 1.0
      ? -6.0 * (1.0 - 2.0 * lead_x) / (ease.x * ease.x) : 0.0;
  float trail_bend = trail_x < 1.0 ? 2.0 / (ease.y * ease.y) : 0.0;
  float first_at = shape.start_head_bx * along + shape.start_head_by;
  float second_at = shape.start_head_cx * along + shape.start_head_cy;
  return float3(
      shape.start_head_ax * (0.85 * sin(first_at) + 0.15 * sin(second_at)) +
          shape.start_head_ay * (1.0 - lead_x * lead_x * (3.0 - 2.0 * lead_x)) +
          shape.end_head_ax * (1.0 - trail_x) * (1.0 - trail_x),
      shape.start_head_ax * (0.85 * shape.start_head_bx * cos(first_at) +
                             0.15 * shape.start_head_cx * cos(second_at)) -
          shape.start_head_ay * 6.0 * lead_x * (1.0 - lead_x) / ease.x +
          shape.end_head_ax * 2.0 * (1.0 - trail_x) / ease.y,
      -shape.start_head_ax *
              (0.85 * shape.start_head_bx * shape.start_head_bx * sin(first_at) +
               0.15 * shape.start_head_cx * shape.start_head_cx * sin(second_at)) +
          shape.start_head_ay * lead_bend + shape.end_head_ax * trail_bend);
}

/// Where the stroke begins on the walk's piece `start` into it, before any
/// whole laps are added.
float annotation_shape_base(PreviewGeometry shape, float start, float lap) {
  float base = start - shape.cx;
  return base - lap * floor(base / lap);
}

/// The pen's line `at` into a piece whose pass begins `offset` into the
/// stroke: how far `local` is from it, how far `local` lies along the line's
/// direction, that direction's squared length, and how far `local` lies
/// along the line's curl. On a side `local` is its distance along the side
/// and out from it, and `at` a length; round a corner `local` is its reach
/// from the corner's centre and its turn, and `at` a turn. The twin of the
/// probes in `annotations/outline/geometry.rs`.
float4 annotation_shape_probe(PreviewGeometry shape, bool corner, float2 local, float offset,
                              float rounding, float at) {
  if (!corner) {
    float3 wander = annotation_shape_wander(shape, offset + at);
    float beyond = local.y - wander.x;
    return float4(length(float2(local.x - at, beyond)), local.x - at + beyond * wander.y,
                  1.0 + wander.y * wander.y, beyond * wander.z);
  }
  float3 wander = annotation_shape_wander(shape, offset + at * rounding);
  float bend = rounding + wander.x;
  // A stroke pulled inside a corner further than it is rounded needs no
  // join: its two sides already cross there.
  if (bend < 0.0) return float4(1e20, 0.0, 1.0, 0.0);
  float lean = wander.y * rounding;
  float swing = wander.z * rounding * rounding;
  // `local` sits `gap.x` beyond the line and `gap.y` ahead of it. Per unit
  // of turn the line runs `lean` out and `bend` on, and curls `swing - bend`
  // out and `2 * lean` on.
  float2 gap = local.x * float2(cos(local.y - at), sin(local.y - at)) - float2(bend, 0.0);
  return float4(length(gap), gap.x * lean + gap.y * bend, lean * lean + bend * bend,
                gap.x * (swing - bend) + gap.y * 2.0 * lean);
}

/// The distance from `local` to the nearest point of the pen's line within
/// `span`, searching from `at` by Newton steps, each held to twice what
/// following the line alone would take and halved back towards the best
/// point so far when it lands further away. The span's ends are measured
/// too, so a tip is never missed. The twin of `nearest` in
/// `annotations/outline/geometry.rs`.
float annotation_shape_nearest(PreviewGeometry shape, bool corner, float2 local, float offset,
                               float rounding, float2 span, float at) {
  float best = 1e20;
  float best_at = at;
  float shift = 0.0;
  for (int tries = 0; tries < annotation_shape_tries; ++tries) {
    float4 probe = annotation_shape_probe(shape, corner, local, offset, rounding, at);
    if (probe.x < best) {
      best = probe.x;
      best_at = at;
      shift = probe.y / max(max(probe.z - probe.w, probe.z * 0.5), 1e-6);
    } else {
      shift *= 0.5;
    }
    at = clamp(best_at + shift, span.x, span.y);
  }
  best = min(best, annotation_shape_probe(shape, corner, local, offset, rounding, at).x);
  best = min(best, annotation_shape_probe(shape, corner, local, offset, rounding, span.x).x);
  return min(best, annotation_shape_probe(shape, corner, local, offset, rounding, span.y).x);
}

/// How far `local` is from the pen's line along one side, from its `from`
/// end along `toward` for `run`, `normal` pointing out. `window` is the part
/// of the stroke drawn; the stroke may pass over the side on three laps. The
/// search starts square to the side, so a line leaving the outline keeps the
/// pen's width and a tip that flicks away ends in a true round pen.
float annotation_shape_side(
    float2 local, float2 from, float2 toward, float2 normal, float start, float run,
    PreviewGeometry shape, float lap, float2 window) {
  float2 relative = local - from;
  float2 across = float2(dot(relative, toward), dot(relative, normal));
  float base = annotation_shape_base(shape, start, lap);
  float nearest = 1e20;
  for (int turn = -1; turn <= 1; ++turn) {
    float offset = base + (float)turn * lap;
    float low = max(window.x - offset, 0.0);
    float high = min(window.y - offset, run);
    if (low > high) continue;
    nearest = min(nearest, annotation_shape_nearest(shape, false, across, offset, 0.0,
                                                    float2(low, high),
                                                    clamp(across.x, low, high)));
  }
  return nearest;
}

/// How far `local` is from the pen's line round one corner: a quarter turn
/// about `centre` from `from_angle`, searched by the turn as a side is by its
/// length. A square corner is passed at one moment, and faces every way
/// between its sides.
float annotation_shape_corner(
    float2 local, float2 centre, float from_angle, float rounding, float start,
    PreviewGeometry shape, float lap, float2 window) {
  float2 relative = local - centre;
  float turned = atan2(relative.y, relative.x) - from_angle;
  // Held to the half turn centred on the corner's own, so a point behind it
  // lands on the nearer of its two ends.
  turned -= annotation_shape_turn *
            floor((turned + 1.5 * annotation_shape_quarter) / annotation_shape_turn);
  float2 polar = float2(length(relative), turned);
  float base = annotation_shape_base(shape, start, lap);
  float nearest = 1e20;
  for (int turn = -1; turn <= 1; ++turn) {
    float offset = base + (float)turn * lap;
    float low = max(window.x - offset, 0.0);
    float high = min(window.y - offset, annotation_shape_quarter * rounding);
    if (low > high) continue;
    // A square corner's wander is one number, so the clamp is exact.
    if (rounding <= 0.0) {
      nearest = min(nearest, annotation_shape_probe(shape, true, polar, offset, 0.0,
                                                    clamp(turned, 0.0,
                                                          annotation_shape_quarter)).x);
      continue;
    }
    float2 span = float2(low, high) / rounding;
    nearest = min(nearest, annotation_shape_nearest(shape, true, polar, offset, rounding, span,
                                                    clamp(turned, span.x, span.y)));
  }
  return nearest;
}

/// How far `probe` falls outside the drawn stroke: zero on its edge and
/// negative inside it.
float annotation_shape_distance(float2 probe, PreviewGeometry shape) {
  if (shape.high <= shape.low || !(shape.cy > 0.0)) return 1e20;
  float2 low = float2(shape.ax, shape.ay);
  float2 high = float2(shape.bx, shape.by);
  float2 half_size = (high - low) * 0.5;
  float rounding = max(min(shape.rounding, min(half_size.x, half_size.y)), 0.0);
  float lap = annotation_shape_perimeter(half_size, rounding);
  float2 local = probe - (low + high) * 0.5;
  float2 window = float2(shape.low, shape.high) * shape.cy;
  float2 inner = half_size - rounding;
  float across = 2.0 * inner.x;
  float down = 2.0 * inner.y;
  float bend = annotation_shape_quarter * rounding;
  float q = annotation_shape_quarter;
  // Clockwise from the top side's left end: each side, then the corner
  // after it.
  float start = 0.0;
  float nearest = annotation_shape_side(local, float2(-inner.x, -half_size.y), float2(1.0, 0.0),
                                        float2(0.0, -1.0), start, across, shape, lap, window);
  start += across;
  nearest = min(nearest, annotation_shape_corner(local, float2(inner.x, -inner.y), -q, rounding,
                                                 start, shape, lap, window));
  start += bend;
  nearest = min(nearest, annotation_shape_side(local, float2(half_size.x, -inner.y),
                                               float2(0.0, 1.0), float2(1.0, 0.0), start, down,
                                               shape, lap, window));
  start += down;
  nearest = min(nearest, annotation_shape_corner(local, inner, 0.0, rounding, start, shape, lap,
                                                 window));
  start += bend;
  nearest = min(nearest, annotation_shape_side(local, float2(inner.x, half_size.y),
                                               float2(-1.0, 0.0), float2(0.0, 1.0), start, across,
                                               shape, lap, window));
  start += across;
  nearest = min(nearest, annotation_shape_corner(local, float2(-inner.x, inner.y), q, rounding,
                                                 start, shape, lap, window));
  start += bend;
  nearest = min(nearest, annotation_shape_side(local, float2(-half_size.x, inner.y),
                                               float2(0.0, -1.0), float2(-1.0, 0.0), start, down,
                                               shape, lap, window));
  start += down;
  nearest = min(nearest, annotation_shape_corner(local, -inner, 2.0 * q, rounding, start, shape,
                                                 lap, window));
  return nearest - shape.width * 0.5;
}

/// Accumulated exposure coverage for a shape: the stroke is drawn at every
/// prepared sample between the shutter start and now, so an arriving one
/// smears along the outline it covered. Each sample carries its own opacity.
float annotation_shape_exposure(float2 probe, PreviewArrow annotation, float feather) {
  float total = 0.0;
  for (uint tap = 0u; tap < annotation.sample_count; ++tap) {
    PreviewSample sample = annotation_samples[annotation.sample_first + tap];
    total += annotation_edge(annotation_shape_distance(probe, sample.geometry), feather) *
             sample.opacity;
  }
  return total / (float)annotation.sample_count;
}

/// One shape: its stroke in the annotation's own colour.
float4 annotation_shape_layer(
    float4 rgba, PreviewArrow annotation, float4 color, float2 canvas_point,
    float feather, float halo) {
  PreviewGeometry shape = annotation.geometry;
  // The stroke stays within the pen and its wander of the clean outline.
  // Every exposure sample shares the box and the wander, so these tests
  // cover all of them.
  float reach = shape.width * 0.5 + abs(shape.start_head_ax) +
                max(abs(shape.start_head_ay), abs(shape.end_head_ax)) + halo + feather + 1.0;
  float2 low = float2(shape.ax, shape.ay);
  float2 high = float2(shape.bx, shape.by);
  if (any(canvas_point < low - reach) || any(canvas_point > high + reach)) return rgba;
  float2 half_size = (high - low) * 0.5;
  float rounding = max(min(shape.rounding, min(half_size.x, half_size.y)), 0.0);
  float2 q = abs(canvas_point - (low + high) * 0.5) - half_size + rounding;
  if (abs(length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - rounding) > reach) return rgba;
  float distance = annotation_shape_distance(canvas_point, shape);
  if (halo > 0.0) {
    // The halo hugs the stroke from its edge outwards, the ruler's way.
    float band = (1.0 - annotation_edge(distance, feather)) *
        annotation_edge(distance - halo, feather);
    float alpha = band * color.a * annotation_hover_alpha;
    if (alpha > 0.0) {
      rgba.rgb = color.rgb * alpha + rgba.rgb * (1.0 - alpha);
      rgba.a = alpha + rgba.a * (1.0 - alpha);
    }
  }
  // A still frame draws the prepared stroke directly, its opacity already
  // folded into the colour; a moving one averages it over the exposure.
  float coverage = annotation.sample_count == 0u
      ? annotation_edge(distance, feather)
      : annotation_shape_exposure(canvas_point, annotation, feather);
  if (coverage <= 0.0) return rgba;
  float alpha = coverage * color.a;
  rgba.rgb = color.rgb * alpha + rgba.rgb * (1.0 - alpha);
  rgba.a = alpha + rgba.a * (1.0 - alpha);
  return rgba;
}
