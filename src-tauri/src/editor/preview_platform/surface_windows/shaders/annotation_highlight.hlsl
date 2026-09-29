// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The highlight, the twin of
// `gpu_compositor_macos_shader_source_annotation_highlight.h`: every band of
// every highlight in a run, recoloured rather than painted over. Included by
// `annotations.hlsl`, after the pieces every kind shares.
//
// A highlight reads the pixel under it - `base` - and maps the page to the
// highlight's colour and the ink printed on it to a colour that reads on that,
// measured against the tone the selection read. Its bands ride in
// `annotation_points` in source pixels, placed by the prepared record's `a`
// (where the source's origin lands) and `b` (one source pixel's reach); `c` is
// the tone and `low` and `high` its reveal window, which each line draws over
// its own share of, starting a little after the line before. A highlight that
// moved since the shutter opened carries exposure samples, each with its own
// window, and each line is averaged over them, so its drawing end smears.

static const uint annotation_highlight_kind = 4u;
static const uint annotation_highlight_hand_drawn = 1u << 6;

uint highlight_hash(uint value) {
  value ^= value >> 16;
  value *= 0x7feb352du;
  value ^= value >> 15;
  value *= 0x846ca68bu;
  value ^= value >> 16;
  return value;
}

/// A number in [0, 1) that only `seed` and `salt` decide.
float highlight_random(uint seed, uint salt) {
  return (float)(highlight_hash(seed ^ highlight_hash(salt)) & 0xffffffu) / 16777216.0;
}

/// Smooth value noise along one axis, in [0, 1].
float highlight_noise(float at, uint seed) {
  float cell = floor(at);
  float along = at - cell;
  uint index = asuint((int)cell);
  float from = highlight_random(seed, index);
  float to = highlight_random(seed, index + 1u);
  return lerp(from, to, along * along * (3.0 - 2.0 * along));
}

/// The distance to a box from `low` to `high` whose left and right ends are
/// rounded by radii of their own.
float highlight_box(float2 probe, float2 low, float2 high, float left_radius, float right_radius) {
  float2 centre = (low + high) * 0.5;
  float2 half_size = max((high - low) * 0.5, float2(0.0, 0.0));
  float radius = min(probe.x < centre.x ? left_radius : right_radius,
                     min(half_size.x, half_size.y));
  float2 q = abs(probe - centre) - (half_size - radius);
  return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - radius;
}

/// A marker stroke drawn by hand along the band from `low` to `high`, drawn in
/// from `from` to `to`, its ends settled as far as `settled` says and slanted
/// by `slant`. `joined` is whether its top and its bottom lie under a
/// neighbouring band: a shared edge runs straight and the stroke keeps level,
/// so no page shows between the strokes laid over a box.
float highlight_stroke(float2 probe, float2 low, float2 high, float from, float to,
                       float2 settled, float slant, uint seed, float2 joined) {
  float band = high.y - low.y;
  // Turned about the band's own centre, so a stroke drawing itself in does not
  // swing as it grows, and by no more than moves its ends a tenth of its
  // height, so a long line stays on the text it covers.
  float2 centre = (low + high) * 0.5;
  float free_hand = 1.0 - max(joined.x, joined.y);
  float most = min(0.026, band * 0.1 / max((high.x - low.x) * 0.5, 1.0));
  float tilt = (highlight_random(seed, 1u) * 2.0 - 1.0) * most * free_hand;
  float2 offset = probe - centre;
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
      band * 0.07 * (1.0 - joined.x) * (highlight_noise(along / wave, seed ^ 5u) * 2.0 - 1.0);
  float bottom = band * 0.5 +
      band * 0.07 * (1.0 - joined.y) * (highlight_noise(along / wave, seed ^ 6u) * 2.0 - 1.0);
  float radius = band * 0.16;
  float2 outside = float2(max(left - along, along - right), max(top - level.y, level.y - bottom)) + radius;
  return length(max(outside, 0.0)) + min(max(outside.x, outside.y), 0.0) - radius;
}

