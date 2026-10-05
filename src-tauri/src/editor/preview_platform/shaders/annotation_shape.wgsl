// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The shape: an outline round a rounded box, walked clockwise from the top
// side's left end - four sides and four corners - and the pen's stroke along
// a window of that walk with a round pen at each end. Each piece of the walk
// is measured on its own and the nearest kept, so wherever the pen went is
// drawn. Every number it reads was prepared by Rust's
// `outline::geometry::prepare_shape`, which also decides a hand-drawn
// stroke's wander, and `shape_distance` there picks the same stroke.

const annotation_shape_kind: u32 = 5u;
const annotation_shape_quarter: f32 = 1.5707963268;
const annotation_shape_turn: f32 = 6.2831853072;
/// How many points along the pen's line each search for the nearest one
/// tries before its ends, as `TRIES` in `annotations/outline/geometry.rs`.
const annotation_shape_tries: i32 = 4;

fn annotation_shape_perimeter(half_size: vec2<f32>, rounding: f32) -> f32 {
  return 4.0 * (half_size.x + half_size.y - 2.0 * rounding) + annotation_shape_turn * rounding;
}

/// How far the stroke sits outside the outline `along` into it in `x`, how
/// fast that changes along it in `y`, and how fast that change changes in
/// `z`: a bow all the way round, a lead-in easing off from the start and a
/// trail-off easing in towards the end. Nowhere for a clean stroke, whose
/// wander is all zero. The twin of `wander`, `slope` and `curl` in
/// `annotations/outline/wander.rs`.
fn annotation_shape_wander(shape: PreviewGeometry, along: f32) -> vec3<f32> {
  let ease = max(vec2<f32>(shape.end_head_bx, shape.end_head_by), vec2<f32>(1e-6));
  // The lead-in settles in an S from its start; the trail-off flicks off,
  // steepest at the tip. Each is gone past its own span.
  let lead_x = saturate(along / ease.x);
  let trail_x = saturate((shape.cy - along) / ease.y);
  var lead_bend = 0.0;
  if (lead_x > 0.0 && lead_x < 1.0) {
    lead_bend = -6.0 * (1.0 - 2.0 * lead_x) / (ease.x * ease.x);
  }
  var trail_bend = 0.0;
  if (trail_x < 1.0) {
    trail_bend = 2.0 / (ease.y * ease.y);
  }
  let first_at = shape.start_head_bx * along + shape.start_head_by;
  let second_at = shape.start_head_cx * along + shape.start_head_cy;
  return vec3<f32>(
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
fn annotation_shape_base(shape: PreviewGeometry, start: f32, lap: f32) -> f32 {
  let base = start - shape.cx;
  return base - lap * floor(base / lap);
}

/// The pen's line `at` into a piece whose pass begins `offset` into the
/// stroke: how far `local` is from it, how far `local` lies along the line's
/// direction, that direction's squared length, and how far `local` lies
/// along the line's curl. On a side `local` is its distance along the side
/// and out from it, and `at` a length; round a corner `local` is its reach
/// from the corner's centre and its turn, and `at` a turn. The twin of the
/// probes in `annotations/outline/geometry.rs`.
fn annotation_shape_probe(shape: PreviewGeometry, corner: bool, local: vec2<f32>, offset: f32,
                          rounding: f32, at: f32) -> vec4<f32> {
  if (!corner) {
    let wander = annotation_shape_wander(shape, offset + at);
    let beyond = local.y - wander.x;
    return vec4<f32>(length(vec2<f32>(local.x - at, beyond)), local.x - at + beyond * wander.y,
                     1.0 + wander.y * wander.y, beyond * wander.z);
  }
  let wander = annotation_shape_wander(shape, offset + at * rounding);
  let bend = rounding + wander.x;
  // A stroke pulled inside a corner further than it is rounded needs no
  // join: its two sides already cross there.
  if (bend < 0.0) {
    return vec4<f32>(1e20, 0.0, 1.0, 0.0);
  }
  let lean = wander.y * rounding;
  let swing = wander.z * rounding * rounding;
  // `local` sits `gap.x` beyond the line and `gap.y` ahead of it. Per unit
  // of turn the line runs `lean` out and `bend` on, and curls `swing - bend`
  // out and `2 * lean` on.
  let gap = local.x * vec2<f32>(cos(local.y - at), sin(local.y - at)) - vec2<f32>(bend, 0.0);
  return vec4<f32>(length(gap), gap.x * lean + gap.y * bend, lean * lean + bend * bend,
                   gap.x * (swing - bend) + gap.y * 2.0 * lean);
}

/// The distance from `local` to the nearest point of the pen's line within
/// `span`, searching from `at` by Newton steps, each held to twice what
/// following the line alone would take and halved back towards the best
/// point so far when it lands further away. The span's ends are measured
/// too, so a tip is never missed. The twin of `nearest` in
/// `annotations/outline/geometry.rs`.
fn annotation_shape_nearest(shape: PreviewGeometry, corner: bool, local: vec2<f32>, offset: f32,
                            rounding: f32, span: vec2<f32>, start_at: f32) -> f32 {
  var at = start_at;
  var best = 1e20;
  var best_at = at;
  var shift = 0.0;
  for (var tries = 0; tries < annotation_shape_tries; tries++) {
    let probe = annotation_shape_probe(shape, corner, local, offset, rounding, at);
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

/// How far `local` is from the pen's line along one side, from its `origin`
/// end along `toward` for `run`, `normal` pointing out. `window` is the part
/// of the stroke drawn; the stroke may pass over the side on three laps. The
/// search starts square to the side, so a line leaving the outline keeps the
/// pen's width and a tip that flicks away ends in a true round pen.
fn annotation_shape_side(local: vec2<f32>, origin: vec2<f32>, toward: vec2<f32>, normal: vec2<f32>,
                         start: f32, run: f32, shape: PreviewGeometry, lap: f32,
                         window: vec2<f32>) -> f32 {
  let relative = local - origin;
  let across = vec2<f32>(dot(relative, toward), dot(relative, normal));
  let base = annotation_shape_base(shape, start, lap);
  var nearest = 1e20;
  for (var turn = -1; turn <= 1; turn++) {
    let offset = base + f32(turn) * lap;
    let low = max(window.x - offset, 0.0);
    let high = min(window.y - offset, run);
    if (low > high) {
      continue;
    }
    nearest = min(nearest, annotation_shape_nearest(shape, false, across, offset, 0.0,
                                                    vec2<f32>(low, high),
                                                    clamp(across.x, low, high)));
  }
  return nearest;
}

/// How far `local` is from the pen's line round one corner: a quarter turn
/// about `centre` from `from_angle`, searched by the turn as a side is by its
/// length. A square corner is passed at one moment, and faces every way
/// between its sides.
fn annotation_shape_corner(local: vec2<f32>, centre: vec2<f32>, from_angle: f32, rounding: f32,
                           start: f32, shape: PreviewGeometry, lap: f32,
                           window: vec2<f32>) -> f32 {
  let relative = local - centre;
  var turned = atan2(relative.y, relative.x) - from_angle;
  // Held to the half turn centred on the corner's own, so a point behind it
  // lands on the nearer of its two ends.
  turned -= annotation_shape_turn *
      floor((turned + 1.5 * annotation_shape_quarter) / annotation_shape_turn);
  let polar = vec2<f32>(length(relative), turned);
  let base = annotation_shape_base(shape, start, lap);
  var nearest = 1e20;
  for (var turn = -1; turn <= 1; turn++) {
    let offset = base + f32(turn) * lap;
    let low = max(window.x - offset, 0.0);
    let high = min(window.y - offset, annotation_shape_quarter * rounding);
    if (low > high) {
      continue;
    }
    // A square corner's wander is one number, so the clamp is exact.
    if (rounding <= 0.0) {
      nearest = min(nearest, annotation_shape_probe(shape, true, polar, offset, 0.0,
                                                    clamp(turned, 0.0,
                                                          annotation_shape_quarter)).x);
      continue;
    }
    let span = vec2<f32>(low, high) / rounding;
    nearest = min(nearest, annotation_shape_nearest(shape, true, polar, offset, rounding, span,
                                                    clamp(turned, span.x, span.y)));
  }
  return nearest;
}

/// How far `probe` falls outside the drawn stroke: zero on its edge and
/// negative inside it.
fn annotation_shape_distance(probe: vec2<f32>, shape: PreviewGeometry) -> f32 {
  if (shape.high <= shape.low || !(shape.cy > 0.0)) {
    return 1e20;
  }
  let low = vec2<f32>(shape.ax, shape.ay);
  let high = vec2<f32>(shape.bx, shape.by);
  let half_size = (high - low) * 0.5;
  let rounding = max(min(shape.rounding, min(half_size.x, half_size.y)), 0.0);
  let lap = annotation_shape_perimeter(half_size, rounding);
  let local = probe - (low + high) * 0.5;
  let window = vec2<f32>(shape.low, shape.high) * shape.cy;
  let inner = half_size - rounding;
  let across = 2.0 * inner.x;
  let down = 2.0 * inner.y;
  let bend = annotation_shape_quarter * rounding;
  let q = annotation_shape_quarter;
  // Clockwise from the top side's left end: each side, then the corner
  // after it.
  var side_from = array<vec2<f32>, 4>(vec2<f32>(-inner.x, -half_size.y),
                                      vec2<f32>(half_size.x, -inner.y),
                                      vec2<f32>(inner.x, half_size.y),
                                      vec2<f32>(-half_size.x, inner.y));
  var side_toward = array<vec2<f32>, 4>(vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0),
                                        vec2<f32>(-1.0, 0.0), vec2<f32>(0.0, -1.0));
  var side_normal = array<vec2<f32>, 4>(vec2<f32>(0.0, -1.0), vec2<f32>(1.0, 0.0),
                                        vec2<f32>(0.0, 1.0), vec2<f32>(-1.0, 0.0));
  var corner_centre = array<vec2<f32>, 4>(vec2<f32>(inner.x, -inner.y), inner,
                                          vec2<f32>(-inner.x, inner.y), -inner);
  var corner_angle = array<f32, 4>(-q, 0.0, q, 2.0 * q);
  var start = 0.0;
  var nearest = 1e20;
  for (var quarter = 0u; quarter < 4u; quarter++) {
    let run = select(down, across, (quarter & 1u) == 0u);
    nearest = min(nearest, annotation_shape_side(local, side_from[quarter], side_toward[quarter],
                                                 side_normal[quarter], start, run, shape, lap,
                                                 window));
    start += run;
    nearest = min(nearest, annotation_shape_corner(local, corner_centre[quarter],
                                                   corner_angle[quarter], rounding, start, shape,
                                                   lap, window));
    start += bend;
  }
  return nearest - shape.width * 0.5;
}

/// The clean outline's point `along` the walk, a lap of `lap` round a box
/// `half_size` across each way with corners rounded by `rounding`: what the
/// pen's line strays from by its wander alone.
fn annotation_shape_walk_point(half_size: vec2<f32>, rounding: f32, lap: f32,
                               along_walk: f32) -> vec2<f32> {
  let inner = half_size - rounding;
  let bend = annotation_shape_quarter * rounding;
  let q = annotation_shape_quarter;
  var side_from = array<vec2<f32>, 4>(vec2<f32>(-inner.x, -half_size.y),
                                      vec2<f32>(half_size.x, -inner.y),
                                      vec2<f32>(inner.x, half_size.y),
                                      vec2<f32>(-half_size.x, inner.y));
  var side_toward = array<vec2<f32>, 4>(vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0),
                                        vec2<f32>(-1.0, 0.0), vec2<f32>(0.0, -1.0));
  var corner_centre = array<vec2<f32>, 4>(vec2<f32>(inner.x, -inner.y), inner,
                                          vec2<f32>(-inner.x, inner.y), -inner);
  var corner_angle = array<f32, 4>(-q, 0.0, q, 2.0 * q);
  var along = along_walk - lap * floor(along_walk / max(lap, 1e-6));
  for (var quarter = 0u; quarter < 4u; quarter++) {
    let run = select(2.0 * inner.y, 2.0 * inner.x, (quarter & 1u) == 0u);
    if (along <= run) {
      return side_from[quarter] + side_toward[quarter] * along;
    }
    along -= run;
    if (along <= bend || quarter == 3u) {
      let angle = corner_angle[quarter] + min(along, bend) / max(rounding, 1e-6);
      return corner_centre[quarter] + rounding * vec2<f32>(cos(angle), sin(angle));
    }
    along -= bend;
  }
  return side_from[0];
}

/// Whether the stretch of the stroke from `span.x` to `span.y` into it can
/// reach `local`, measured from the box's middle, within `reach`. The clean
/// outline is walked by its own length, so that stretch of it lies within half
/// its length of its middle, and the pen's line strays from it by no more than
/// the wander `reach` already allows.
fn annotation_shape_stretch_near(local: vec2<f32>, shape: PreviewGeometry, half_size: vec2<f32>,
                                 rounding: f32, lap: f32, span: vec2<f32>, reach: f32) -> bool {
  if (span.y <= span.x) {
    return false;
  }
  let middle = annotation_shape_walk_point(half_size, rounding, lap,
                                           shape.cx + 0.5 * (span.x + span.y));
  return length(local - middle) <= 0.5 * (span.y - span.x) + reach;
}

/// Accumulated exposure coverage for a shape: the stroke is drawn at every
/// prepared sample between the shutter start and now, so an arriving one
/// smears along the outline it covered. Each sample carries its own opacity.
///
/// The samples differ only in the window of the stroke they draw, and each
/// window's ends move one way across the exposure, so the first and last
/// samples bound them. Every window holds the part all of them share, so a
/// sample's distance is the nearer of that shared part's, measured once, and
/// that of the stretch only its own window adds at either end. A point the
/// shared part already covers is covered by every sample, and one no added
/// stretch can reach is covered by every sample as the shared part covers it.
fn annotation_shape_exposure(probe: vec2<f32>, annotation: PreviewArrow, feather: f32) -> f32 {
  let count = annotation.sample_count;
  let first = annotation_samples[annotation.sample_first].geometry;
  let last = annotation_samples[annotation.sample_first + count - 1u].geometry;
  var overlap = first;
  overlap.low = max(first.low, last.low);
  overlap.high = min(first.high, last.high);
  var opacity = 0.0;
  for (var tap = 0u; tap < count; tap++) {
    opacity += annotation_samples[annotation.sample_first + tap].opacity;
  }
  if (overlap.low > overlap.high) {
    // The windows share nothing, so each sample is measured whole.
    var total = 0.0;
    for (var tap = 0u; tap < count; tap++) {
      let sample = annotation_samples[annotation.sample_first + tap];
      total += annotation_edge(annotation_shape_distance(probe, sample.geometry), feather) *
          sample.opacity;
    }
    return total / f32(count);
  }
  let overlap_distance = annotation_shape_distance(probe, overlap);
  let overlap_coverage = annotation_edge(overlap_distance, feather);
  if (overlap_coverage >= 1.0) {
    return opacity / f32(count);
  }
  let low = vec2<f32>(first.ax, first.ay);
  let high = vec2<f32>(first.bx, first.by);
  let half_size = (high - low) * 0.5;
  let rounding = max(min(first.rounding, min(half_size.x, half_size.y)), 0.0);
  let lap = annotation_shape_perimeter(half_size, rounding);
  let local = probe - (low + high) * 0.5;
  let reach = annotation_shape_reach(first, 0.0, feather);
  let leading = annotation_shape_stretch_near(
      local, first, half_size, rounding, lap,
      vec2<f32>(min(first.low, last.low), overlap.low) * first.cy, reach);
  let trailing = annotation_shape_stretch_near(
      local, first, half_size, rounding, lap,
      vec2<f32>(overlap.high, max(first.high, last.high)) * first.cy, reach);
  if (!leading && !trailing) {
    return overlap_coverage * opacity / f32(count);
  }
  var total = 0.0;
  for (var tap = 0u; tap < count; tap++) {
    let sample = annotation_samples[annotation.sample_first + tap];
    var distance = overlap_distance;
    if (leading && sample.geometry.low < overlap.low) {
      var stretch = sample.geometry;
      stretch.high = overlap.low;
      distance = min(distance, annotation_shape_distance(probe, stretch));
    }
    if (trailing && sample.geometry.high > overlap.high) {
      var stretch = sample.geometry;
      stretch.low = overlap.high;
      distance = min(distance, annotation_shape_distance(probe, stretch));
    }
    total += annotation_edge(distance, feather) * sample.opacity;
  }
  return total / f32(count);
}

/// Where a shape's stroke can reach: within the pen and its wander of the
/// clean outline. Every exposure sample shares the box and the wander, so
/// this covers all of them.
fn annotation_shape_bounds(shape: PreviewGeometry, halo: f32, feather: f32) -> vec4<f32> {
  let reach = annotation_shape_reach(shape, halo, feather);
  return vec4<f32>(vec2<f32>(shape.ax, shape.ay) - reach, vec2<f32>(shape.bx, shape.by) + reach);
}

fn annotation_shape_reach(shape: PreviewGeometry, halo: f32, feather: f32) -> f32 {
  return shape.width * 0.5 + abs(shape.start_head_ax) +
      max(abs(shape.start_head_ay), abs(shape.end_head_ax)) + halo + feather + 1.0;
}

/// One shape: its stroke in the annotation's own colour.
fn annotation_shape_layer(rgba_in: vec4<f32>, annotation: PreviewArrow, color: vec4<f32>,
                          canvas_point: vec2<f32>, feather: f32, halo: f32) -> vec4<f32> {
  let shape = annotation.geometry;
  if (annotation_outside(canvas_point, annotation_shape_bounds(shape, halo, feather))) {
    return rgba_in;
  }
  let reach = annotation_shape_reach(shape, halo, feather);
  let low = vec2<f32>(shape.ax, shape.ay);
  let high = vec2<f32>(shape.bx, shape.by);
  let half_size = (high - low) * 0.5;
  let rounding = max(min(shape.rounding, min(half_size.x, half_size.y)), 0.0);
  let q = abs(canvas_point - (low + high) * 0.5) - half_size + rounding;
  if (abs(length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - rounding) > reach) {
    return rgba_in;
  }
  // A still frame draws the prepared stroke directly, its opacity already
  // folded into the colour; a moving one averages it over the exposure, and
  // measures the stroke as prepared only for a halo.
  let still = annotation.sample_count == 0u;
  var distance = 1e20;
  if (still || halo > 0.0) {
    distance = annotation_shape_distance(canvas_point, shape);
  }
  let rgba = annotation_halo(rgba_in, color, distance, halo, feather);
  var coverage: f32;
  if (still) {
    coverage = annotation_edge(distance, feather);
  } else {
    coverage = annotation_shape_exposure(canvas_point, annotation, feather);
  }
  if (coverage <= 0.0) {
    return rgba;
  }
  return annotation_over(rgba, color.rgb, coverage * color.a);
}
