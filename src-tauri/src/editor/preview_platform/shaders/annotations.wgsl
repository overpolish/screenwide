// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Annotations. Geometry arrives already prepared in canvas pixels -
// `annotations::arrow::geometry::prepare_arrow` and each kind's own
// `geometry` module on the Rust side, which the macOS chrome picks through -
// so drawing and picking measure the same distances against the same
// numbers.
//
// The shader that includes this defines `annotation_magnify_placement`,
// `annotation_magnify_fetch`, `annotation_cursor_sample` and
// `annotation_spotlight_blur` over its own source.

// Every member is a scalar, so each element is packed exactly like its Rust
// twin in `compositor/arrows.rs`, which pins the offsets.
struct PreviewGeometry {
  ax: f32, ay: f32,
  bx: f32, by: f32,
  cx: f32, cy: f32,
  width: f32, low: f32, high: f32,
  start_head_ax: f32, start_head_ay: f32,
  start_head_bx: f32, start_head_by: f32,
  start_head_cx: f32, start_head_cy: f32,
  end_head_ax: f32, end_head_ay: f32,
  end_head_bx: f32, end_head_by: f32,
  end_head_cx: f32, end_head_cy: f32,
  rounding: f32,
  head: u32,
}

struct PreviewArrow {
  geometry: PreviewGeometry,
  red: f32, green: f32, blue: f32, alpha: f32,
  hover: f32,
  sample_first: u32, sample_count: u32,
  kind: u32,
  flags: u32,
  params: array<f32, 3>,
  data_offset: u32, data_count: u32,
}

/// The annotation part way through the interval this frame covers, and how
/// solid it was then.
struct PreviewSample {
  geometry: PreviewGeometry,
  opacity: f32,
}

@group(0) @binding(7) var<storage, read> annotation_arrows: array<PreviewArrow>;
@group(0) @binding(8) var<storage, read> annotation_samples: array<PreviewSample>;
/// The counters' numbers and text boxes' type, rasterised at the size they
/// are drawn and stacked into one texture.
@group(0) @binding(9) var annotation_numbers: texture_2d<f32>;
@group(0) @binding(10) var<storage, read> annotation_points: array<vec2<f32>>;
/// Text, four bytes to an element: byte `i` is `text[i >> 2] >> ((i & 3) * 8)`.
@group(0) @binding(11) var<storage, read> annotation_text: array<u32>;

/// The halo's opacity, matching the ruler's.
const annotation_hover_alpha: f32 = 0.24;

// How many parameters the curve is sampled at before the best one is
// refined, and how many golden-section steps refine it.
const annotation_curve_samples: u32 = 16u;
const annotation_refine_steps: u32 = 10u;

fn annotation_color(annotation: PreviewArrow) -> vec4<f32> {
  return vec4<f32>(annotation.red, annotation.green, annotation.blue, annotation.alpha);
}

/// Bounds that hold nothing: no point lies inside them.
const annotation_no_bounds: vec4<f32> = vec4<f32>(1e30, 1e30, -1e30, -1e30);
/// Bounds that hold everything, for what reaches the whole canvas.
const annotation_all_bounds: vec4<f32> = vec4<f32>(-1e30, -1e30, 1e30, 1e30);

/// Whether `point` lies outside `bounds`, low in `xy` and high in `zw`. Every
/// layer turns a pixel away here before measuring anything, and the tiles
/// are binned against the same bounds, so the two always agree.
fn annotation_outside(point: vec2<f32>, bounds: vec4<f32>) -> bool {
  return any(point < bounds.xy) || any(point > bounds.zw);
}

/// `a` and `b` grown to hold each other.
fn annotation_union(a: vec4<f32>, b: vec4<f32>) -> vec4<f32> {
  return vec4<f32>(min(a.xy, b.xy), max(a.zw, b.zw));
}