/// `base` recoloured under a highlight in `colour`: the page becomes the
/// highlight's colour and its ink a colour that reads on it, keeping its hue.
float3 highlight_recolour(float3 base, float3 colour, float2 tone) {
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
    float3 laid = base * lerp(float3(1.0, 1.0, 1.0), colour, 0.7);
    float3 lifted = base + colour * (1.0 - base) * 0.25;
    return lerp(laid, lifted, dark);
  }
  if (abs(span) < 0.2) span = span < 0.0 ? -0.2 : 0.2;
  float ink_share = smoothstep(0.04, 0.55, saturate((luminance - tone.x) / span));
  float3 ink = dot(colour, weights) > 0.5
      ? base * min(1.0, 0.2 / max(luminance, 1e-3))
      : 1.0 - (1.0 - base) * min(1.0, 0.15 / max(1.0 - luminance, 1e-3));
  return lerp(colour, saturate(ink), ink_share);
}

/// Where one line `span` long is drawn from and to at the reveal `low` to
/// `high`: it draws itself in over `share` of the reveal, from `begin`.
float2 highlight_window(float low, float high, float begin, float share, float span) {
  return span * saturate((float2(low, high) - begin) / share);
}

/// How far `probe` falls outside the line from `low` to `high` drawn over
/// `window` along it.
float highlight_line(float2 probe, float2 low, float2 high, float2 window, float2 joined,
                     bool hand, uint stroke, float slant) {
  float span = max(high.x - low.x, 0.0);
  float height = high.y - low.y;
  // How far each end has settled: a reveal's moving tip is round, and it
  // eases into the end's own shape over the last line height it travels.
  float2 settled = saturate(1.0 - float2(window.x, span - window.y) / max(height, 1.0));
  if (hand)
    return highlight_stroke(probe, low, high, low.x + window.x, low.x + window.y, settled,
                            slant, stroke, joined);
  // The drawn end of a band still being drawn is the marker's round tip.
  // Settled, a corner on an edge shared with a neighbour is square, so a box
  // laid in strokes has no notch where two of them meet.
  // A settled corner is rounded as a one-line text box's is, as a share of
  // its height: `CORNER` over `LINE_HEIGHT` and twice `PAD_Y`, in
  // `editor/annotations/text/geometry.rs` and `text/metrics.rs`.
  float tip = height * 0.5;
  float square = height * 0.21 * (1.0 - (probe.y < (low.y + high.y) * 0.5 ? joined.x : joined.y));
  return highlight_box(probe, float2(low.x + window.x, low.y), float2(low.x + window.y, high.y),
                       lerp(tip, square, settled.x), lerp(tip, square, settled.y));
}

