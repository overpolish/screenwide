// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

/// The highlight: every band of every highlight on one layer, recoloured
/// rather than painted over. The twin of `annotation_highlight.hlsl`.
///
/// A highlight reads the pixel under it - `base` - and maps the page to the
/// highlight's colour and the ink printed on it to a colour that reads on
/// that, measured against the tone the selection read. Opacity alone vanishes
/// on a dark page; this does not care which way round the page is.
///
/// Its bands ride in the side buffer in source pixels, placed here by the
/// record's `a` (where the source's origin lands) and `b` (one source pixel's
/// reach). The reveal draws them in reading order, each line over its own
/// share of it and starting a little after the line before, the way a hand
/// goes back for the next line.
#define GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_HIGHLIGHT @R"METAL(
constant uint annotation_highlight_kind = 4u;
constant uint annotation_highlight_hand_drawn = 1u << 6;

static uint highlight_hash(uint value) {
  value ^= value >> 16;
  value *= 0x7feb352du;
  value ^= value >> 15;
  value *= 0x846ca68bu;
  value ^= value >> 16;
  return value;
}

/// A number in [0, 1) that only `seed` and `salt` decide.
static float highlight_random(uint seed, uint salt) {
  return float(highlight_hash(seed ^ highlight_hash(salt)) & 0xffffffu) / 16777216.0;
}

/// Smooth value noise along one axis, in [0, 1].
static float highlight_noise(float at, uint seed) {
  float cell = floor(at);
  float along = at - cell;
  uint index = as_type<uint>(int(cell));
  float from = highlight_random(seed, index);
  float to = highlight_random(seed, index + 1u);
  return mix(from, to, along * along * (3.0 - 2.0 * along));
}

/// The distance to a box from `low` to `high` whose left and right ends are
/// rounded by radii of their own: a clean band's ends are squarer than the
/// round tip it draws itself in with.
static float highlight_box(float2 point, float2 low, float2 high, float left_radius,
                           float right_radius) {
  float2 centre = (low + high) * 0.5;
  float2 half_size = max((high - low) * 0.5, float2(0.0));
  float radius = min(point.x < centre.x ? left_radius : right_radius,
                     min(half_size.x, half_size.y));
  float2 q = abs(point - centre) - (half_size - radius);
  return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - radius;
}

/// A marker stroke drawn by hand along the band from `low` to `high`, drawn in
/// from `from` to `to`: a little off level, its chisel ends slanted by
/// `slant`, its edges wavering, and its settled ends running a little long.
/// `settled` is how far each end has settled, 0 while a reveal is still
/// moving it and 1 once it has arrived, so a moving tip reaches exactly as far
/// as it has drawn.
///
/// `shared` is whether its top and its bottom lie under a neighbouring band,
/// as the strokes laid over a box do. A shared edge runs straight and the
/// stroke keeps level, so no page shows between neighbours; the edge is under
/// the neighbour anyway.
static float highlight_stroke(float2 point, float2 low, float2 high, float from, float to,
                              float2 settled, float slant, uint seed, float2 shared) {
  float band = high.y - low.y;
  // Turned about the band's own centre, not the part drawn so far, so a
  // stroke drawing itself in does not swing as it grows. The turn is held to
  // what moves its ends a tenth of its height, so a long line stays on the
  // text it covers.
  float2 centre = (low + high) * 0.5;
  float free_hand = 1.0 - max(shared.x, shared.y);
  float most = min(0.026, band * 0.1 / max((high.x - low.x) * 0.5, 1.0));
  float tilt = (highlight_random(seed, 1u) * 2.0 - 1.0) * most * free_hand;
  float2 offset = point - centre;
  float2 level = float2(offset.x * cos(tilt) + offset.y * sin(tilt),
                        offset.y * cos(tilt) - offset.x * sin(tilt));
  level.y -= (highlight_random(seed, 2u) - 0.5) * 0.12 * band * free_hand;
  float along = level.x + level.y * slant + centre.x;
  // A settled end only ever runs long: one that fell short would leave the
  // edge of a word unmarked. It runs long as the stroke reaches it, so the
  // draw lands on its end rather than jumping out to it.
  float left = from - settled.x * (0.05 + highlight_random(seed, 3u) * 0.25) * band;
  float right = to + settled.y * (0.05 + highlight_random(seed, 4u) * 0.3) * band;
  float wave = max(band * 2.6, 1.0);
  float top = -band * 0.5 +
      band * 0.07 * (1.0 - shared.x) * (highlight_noise(along / wave, seed ^ 5u) * 2.0 - 1.0);
  float bottom = band * 0.5 +
      band * 0.07 * (1.0 - shared.y) * (highlight_noise(along / wave, seed ^ 6u) * 2.0 - 1.0);
  float radius = band * 0.16;
  float2 outside = float2(max(left - along, along - right), max(top - level.y, level.y - bottom)) + radius;
  return length(max(outside, 0.0)) + min(max(outside.x, outside.y), 0.0) - radius;
}

