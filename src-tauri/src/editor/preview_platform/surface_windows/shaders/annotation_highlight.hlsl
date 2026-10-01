// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The highlight, the twin of
// `gpu_compositor_macos_shader_source_annotation_highlight.h`: every band of
// every highlight in a run, recoloured rather than painted over. Included by
// `annotations.hlsl` after the magnifier, whose picture functions it reads
// each glyph's full ink through.
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
static const uint annotation_highlight_laid_by_hand = 1u << 8;

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

/// Encoded channels in linear light.
float3 highlight_light(float3 encoded) {
  float3 channel = saturate(encoded);
  return lerp(pow((channel + 0.055) / 1.055, 2.4), channel / 12.92, step(channel, 0.04045));
}

/// Linear light as encoded channels.
float3 highlight_encode(float3 light) {
  float3 channel = saturate(light);
  return lerp(1.055 * pow(channel, 1.0 / 2.4) - 0.055, channel * 12.92, step(channel, 0.0031308));
}

/// How far the strongest pixel around `canvas_point` stands out from `page`,
/// in linear light and in the direction ink lies, `toward`: the picture
/// `annotation_magnify_fetch` reads, five by five texels spread over a reach
/// that grows with the line's height, `line_height` canvas pixels. Negative
/// where there is no picture. The twin of the Metal `highlight_peak`.
float highlight_peak(float2 canvas_point, float line_height, float page, float toward) {
  AnnotationMagnifyPlacement at = annotation_magnify_placement();
  if (any(at.image.zw <= 0.0) || any(at.size < 1.0)) return -1.0;
  const float3 weights = float3(0.2126, 0.7152, 0.0722);
  float2 per = at.size / at.image.zw;
  float2 centre = (canvas_point - at.image.xy) * per;
  float spread = clamp(line_height * per.y * 0.12, 2.0, 8.0) * 0.5;
  float peak = 0.0;
  for (int y = -2; y <= 2; ++y)
    for (int x = -2; x <= 2; ++x) {
      float2 probe = floor(centre + float2(x, y) * spread);
      int2 texel = int2(clamp(probe, at.texels.xy, at.texels.zw));
      float3 lit = highlight_light(annotation_magnify_fetch(texel).rgb);
      peak = max(peak, toward * (dot(lit, weights) - page));
    }
  return peak;
}

/// The picture at canvas point `q`, encoded; past what the canvas shows, the
/// edge the crop left carries on.
float3 highlight_fetch(float2 q) {
  AnnotationMagnifyPlacement at = annotation_magnify_placement();
  float2 per = at.size / at.image.zw;
  int2 texel = int2(clamp(floor((q - at.image.xy) * per), at.texels.xy, at.texels.zw));
  return annotation_magnify_fetch(texel).rgb;
}

/// Whether `rgb` stands out from `page` towards the ink by at least half of
/// `own`, eased in from three tenths of it.
float highlight_inked(float3 rgb, float page, float toward, float own) {
  float stands = toward * (dot(highlight_light(rgb), float3(0.2126, 0.7152, 0.0722)) - page);
  return smoothstep(0.3, 0.6, stands / own);
}

/// How surely `canvas_point` lies on a fill rather than on type, or on the
/// type printed on one. Where the band's margins above and below, `rows`
/// past its line, are both one fill, the column is decided whole: what lies
/// between them is tinted, and past them what is the fill's own colour.
/// Otherwise ink in one margin; the pixel itself standing out, or ink to its
/// left and right; and the fill's colour most of what lies around it. The
/// twin of the Metal `highlight_fill`, which says why each.
float highlight_fill(float2 canvas_point, float2 rows, float page, float toward, float stands,
                     float faint) {
  AnnotationMagnifyPlacement at = annotation_magnify_placement();
  float height = rows.y - rows.x;
  if (any(at.image.zw <= 0.0) || any(at.size < 1.0) || height <= 0.0) return 0.0;
  float own = max(stands, faint);
  float2 span = rows.x + height * float2(0.06, 0.94);
  float2 margin = float2(1.0, 1.0);
  float2 centre = float2(0.0, 0.0), beside = float2(0.0, 0.0);
  float3 above = float3(0.0, 0.0, 0.0), below = float3(0.0, 0.0, 0.0);
  for (int side = -1; side <= 1; ++side) {
    float x = canvas_point.x + (float)side * height * 0.1;
    float3 top = highlight_fetch(float2(x, span.x));
    float3 bottom = highlight_fetch(float2(x, span.y));
    margin *= float2(highlight_inked(top, page, toward, own),
                     highlight_inked(bottom, page, toward, own));
    float2 inked = float2(highlight_inked(top, page, toward, faint),
                          highlight_inked(bottom, page, toward, faint));
    if (side == 0) {
      centre = inked;
      above = top;
      below = bottom;
    } else {
      beside = max(beside, inked);
    }
  }
  float both = min(centre.x * beside.x, centre.y * beside.y);
  if (both > 0.0 && all(abs(above - below) < 0.12)) {
    if (canvas_point.y >= span.x && canvas_point.y <= span.y) return both;
    return all(abs(highlight_fetch(canvas_point) - above) < 0.12) ? both : 0.0;
  }
  float reaches = max(margin.x, margin.y);
  if (reaches <= 0.0) return 0.0;
  float enclosed = 1.0;
  if (stands <= faint) {
    float2 sides = float2(0.0, 0.0);
    for (int tenth = 1; tenth <= 4; ++tenth) {
      float along = height * 0.1 * (float)tenth;
      sides = max(sides, float2(
          highlight_inked(highlight_fetch(canvas_point - float2(along, 0.0)), page, toward, own),
          highlight_inked(highlight_fetch(canvas_point + float2(along, 0.0)), page, toward, own)));
    }
    enclosed = sides.x * sides.y;
    if (enclosed <= 0.0) return 0.0;
  }
  float3 fill_colour = margin.x >= margin.y ? above : below;
  float alike = 0.0;
  for (int ring = 1; ring <= 2; ++ring)
    for (int turn = 0; turn < 16; ++turn) {
      float angle = (float)turn * (3.14159265 / 8.0);
      float2 probe = canvas_point + float2(cos(angle), sin(angle)) * height * 0.25 * (float)ring;
      alike += all(abs(highlight_fetch(probe) - fill_colour) < 0.12) ? 1.0 : 0.0;
    }
  return reaches * enclosed * smoothstep(0.3, 0.42, alike / 32.0);
}