/// Lays `color` over `rgba` with `alpha` of coverage.
fn annotation_over(rgba: vec4<f32>, color: vec3<f32>, alpha: f32) -> vec4<f32> {
  return vec4<f32>(color * alpha + rgba.rgb * (1.0 - alpha), alpha + rgba.a * (1.0 - alpha));
}

/// The halo a hovered annotation wears, hugging its edge `distance` from the
/// edge outwards, the ruler's way.
fn annotation_halo(rgba: vec4<f32>, color: vec4<f32>, distance: f32, halo: f32,
                   feather: f32) -> vec4<f32> {
  if (halo <= 0.0) {
    return rgba;
  }
  let band = (1.0 - annotation_edge(distance, feather)) * annotation_edge(distance - halo, feather);
  let alpha = band * color.a * annotation_hover_alpha;
  if (alpha <= 0.0) {
    return rgba;
  }
  return annotation_over(rgba, color.rgb, alpha);
}

fn annotation_curve_point(a: vec2<f32>, leg: vec2<f32>, bend: vec2<f32>, t: f32) -> vec2<f32> {
  return a + (leg * 2.0 + bend * t) * t;
}

fn annotation_curve_squared(a: vec2<f32>, leg: vec2<f32>, bend: vec2<f32>, probe: vec2<f32>,
                            t: f32) -> f32 {
  let offset = annotation_curve_point(a, leg, bend, t) - probe;
  return dot(offset, offset);
}