/// One highlight over `rgba`, recolouring `base`. The editor hands the same
/// pixel as both, so a highlight recolours whatever the document drew under
/// it; the live overlay's `rgba` is transparent, and `base` is the desktop
/// captured under it.
float4 annotation_highlight_layer(float4 rgba, float4 base, PreviewArrow annotation,
                                  float2 canvas_point, float feather) {
  float4 colour = float4(annotation.red, annotation.green, annotation.blue, annotation.alpha);
  uint pairs = annotation.data_count / 2u;
  if (colour.a <= 0.0 || pairs == 0u) return rgba;
  PreviewGeometry record = annotation.geometry;
  float2 origin = float2(record.ax, record.ay);
  float2 unit = float2(record.bx, record.by);
  float2 tone = float2(record.cx, record.cy);
  bool hand = (annotation.flags & annotation_highlight_hand_drawn) != 0u;
  uint seed = (uint)annotation.params[0] | ((uint)annotation.params[1] << 16);
  // One hand holds the marker for the whole highlight, so its chisel leans
  // the same way on every line, which way being the seed's. How far is each
  // line's own: the hand is lifted and put down again between lines.
  float lean = highlight_random(seed, 8u) < 0.5 ? -1.0 : 1.0;
  // Each line draws itself in over its own share of the reveal, starting a
  // little after the line before it.
  float lines = (float)pairs;
  float share = min(1.0, max(0.35, 2.0 / (lines + 1.0)));
  float spacing = pairs > 1u ? (1.0 - share) / (lines - 1.0) : 0.0;
  float halo = max(annotation.hover, 0.0);
  // The exposure's samples run steadily from the shutter opening to now, so
  // the first and the last hold every sample's ends between them, and their
  // opacities' mean is the mean of all of them.
  uint taps = annotation.sample_count;
  PreviewSample opened = {record, 1.0};
  PreviewSample closed = opened;
  if (taps > 0u) {
    opened = annotation_samples[annotation.sample_first];
    closed = annotation_samples[annotation.sample_first + taps - 1u];
  }
  float opacity = (opened.opacity + closed.opacity) * 0.5;
  float coverage = 0.0, nearest = 1e20;
  for (uint row = 0u; row < pairs; ++row) {
    uint at = annotation.data_offset + row * 2u;
    float2 low = origin + annotation_points[at] * unit;
    float2 high = origin + annotation_points[at + 1u] * unit;
    float span = max(high.x - low.x, 0.0);
    float height = high.y - low.y;
    float begin = (float)row * spacing;
    float2 now = highlight_window(record.low, record.high, begin, share, span);
    float2 start = highlight_window(opened.geometry.low, opened.geometry.high, begin, share, span);
    float2 end = highlight_window(closed.geometry.low, closed.geometry.high, begin, share, span);
    float from = min(now.x, min(start.x, end.x));
    float to = max(now.y, max(start.y, end.y));
    if (to <= from || height <= 0.0) continue;
    float reach = height + halo + feather + 1.0;
    if (canvas_point.y < low.y - reach || canvas_point.y > high.y + reach ||
        canvas_point.x < low.x + from - reach || canvas_point.x > low.x + to + reach)
      continue;
    // Whether its top and its bottom lie under a neighbouring band, as the
    // strokes laid over a box do.
    float2 joined = float2(
        row > 0u && origin.y + annotation_points[at - 1u].y * unit.y > low.y ? 1.0 : 0.0,
        row + 1u < pairs && origin.y + annotation_points[at + 2u].y * unit.y < high.y ? 1.0
                                                                                      : 0.0);
    uint stroke = seed ^ (row * 0x9e3779b9u);
    float slant = lean * (0.08 + highlight_random(stroke, 10u) * 0.32);
    if ((taps == 0u || halo > 0.0) && now.y > now.x) {
      float edge = highlight_line(canvas_point, low, high, now, joined, hand, stroke, slant);
      nearest = min(nearest, edge);
      if (taps == 0u) {
        coverage = max(coverage, annotation_edge(edge, feather));
        continue;
      }
    }
    if (taps == 0u) continue;
    // More than two line heights inside every sample's ends, every sample
    // draws this pixel alike, so the last stands for them all.
    float along = canvas_point.x - low.x;
    float margin = height * 2.0 + feather + 1.0;
    float line_coverage = 0.0;
    if (along > max(start.x, end.x) + margin && along < min(start.y, end.y) - margin) {
      float edge = highlight_line(canvas_point, low, high, end, joined, hand, stroke, slant);
      line_coverage = annotation_edge(edge, feather) * opacity;
    } else {
      for (uint tap = 0u; tap < taps; ++tap) {
        PreviewSample sample = annotation_samples[annotation.sample_first + tap];
        float2 window =
            highlight_window(sample.geometry.low, sample.geometry.high, begin, share, span);
        if (window.y <= window.x) continue;
        float edge = highlight_line(canvas_point, low, high, window, joined, hand, stroke, slant);
        line_coverage += annotation_edge(edge, feather) * sample.opacity;
      }
      line_coverage /= (float)taps;
    }
    coverage = max(coverage, line_coverage);
  }
  if (halo > 0.0 && nearest < 1e19) {
    float ring = (1.0 - annotation_edge(nearest, feather)) *
        annotation_edge(nearest - halo, feather);
    float halo_alpha = ring * colour.a * annotation_hover_alpha;
    rgba.rgb = colour.rgb * halo_alpha + rgba.rgb * (1.0 - halo_alpha);
    rgba.a = halo_alpha + rgba.a * (1.0 - halo_alpha);
  }
  if (coverage <= 0.0) return rgba;
  float alpha = coverage * colour.a;
  rgba.rgb = highlight_recolour(base.rgb, colour.rgb, tone) * alpha + rgba.rgb * (1.0 - alpha);
  rgba.a = alpha + rgba.a * (1.0 - alpha);
  return rgba;
}

/// Every highlight in `[first, last)` over `rgba`, each recolouring `base`:
/// the live overlay's pass, which puts its highlights under everything else it
/// draws.
float4 composite_highlights(float4 rgba, float4 base, float2 canvas_point, uint first,
                            uint last, float feather) {
  for (uint index = first; index < last; ++index) {
    PreviewArrow annotation = annotation_arrows[index];
    if (annotation.kind != annotation_highlight_kind) continue;
    rgba = annotation_highlight_layer(rgba, base, annotation, canvas_point, feather);
  }
  return rgba;
}