/// `base` under a felt marker in `colour`. The twin of the Metal
/// `highlight_tint`, which says how.
float3 highlight_tint(float3 base, float3 colour) {
  float dark = 1.0 - saturate((dot(base, float3(0.2126, 0.7152, 0.0722)) - 0.2) / 0.4);
  float3 laid = base * lerp(float3(1.0, 1.0, 1.0), colour, 0.7);
  float3 lifted = base + colour * (1.0 - base) * 0.25;
  return lerp(laid, lifted, dark);
}

/// `base` recoloured under a highlight in `colour`: the page becomes the
/// highlight's colour and its ink a colour that reads on it, keeping its hue;
/// where `fills`, a fill the line between `rows` is drawn on is tinted
/// instead. The twin of `highlight_recolour` in
/// `gpu_compositor_macos_shader_source_annotation_highlight_ink.h`, which says
/// why the ink is measured and laid in linear light, against each glyph's own
/// strongest pixel.
float3 highlight_recolour(float3 base, float3 colour, float2 tone, float2 canvas_point,
                          float2 rows, bool fills) {
  const float3 weights = float3(0.2126, 0.7152, 0.0722);
  float span = tone.y - tone.x;
  // No page was read under it - a photo, a gradient, a box laid by hand - so
  // there is nothing to map from, and the highlight tints what is there.
  if (abs(span) < 0.01) return highlight_tint(base, colour);
  if (abs(span) < 0.2) span = span < 0.0 ? -0.2 : 0.2;
  // The page, the ink the selection read and a quarter of the way between
  // them by eye, in linear light.
  float3 levels = highlight_light(float3(tone.x, tone.x + span, tone.x + span * 0.25));
  float page = levels.x;
  float toward = span > 0.0 ? 1.0 : -1.0;
  float faint = toward * (levels.z - page);
  float3 lit = highlight_light(base);
  float stands = toward * (dot(lit, weights) - page);
  // What counts as the glyph's full ink: its own strongest pixel, but never
  // less than the quarter mark. Without a picture, the ink the selection read.
  float peak = highlight_peak(canvas_point, rows.y - rows.x, page, toward);
  float full = peak < 0.0 ? toward * (levels.y - page) : max(peak, faint);
  // How much of the pixel the ink covers, in a straight line from the page to
  // the ink, so an edge keeps the share it has in the capture. The floor keeps
  // faint page texture off.
  float cover = saturate(stands / full);
  float share = saturate((cover - 0.04) / 0.96);
  // The ink taken back out of the page it was mixed with, the page read as a
  // grey of its brightness, so an edge is not inked in the page's colour.
  float3 text = highlight_encode((lit - page * (1.0 - cover)) / max(cover, 0.1));
  float text_luminance = dot(text, weights);
  float3 ink = dot(colour, weights) > 0.5
      ? text * min(1.0, 0.2 / max(text_luminance, 1e-3))
      : 1.0 - (1.0 - text) * min(1.0, 0.15 / max(1.0 - text_luminance, 1e-3));
  float3 recoloured = highlight_encode(lerp(highlight_light(colour), highlight_light(ink), share));
  if (!fills) return recoloured;
  float fill = highlight_fill(canvas_point, rows, page, toward, stands, faint);
  return lerp(recoloured, highlight_tint(base, colour), fill);
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

/// One highlight over `rgba`, recolouring `base`. The editor hands the pixel
/// as every mark but the highlights drew it, so a highlight recolours what
/// lies under it and overlapping highlights merge; the live overlay's `rgba`
/// is transparent, and `base` is the desktop captured under it.
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
  // The top and bottom of the line `canvas_point` lies on, or of the first
  // one near.
  float2 rows = float2(0.0, 0.0);
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
    if (rows.y <= rows.x || (canvas_point.y >= low.y && canvas_point.y <= high.y))
      rows = float2(low.y, high.y);
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
  // A box's bands are strokes a marker tall, not lines with fills drawn under
  // them, so it recolours every pixel.
  bool fills = (annotation.flags & annotation_highlight_laid_by_hand) == 0u;
  rgba.rgb = highlight_recolour(base.rgb, colour.rgb, tone, canvas_point, rows, fills) * alpha +
             rgba.rgb * (1.0 - alpha);
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