/// The distance from `probe` to the quadratic Bezier `a, b, c`, with the
/// curve parameter restricted to `[low, high]`, and the parameter that
/// distance is at.
///
/// The refinement never trusts a derivative: the closed form scales by
/// 1 / |a - 2b + c|², which vanishes for the nearly straight arrow a plain
/// drag makes, and Newton on the derivative walks towards a maximum on a
/// tight bend. So golden-section on the squared distance itself, inside one
/// sample step of the best sample, with Newton only as a polish that has to
/// prove it shortened the distance.
fn annotation_curve_nearest(probe: vec2<f32>, a: vec2<f32>, b: vec2<f32>, c: vec2<f32>, low: f32,
                            high: f32) -> vec2<f32> {
  let leg = b - a;
  let bend = a - 2.0 * b + c;
  let span = max(high - low, 0.0);
  let step_size = span / f32(annotation_curve_samples);
  var best = low;
  var best_squared = 1e30;
  for (var index = 0u; index <= annotation_curve_samples; index++) {
    let t = low + step_size * f32(index);
    let squared = annotation_curve_squared(a, leg, bend, probe, t);
    if (squared < best_squared) {
      best_squared = squared;
      best = t;
    }
  }
  var left = max(best - step_size, low);
  var right = min(best + step_size, high);
  let golden = 0.6180339887;
  var inner_left = right - (right - left) * golden;
  var inner_right = left + (right - left) * golden;
  var left_squared = annotation_curve_squared(a, leg, bend, probe, inner_left);
  var right_squared = annotation_curve_squared(a, leg, bend, probe, inner_right);
  for (var refine = 0u; refine < annotation_refine_steps; refine++) {
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
  var t = select(inner_right, inner_left, left_squared < right_squared);
  var refined = min(left_squared, right_squared);
  for (var polish = 0u; polish < 2u; polish++) {
    let offset = annotation_curve_point(a, leg, bend, t) - probe;
    let tangent = leg * 2.0 + bend * (2.0 * t);
    let slope = dot(tangent, tangent) + 2.0 * dot(offset, bend);
    if (abs(slope) < 1e-6) {
      break;
    }
    let next = t - dot(offset, tangent) / slope;
    if (next < left || next > right) {
      break;
    }
    let squared = annotation_curve_squared(a, leg, bend, probe, next);
    if (squared >= refined) {
      break;
    }
    refined = squared;
    t = next;
  }
  if (best_squared < refined) {
    return vec2<f32>(sqrt(best_squared), best);
  }
  return vec2<f32>(sqrt(refined), t);
}

/// The distance from `probe` to the quadratic Bezier `a, b, c`, with the
/// curve parameter restricted to `[low, high]`.
fn annotation_curve_distance(probe: vec2<f32>, a: vec2<f32>, b: vec2<f32>, c: vec2<f32>,
                             low: f32, high: f32) -> f32 {
  return annotation_curve_nearest(probe, a, b, c, low, high).x;
}

/// Signed distance to a triangle: negative inside it.
fn annotation_triangle_distance(probe: vec2<f32>, a: vec2<f32>, b: vec2<f32>, c: vec2<f32>) -> f32 {
  let edge_ab = b - a;
  let edge_bc = c - b;
  let edge_ca = a - c;
  let to_a = probe - a;
  let to_b = probe - b;
  let to_c = probe - c;
  let nearest_ab = to_a - edge_ab * clamp(dot(to_a, edge_ab) / max(dot(edge_ab, edge_ab), 1e-6),
                                          0.0, 1.0);
  let nearest_bc = to_b - edge_bc * clamp(dot(to_b, edge_bc) / max(dot(edge_bc, edge_bc), 1e-6),
                                          0.0, 1.0);
  let nearest_ca = to_c - edge_ca * clamp(dot(to_c, edge_ca) / max(dot(edge_ca, edge_ca), 1e-6),
                                          0.0, 1.0);
  let winding = sign(edge_ab.x * edge_ca.y - edge_ab.y * edge_ca.x);
  let distances = min(
      min(vec2<f32>(dot(nearest_ab, nearest_ab), winding * (to_a.x * edge_ab.y - to_a.y * edge_ab.x)),
          vec2<f32>(dot(nearest_bc, nearest_bc), winding * (to_b.x * edge_bc.y - to_b.y * edge_bc.x))),
      vec2<f32>(dot(nearest_ca, nearest_ca), winding * (to_c.x * edge_ca.y - to_c.y * edge_ca.x)));
  return -sqrt(distances.x) * sign(distances.y);
}

/// Shaft distance in `x` and head distance in `y`. Head vertices and shaft
/// limits were prepared before the draw; this only evaluates distances, and
/// picking uses the same prepared heads.
fn annotation_arrow_distance(probe: vec2<f32>, arrow: PreviewGeometry) -> vec2<f32> {
  let a = vec2<f32>(arrow.ax, arrow.ay);
  let b = vec2<f32>(arrow.bx, arrow.by);
  let c = vec2<f32>(arrow.cx, arrow.cy);
  // An empty window is an annotation that has not started, or one whose head
  // has eaten what was left of its shaft. Either way there is no shaft to draw.
  var result = vec2<f32>(1e20, 1e20);
  if (arrow.high > arrow.low) {
    result.x = annotation_curve_distance(probe, a, b, c, arrow.low, arrow.high) - arrow.width * 0.5;
  }
  if (arrow.head != 0u) {
    result.y = min(result.y, annotation_triangle_distance(probe,
        vec2<f32>(arrow.end_head_ax, arrow.end_head_ay),
        vec2<f32>(arrow.end_head_bx, arrow.end_head_by),
        vec2<f32>(arrow.end_head_cx, arrow.end_head_cy)) - arrow.rounding);
  }
  if (arrow.head == 2u) {
    result.y = min(result.y, annotation_triangle_distance(probe,
        vec2<f32>(arrow.start_head_ax, arrow.start_head_ay),
        vec2<f32>(arrow.start_head_bx, arrow.start_head_by),
        vec2<f32>(arrow.start_head_cx, arrow.start_head_cy)) - arrow.rounding);
  }
  return result;
}

/// How much of a pixel lies inside an edge `distance` canvas pixels away,
/// where `feather` is half a drawn pixel. A linear ramp across that one pixel
/// is the area a straight edge actually covers; `smoothstep` is half again as
/// steep at the edge and leaves visible steps on a 1x display.
fn annotation_edge(distance: f32, feather: f32) -> f32 {
  return saturate(0.5 - distance / (2.0 * feather));
}

/// Where the annotations' type was rasterised: the atlas's size in pixels and
/// how many atlas pixels it holds per canvas pixel.
struct AnnotationTextAtlas {
  size: vec2<u32>,
  scale: f32,
}

/// The atlas's coverage at `texel`, bilinearly filtered and held to the cell
/// from `low` to `high`, so a tap at the cell's edge reads its own
/// transparent margin rather than the next cell over.
fn annotation_atlas_bilinear(texel: vec2<f32>, low: vec2<f32>, high: vec2<f32>) -> f32 {
  let base = texel - 0.5;
  let first = floor(base);
  let blend = base - first;
  let low_texel = vec2<i32>(clamp(first, low, high));
  let high_texel = vec2<i32>(clamp(first + 1.0, low, high));
  let top = mix(textureLoad(annotation_numbers, vec2<i32>(low_texel.x, low_texel.y), 0).a,
                textureLoad(annotation_numbers, vec2<i32>(high_texel.x, low_texel.y), 0).a, blend.x);
  let bottom = mix(textureLoad(annotation_numbers, vec2<i32>(low_texel.x, high_texel.y), 0).a,
                   textureLoad(annotation_numbers, vec2<i32>(high_texel.x, high_texel.y), 0).a,
                   blend.x);
  return mix(top, bottom, blend.y);
}

/// How much of the type in the atlas cell at `origin`, `size` atlas pixels
/// across, covers one drawn pixel centred on `texel` and spanning `footprint`
/// atlas pixels: four filtered taps spread over that span. The atlas holds
/// two to four atlas pixels per drawn pixel, so the taps take in the whole
/// pixel whether the canvas is drawn larger or smaller than its resolution.
fn annotation_atlas_coverage(atlas: AnnotationTextAtlas, origin: vec2<f32>, size: vec2<f32>,
                             texel: vec2<f32>, footprint: f32) -> f32 {
  let low = origin;
  let high = min(origin + size, vec2<f32>(atlas.size)) - 1.0;
  let spread = footprint * 0.25;
  var total = 0.0;
  for (var tap = 0u; tap < 4u; tap++) {
    let offset = vec2<f32>(select(-spread, spread, (tap & 1u) != 0u),
                           select(-spread, spread, (tap >> 1u) != 0u));
    total += annotation_atlas_bilinear(texel + offset, low, high);
  }
  return total * 0.25;
}

/// Coverage of one prepared arrow, feathered over `feather` canvas pixels.
fn annotation_coverage(probe: vec2<f32>, arrow: PreviewGeometry, feather: f32) -> f32 {
  let distances = annotation_arrow_distance(probe, arrow);
  return max(annotation_edge(distances.x, feather), annotation_edge(distances.y, feather));
}

/// Accumulated exposure coverage: the annotation is drawn at every prepared
/// sample between the shutter start and now, so a moving shaft and its head
/// smear along the path they actually travelled while a held end stays sharp.
fn annotation_exposure(probe: vec2<f32>, annotation: PreviewArrow, feather: f32) -> f32 {
  var total = 0.0;
  for (var tap = 0u; tap < annotation.sample_count; tap++) {
    let sample = annotation_samples[annotation.sample_first + tap];
    total += annotation_coverage(probe, sample.geometry, feather) * sample.opacity;
  }
  return total / f32(annotation.sample_count);
}

/// The ink a label carries on `color`: whichever of black or white reads on
/// it. Luminance rather than a fixed white, because the palette runs from
/// yellow to near-black.
fn annotation_label_tint(color: vec3<f32>) -> vec3<f32> {
  let luminance = dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
  return select(vec3<f32>(1.0), vec3<f32>(0.0), luminance > 0.6);
}

/// A hovered redaction's halo. The box itself was applied to the source and
/// is not drawn here, so this is the only thing that finds an erased box on
/// its own surface: it is white round a dark fill and black round a light
/// one, and stronger than the halo a coloured shape wears in its own colour.
/// The twin of the halo `annotation_redact_halo` draws.
const annotation_redact_halo_alpha: f32 = 0.5;

fn annotation_redact_halo(rgba: vec4<f32>, annotation: PreviewArrow, canvas_point: vec2<f32>,
                          feather: f32) -> vec4<f32> {
  let halo = max(annotation.hover, 0.0);
  if (halo <= 0.0) {
    return rgba;
  }
  let box = annotation.geometry;
  if (annotation_outside(canvas_point, annotation_redact_halo_bounds(annotation, feather))) {
    return rgba;
  }
  let low = vec2<f32>(box.ax, box.ay);
  let high = vec2<f32>(box.bx, box.by);
  let rounding = min(box.rounding, min(high.x - low.x, high.y - low.y) * 0.5);
  let q = abs(canvas_point - (low + high) * 0.5) - ((high - low) * 0.5 - rounding);
  let distance = length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - rounding;
  let band = (1.0 - annotation_edge(distance, feather)) * annotation_edge(distance - halo, feather);
  let luminance = dot(vec3<f32>(annotation.red, annotation.green, annotation.blue),
                      vec3<f32>(0.2126, 0.7152, 0.0722));
  let tone = select(vec3<f32>(1.0), vec3<f32>(0.0), luminance > 0.5);
  return annotation_over(rgba, tone, band * annotation_redact_halo_alpha);
}

/// Where a redaction's halo can reach: nowhere while it is not hovered.
fn annotation_redact_halo_bounds(annotation: PreviewArrow, feather: f32) -> vec4<f32> {
  let halo = max(annotation.hover, 0.0);
  if (halo <= 0.0) {
    return annotation_no_bounds;
  }
  let box = annotation.geometry;
  let reach = halo + feather + 1.0;
  return vec4<f32>(vec2<f32>(box.ax, box.ay) - reach, vec2<f32>(box.bx, box.by) + reach);
}

/// Draws one spotlight's hover halo or one magnifier, the kinds that act on
/// the picture, over `rgba`.
fn annotation_picture_layer(rgba: vec4<f32>, annotation: PreviewArrow, canvas_point: vec2<f32>,
                            feather: f32, cursor: bool) -> vec4<f32> {
  if (annotation.kind == annotation_magnify_kind) {
    return annotation_magnify_layer(rgba, annotation, canvas_point, feather,
                                    max(annotation.hover, 0.0), cursor);
  }
  return annotation_redact_halo(rgba, annotation, canvas_point, feather);
}

/// Draws one prepared arrow over `rgba`.
fn annotation_arrow_layer(rgba_in: vec4<f32>, annotation: PreviewArrow, color: vec4<f32>,
                          canvas_point: vec2<f32>, feather: f32, halo: f32) -> vec4<f32> {
  let arrow = annotation.geometry;
  if (annotation_outside(canvas_point, annotation_arrow_bounds(arrow, halo, feather))) {
    return rgba_in;
  }
  let distances = annotation_arrow_distance(canvas_point, arrow);
  let rgba = annotation_halo(rgba_in, color, min(distances.x, distances.y), halo, feather);
  // A still frame draws the prepared arrow directly; a moving one averages
  // the arrow over the exposure, head and shaft together.
  var coverage: f32;
  if (annotation.sample_count == 0u) {
    coverage = max(annotation_edge(distances.x, feather), annotation_edge(distances.y, feather));
  } else {
    coverage = annotation_exposure(canvas_point, annotation, feather);
  }
  if (coverage <= 0.0) {
    return rgba;
  }
  return annotation_over(rgba, color.rgb, coverage * color.a);
}

/// Where an arrow can reach. The curve lies inside the hull of its three
/// points; the heads reach back from a tip and across it. One rectangle covers
/// all of that, and keeps a long list cheap over most of the canvas.
fn annotation_arrow_bounds(arrow: PreviewGeometry, halo: f32, feather: f32) -> vec4<f32> {
  let a = vec2<f32>(arrow.ax, arrow.ay);
  let b = vec2<f32>(arrow.bx, arrow.by);
  let c = vec2<f32>(arrow.cx, arrow.cy);
  let radius = arrow.width * 0.5;
  let reach = radius * select(1.0, 9.0, arrow.head != 0u) + halo + feather + 1.0;
  return vec4<f32>(min(a, min(b, c)) - reach, max(a, max(b, c)) + reach);
}

/// Draws one prepared annotation over `rgba`, whatever its kind but a
/// highlight, which recolours what is under it in `annotation_highlight_layer`
/// instead, and a spotlight or magnifier, which `annotation_picture_layer`
/// draws.
///
/// Annotations are deliberately not clipped to the crop: an arrow may point in
/// from the padding. `feather` is how wide an edge is smoothed, in canvas
/// pixels, and `number_atlas` is where the counters' numbers and the text
/// boxes' text were rasterised - a zero size where nothing rasterised any.
fn annotation_layer(rgba: vec4<f32>, annotation: PreviewArrow, canvas_point: vec2<f32>,
                    feather: f32, number_atlas: AnnotationTextAtlas) -> vec4<f32> {
  let color = annotation_color(annotation);
  if (annotation.kind == 3u) {
    return annotation_redact_halo(rgba, annotation, canvas_point, feather);
  }
  let halo = max(annotation.hover, 0.0);
  if (color.a <= 0.0 || annotation.geometry.width <= 0.0) {
    return rgba;
  }
  if (annotation.kind == 1u) {
    return annotation_counter_layer(rgba, annotation, color, canvas_point, feather, halo,
                                    number_atlas);
  }
  if (annotation.kind == 2u) {
    return annotation_text_layer(rgba, annotation, color, canvas_point, feather, halo,
                                 number_atlas);
  }
  if (annotation.kind == annotation_shape_kind) {
    return annotation_shape_layer(rgba, annotation, color, canvas_point, feather, halo);
  }
  if (annotation.kind == annotation_draw_kind) {
    return annotation_draw_layer(rgba, annotation, color, canvas_point, feather, halo);
  }
  return annotation_arrow_layer(rgba, annotation, color, canvas_point, feather, halo);
}

/// Whether a kind acts on the picture rather than marking it: a spotlight
/// shades and blurs it, and a magnifier enlarges it. Each covers the marks
/// below it, and lies over the cursor.
fn annotation_acts_on_picture(kind: u32) -> bool {
  return kind == annotation_spotlight_kind || kind == annotation_magnify_kind;
}

// Each shader that includes this file defines `annotation_next(probe, start,
// last)`: the first annotation at or after `start`, and before `last`, that
// may reach `probe`, or `last` where none does. Every pass below walks the
// list through it, in document order. The preview answers from the tile
// `probe` lies in; the live overlay, which draws a handful, answers `start`.

/// Draws every annotation in `[first, last)` but the highlights over `rgba`:
/// the live overlay's pass, which recolours its highlights under everything
/// else in `composite_highlights`. The editor draws in document order through
/// `composite_annotation_layers` instead.
fn composite_annotations(rgba_in: vec4<f32>, canvas_point: vec2<f32>, first: u32, last: u32,
                         feather: f32, number_atlas: AnnotationTextAtlas) -> vec4<f32> {
  var rgba = rgba_in;
  for (var index = annotation_next(canvas_point, first, last); index < last;
       index = annotation_next(canvas_point, index + 1u, last)) {
    let annotation = annotation_arrows[index];
    if (annotation.kind == annotation_highlight_kind) {
      continue;
    }
    if (annotation_acts_on_picture(annotation.kind)) {
      rgba = annotation_picture_layer(rgba, annotation, canvas_point, feather, false);
    } else {
      rgba = annotation_layer(rgba, annotation, canvas_point, feather, number_atlas);
    }
  }
  return rgba;
}