/// `base` recoloured under a highlight in `colour`: the page becomes the
/// highlight's colour and its ink - whatever stands out from the page - a
/// colour that reads on it. The ink keeps its hue, pushed dark on a light
/// highlight and light on a dark one. `tone` is the page's luminance and its
/// ink's.
static float3 highlight_recolour(float3 base, float3 colour, float2 tone) {
  const float3 weights = float3(0.2126, 0.7152, 0.0722);
  float luminance = dot(base, weights);
  float span = tone.y - tone.x;
  // No page was read under it - a photo, a gradient, a box laid by hand - so
  // there is nothing to map from, and the highlight tints what is there. Over
  // light it multiplies, well short of fully, as a felt marker's ink does on
  // paper. Multiplied, dark would stay dark and the highlight would vanish, so
  // dark is lifted a quarter of the way to the colour instead. Each pixel is weighed by its own
  // brightness, so a box over mixed content needs no reading of it; dark text
  // on a light page is lifted with the rest of the dark, to a deep shade of
  // the colour that still reads against it.
  if (abs(span) < 0.01) {
    float dark = 1.0 - saturate((luminance - 0.2) / 0.4);
    float3 laid = base * mix(float3(1.0, 1.0, 1.0), colour, 0.7);
    float3 lifted = base + colour * (1.0 - base) * 0.25;
    return mix(laid, lifted, dark);
  }
  if (abs(span) < 0.2) span = span < 0.0 ? -0.2 : 0.2;
  float ink_share = smoothstep(0.04, 0.55, saturate((luminance - tone.x) / span));
  float3 ink = dot(colour, weights) > 0.5
      ? base * min(1.0, 0.2 / max(luminance, 1e-3))
      : 1.0 - (1.0 - base) * min(1.0, 0.15 / max(1.0 - luminance, 1e-3));
  return mix(colour, saturate(ink), ink_share);
}

/// Every highlight on the layer `above_camera` names, over `rgba`, each
/// recolouring `base`. The canvas passes hand the same pixel as both; the live
/// overlay's is transparent, and `base` is the desktop captured under it.
static float4 composite_highlights(
    float4 rgba, float4 base, const device AnnotationUniforms *annotations, uint count,
    uint above_camera, float2 point, float pixel_scale,
    const device packed_float2 *points) {
  float feather = max(pixel_scale, 1e-4) * 0.5;
  for (uint index = 0; index < count; ++index) {
    const device AnnotationUniforms &annotation = annotations[index];
    if (annotation.kind != annotation_highlight_kind ||
        annotation.above_camera != above_camera)
      continue;
    float4 colour = float4(annotation.color);
    uint pairs = annotation.data_count / 2u;
    if (colour.a <= 0.0 || pairs == 0u) continue;
    float2 origin = float2(annotation.arrow.a);
    float2 unit = float2(annotation.arrow.b);
    float2 tone = float2(annotation.arrow.c);
    bool hand = (annotation.flags & annotation_highlight_hand_drawn) != 0u;
    uint seed = uint(annotation.params[0]) | (uint(annotation.params[1]) << 16);
    // One hand holds the marker for the whole highlight, so its chisel leans
    // the same way on every line, which way being the seed's. How far is each
    // line's own: the hand is lifted and put down again between lines.
    float lean = highlight_random(seed, 8u) < 0.5 ? -1.0 : 1.0;
    // Each line draws itself in over its own share of the reveal, starting a
    // little after the line before it, so a box of many strokes goes down one
    // after another at a hand's pace rather than as one racing line.
    float lines = float(pairs);
    float share = min(1.0, max(0.35, 2.0 / (lines + 1.0)));
    float spacing = pairs > 1u ? (1.0 - share) / (lines - 1.0) : 0.0;
    float halo = max(annotation.hover, 0.0);
    float coverage = 0.0, nearest = 1e20;
    for (uint band = 0u; band < pairs; ++band) {
      uint at = annotation.data_offset + band * 2u;
      float2 low = origin + float2(points[at]) * unit;
      float2 high = origin + float2(points[at + 1u]) * unit;
      float span = max(high.x - low.x, 0.0);
      float begin = float(band) * spacing;
      float from = span * saturate((annotation.arrow.low - begin) / share);
      float to = span * saturate((annotation.arrow.high - begin) / share);
      float height = high.y - low.y;
      if (to <= from || height <= 0.0) continue;
      float reach = height + halo + feather + 1.0;
      if (point.y < low.y - reach || point.y > high.y + reach ||
          point.x < low.x + from - reach || point.x > low.x + to + reach)
        continue;
      // How far each end has settled: a reveal's moving tip is round, and it
      // eases into the end's own shape over the last line height it travels.
      float2 settled = saturate(1.0 - float2(from, span - to) / max(height, 1.0));
      // Whether its top and its bottom lie under a neighbouring band, as the
      // strokes laid over a box do.
      float2 shared = float2(
          band > 0u && origin.y + float2(points[at - 1u]).y * unit.y > low.y ? 1.0 : 0.0,
          band + 1u < pairs && origin.y + float2(points[at + 2u]).y * unit.y < high.y ? 1.0
                                                                                      : 0.0);
      float edge;
      if (hand) {
        uint stroke = seed ^ (band * 0x9e3779b9u);
        float slant = lean * (0.08 + highlight_random(stroke, 10u) * 0.32);
        edge = highlight_stroke(point, low, high, low.x + from, low.x + to, settled, slant,
                                stroke, shared);
      } else {
        // The drawn end of a band still being drawn is the marker's round tip.
        // Settled, a corner on an edge shared with a neighbour is square, so a
        // box laid in strokes has no notch where two of them meet.
        // A settled corner is rounded as a one-line text box's is, as a share
        // of its height: `CORNER` over `LINE_HEIGHT` and twice `PAD_Y`, in
        // `editor/annotations/text/geometry.rs` and `text/metrics.rs`.
        float tip = height * 0.5;
        float square = height * 0.21 *
            (1.0 - (point.y < (low.y + high.y) * 0.5 ? shared.x : shared.y));
        edge = highlight_box(point, float2(low.x + from, low.y), float2(low.x + to, high.y),
                             mix(tip, square, settled.x), mix(tip, square, settled.y));
      }
      coverage = max(coverage, annotation_edge(edge, feather));
      nearest = min(nearest, edge);
    }
    if (halo > 0.0 && nearest < 1e19) {
      // The ruler's hover halo, hugging the bands from the edge outwards.
      float ring = (1.0 - annotation_edge(nearest, feather)) *
          annotation_edge(nearest - halo, feather);
      float alpha = ring * colour.a * annotation_hover_alpha;
      rgba.rgb = colour.rgb * alpha + rgba.rgb * (1.0 - alpha);
      rgba.a = alpha + rgba.a * (1.0 - alpha);
    }
    if (coverage <= 0.0) continue;
    float alpha = coverage * colour.a;
    rgba.rgb = highlight_recolour(base.rgb, colour.rgb, tone) * alpha + rgba.rgb * (1.0 - alpha);
    rgba.a = alpha + rgba.a * (1.0 - alpha);
  }
  return rgba;
}
)METAL"
