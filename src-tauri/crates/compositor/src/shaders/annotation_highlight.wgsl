// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The highlight: every band of every highlight in a run, recoloured rather
// than painted over. It reads each glyph's full ink through the magnifier's
// picture functions.
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
//
// The compositor's textures hold encoded values, but a glyph's anti-aliased
// edge is a mix of page and ink in light. Recolouring measures how much of
// each pixel the ink covers in linear light and lays the new ink over the
// highlight's colour by the same share, the way type is drawn. Measured and
// laid in encoded values instead, light text on a dark page comes out bolder
// once it is dark text on a light highlight.
//
// A pixel's brightness alone cannot tell the faint edge of bright text from
// the solid middle of dim text, so each pixel is measured against the
// strongest pixel of the glyph around it, read from the picture: dim and
// coloured text is inked as fully as the brightest on its line.

const annotation_highlight_kind: u32 = 4u;
const annotation_highlight_hand_drawn: u32 = 1u << 6u;
const annotation_highlight_laid_by_hand: u32 = 1u << 8u;
const highlight_weights: vec3<f32> = vec3<f32>(0.2126, 0.7152, 0.0722);

fn highlight_hash(input: u32) -> u32 {
  var value = input;
  value ^= value >> 16u;
  value *= 0x7feb352du;
  value ^= value >> 15u;
  value *= 0x846ca68bu;
  value ^= value >> 16u;
  return value;
}

/// A number in [0, 1) that only `seed` and `salt` decide.
fn highlight_random(seed: u32, salt: u32) -> f32 {
  return f32(highlight_hash(seed ^ highlight_hash(salt)) & 0xffffffu) / 16777216.0;
}

/// Smooth value noise along one axis, in [0, 1].
fn highlight_noise(at: f32, seed: u32) -> f32 {
  let cell = floor(at);
  let along = at - cell;
  let index = bitcast<u32>(i32(cell));
  let low = highlight_random(seed, index);
  let high = highlight_random(seed, index + 1u);
  return mix(low, high, along * along * (3.0 - 2.0 * along));
}

/// The distance to a box from `low` to `high` whose left and right ends are
/// rounded by radii of their own.
fn highlight_box(probe: vec2<f32>, low: vec2<f32>, high: vec2<f32>, left_radius: f32,
                 right_radius: f32) -> f32 {
  let centre = (low + high) * 0.5;
  let half_size = max((high - low) * 0.5, vec2<f32>(0.0));
  let radius = min(select(right_radius, left_radius, probe.x < centre.x),
                   min(half_size.x, half_size.y));
  let q = abs(probe - centre) - (half_size - radius);
  return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - radius;
}

/// A marker stroke drawn by hand along the band from `low` to `high`, drawn in
/// from `start` to `end`, its ends settled as far as `settled` says and slanted
/// by `slant`. `joined` is whether its top and its bottom lie under a
/// neighbouring band: a shared edge runs straight and the stroke keeps level,
/// so no page shows between the strokes laid over a box.
fn highlight_stroke(probe: vec2<f32>, low: vec2<f32>, high: vec2<f32>, start: f32, end: f32,
                    settled: vec2<f32>, slant: f32, seed: u32, joined: vec2<f32>) -> f32 {
  let band = high.y - low.y;
  // Turned about the band's own centre, so a stroke drawing itself in does not
  // swing as it grows, and by no more than moves its ends a tenth of its
  // height, so a long line stays on the text it covers.
  let centre = (low + high) * 0.5;
  let free_hand = 1.0 - max(joined.x, joined.y);
  let most = min(0.026, band * 0.1 / max((high.x - low.x) * 0.5, 1.0));
  let tilt = (highlight_random(seed, 1u) * 2.0 - 1.0) * most * free_hand;
  let offset = probe - centre;
  var level = vec2<f32>(offset.x * cos(tilt) + offset.y * sin(tilt),
                        offset.y * cos(tilt) - offset.x * sin(tilt));
  level.y -= (highlight_random(seed, 2u) - 0.5) * 0.12 * band * free_hand;
  let along = level.x + level.y * slant + centre.x;
  // A settled end only ever runs long: one that fell short would leave the
  // edge of a word unmarked. It runs long as the stroke reaches it, so the
  // draw lands on its end rather than jumping out to it.
  let left = start - settled.x * (0.05 + highlight_random(seed, 3u) * 0.25) * band;
  let right = end + settled.y * (0.05 + highlight_random(seed, 4u) * 0.3) * band;
  let wave = max(band * 2.6, 1.0);
  let top = -band * 0.5 +
      band * 0.07 * (1.0 - joined.x) * (highlight_noise(along / wave, seed ^ 5u) * 2.0 - 1.0);
  let bottom = band * 0.5 +
      band * 0.07 * (1.0 - joined.y) * (highlight_noise(along / wave, seed ^ 6u) * 2.0 - 1.0);
  let radius = band * 0.16;
  let outside = vec2<f32>(max(left - along, along - right), max(top - level.y, level.y - bottom)) +
      radius;
  return length(max(outside, vec2<f32>(0.0))) + min(max(outside.x, outside.y), 0.0) - radius;
}

/// Encoded channels in linear light.
fn highlight_light(encoded: vec3<f32>) -> vec3<f32> {
  let channel = saturate(encoded);
  return mix(pow((channel + 0.055) / 1.055, vec3<f32>(2.4)), channel / 12.92,
             step(channel, vec3<f32>(0.04045)));
}

/// Linear light as encoded channels.
fn highlight_encode(light: vec3<f32>) -> vec3<f32> {
  let channel = saturate(light);
  return mix(1.055 * pow(channel, vec3<f32>(1.0 / 2.4)) - 0.055, channel * 12.92,
             step(channel, vec3<f32>(0.0031308)));
}

/// How far the strongest pixel around `canvas_point` stands out from `page`,
/// in linear light and in the direction ink lies, `toward`: the picture
/// `annotation_magnify_fetch` reads, five by five texels spread over a reach
/// that grows with the line's height, `line_height` canvas pixels. That
/// reaches the core of a glyph from anywhere on its edge, however soft a
/// zoomed or blurred capture left it, with taps spaced closer than type of
/// that size is thick. Negative where there is no picture.
fn highlight_peak(canvas_point: vec2<f32>, line_height: f32, page: f32, toward: f32) -> f32 {
  let at = annotation_magnify_placement();
  if (any(at.image.zw <= vec2<f32>(0.0)) || any(at.size < vec2<f32>(1.0))) {
    return -1.0;
  }
  let per = at.size / at.image.zw;
  let centre = (canvas_point - at.image.xy) * per;
  let spread = clamp(line_height * per.y * 0.12, 2.0, 8.0) * 0.5;
  var peak = 0.0;
  for (var y = -2; y <= 2; y++) {
    for (var x = -2; x <= 2; x++) {
      let probe = floor(centre + vec2<f32>(f32(x), f32(y)) * spread);
      let texel = vec2<i32>(clamp(probe, at.texels.xy, at.texels.zw));
      let lit = highlight_light(annotation_magnify_fetch(texel).rgb);
      peak = max(peak, toward * (dot(lit, highlight_weights) - page));
    }
  }
  return peak;
}

/// The picture at canvas point `q`, encoded; past what the canvas shows, the
/// edge the crop left carries on.
fn highlight_fetch(q: vec2<f32>) -> vec3<f32> {
  let at = annotation_magnify_placement();
  let per = at.size / at.image.zw;
  let texel = vec2<i32>(clamp(floor((q - at.image.xy) * per), at.texels.xy, at.texels.zw));
  return annotation_magnify_fetch(texel).rgb;
}

/// Whether `rgb` stands out from `page` towards the ink by at least half of
/// `own`, eased in from three tenths of it.
fn highlight_inked(rgb: vec3<f32>, page: f32, toward: f32, own: f32) -> f32 {
  let stands = toward * (dot(highlight_light(rgb), highlight_weights) - page);
  return smoothstep(0.3, 0.6, stands / own);
}

/// How surely `canvas_point` lies on a fill rather than on type: a cell's
/// colour, a label or a panel the line is drawn on, or the type printed on
/// one.
///
/// The band, `rows`, reaches past its line's ink by an eighth of its height
/// either side, which is page wherever the line is printed on the page. Where
/// the margin above and the margin below are both one fill, at the point's
/// column and a stroke's width beside it, the line is printed on that fill
/// here: everything between the two is the fill or its type and is tinted
/// whole, and past them only what is the fill's own colour. Decided for the
/// column rather than the pixel, nothing on the fill can fall back to being
/// recoloured one pixel at a time - a letter's dark edge turning the page's
/// colour, a speck of the fill turning ink - however the picture is scaled.
///
/// Where only one margin reaches a fill, the line is not centred on it, and a
/// pixel's neighbourhood cannot tell a label from a big glyph. So three
/// things are asked of the picture, each of which type fails:
///
/// - A fill under the line reaches into the margin, above or below, at the
///   point's column and a stroke's width either side; the width keeps a
///   neighbouring line's tall glyph from passing for one.
/// - The point stands out from `page` itself, or has ink to its left and to
///   its right along its row, as type printed on the fill does and the page
///   the band reaches over beside a fill does not.
/// - The fill's colour, the margin's, is most of what lies around the point a
///   quarter and a half of the band away. Around even the heaviest type, its
///   colour is the lesser part.
///
/// `stands` is how far the point stands out from the page, and `faint` the
/// least that counts as ink.
fn highlight_fill(canvas_point: vec2<f32>, rows: vec2<f32>, page: f32, toward: f32, stands: f32,
                  faint: f32) -> f32 {
  let at = annotation_magnify_placement();
  let height = rows.y - rows.x;
  if (any(at.image.zw <= vec2<f32>(0.0)) || any(at.size < vec2<f32>(1.0)) || height <= 0.0) {
    return 0.0;
  }
  let own = max(stands, faint);
  let span = rows.x + height * vec2<f32>(0.06, 0.94);
  var margin = vec2<f32>(1.0, 1.0);
  var centre = vec2<f32>(0.0, 0.0);
  var beside = vec2<f32>(0.0, 0.0);
  var above = vec3<f32>(0.0);
  var below = vec3<f32>(0.0);
  for (var side = -1; side <= 1; side++) {
    let x = canvas_point.x + f32(side) * height * 0.1;
    let top = highlight_fetch(vec2<f32>(x, span.x));
    let bottom = highlight_fetch(vec2<f32>(x, span.y));
    margin *= vec2<f32>(highlight_inked(top, page, toward, own),
                        highlight_inked(bottom, page, toward, own));
    // Measured against the least ink rather than the pixel's own, so every
    // pixel in the column reads its margins alike.
    let inked = vec2<f32>(highlight_inked(top, page, toward, faint),
                          highlight_inked(bottom, page, toward, faint));
    if (side == 0) {
      centre = inked;
      above = top;
      below = bottom;
    } else {
      beside = max(beside, inked);
    }
  }
  let both = min(centre.x * beside.x, centre.y * beside.y);
  if (both > 0.0 && all(abs(above - below) < vec3<f32>(0.12))) {
    if (canvas_point.y >= span.x && canvas_point.y <= span.y) {
      return both;
    }
    return select(0.0, both, all(abs(highlight_fetch(canvas_point) - above) < vec3<f32>(0.12)));
  }
  let reaches = max(margin.x, margin.y);
  if (reaches <= 0.0) {
    return 0.0;
  }
  var enclosed = 1.0;
  if (stands <= faint) {
    var sides = vec2<f32>(0.0, 0.0);
    for (var tenth = 1; tenth <= 4; tenth++) {
      let along = height * 0.1 * f32(tenth);
      sides = max(sides, vec2<f32>(
          highlight_inked(highlight_fetch(canvas_point - vec2<f32>(along, 0.0)), page, toward, own),
          highlight_inked(highlight_fetch(canvas_point + vec2<f32>(along, 0.0)), page, toward,
                          own)));
    }
    enclosed = sides.x * sides.y;
    if (enclosed <= 0.0) {
      return 0.0;
    }
  }
  let fill_colour = select(below, above, margin.x >= margin.y);
  var alike = 0.0;
  for (var ring = 1; ring <= 2; ring++) {
    for (var turn = 0; turn < 16; turn++) {
      let angle = f32(turn) * (3.14159265 / 8.0);
      let probe = canvas_point + vec2<f32>(cos(angle), sin(angle)) * height * 0.25 * f32(ring);
      alike += select(0.0, 1.0, all(abs(highlight_fetch(probe) - fill_colour) < vec3<f32>(0.12)));
    }
  }
  return reaches * enclosed * smoothstep(0.3, 0.42, alike / 32.0);
}

/// `base` under a felt marker in `colour`: over light it multiplies, well
/// short of fully, as the marker's ink does on paper. Multiplied, dark would
/// stay dark and the highlight would vanish, so dark is lifted a quarter of
/// the way to the colour instead. Each pixel is weighed by its own
/// brightness, so mixed content needs no reading of it; dark text on a light
/// page is lifted with the rest of the dark, to a deep shade of the colour
/// that still reads against it.
fn highlight_tint(base: vec3<f32>, colour: vec3<f32>) -> vec3<f32> {
  let dark = 1.0 - saturate((dot(base, highlight_weights) - 0.2) / 0.4);
  let laid = base * mix(vec3<f32>(1.0), colour, 0.7);
  let lifted = base + colour * (1.0 - base) * 0.25;
  return mix(laid, lifted, dark);
}

/// `base` recoloured under a highlight in `colour`: the page becomes the
/// highlight's colour and its ink - whatever stands out from the page - a
/// colour that reads on it. The ink keeps its hue, pushed dark on a light
/// highlight and light on a dark one. `tone` is the page's luminance and its
/// ink's, encoded, read around `canvas_point` on a line whose top and bottom
/// are `rows`. Where `fills`, a fill the line is drawn on is tinted instead,
/// type and all, so it reads as the marker laid over it.
fn highlight_recolour(base: vec3<f32>, colour: vec3<f32>, tone: vec2<f32>,
                      canvas_point: vec2<f32>, rows: vec2<f32>, fills: bool) -> vec3<f32> {
  var span = tone.y - tone.x;
  // No page was read under it - a photo, a gradient, a box laid by hand - so
  // there is nothing to map from, and the highlight tints what is there.
  if (abs(span) < 0.01) {
    return highlight_tint(base, colour);
  }
  if (abs(span) < 0.2) {
    span = select(0.2, -0.2, span < 0.0);
  }
  // The page, the ink the selection read and a quarter of the way between
  // them by eye, in linear light.
  let levels = highlight_light(vec3<f32>(tone.x, tone.x + span, tone.x + span * 0.25));
  let page = levels.x;
  let toward = select(-1.0, 1.0, span > 0.0);
  let faint = toward * (levels.z - page);
  let lit = highlight_light(base);
  let stands = toward * (dot(lit, highlight_weights) - page);
  // What counts as the glyph's full ink: its own strongest pixel, but never
  // less than the quarter mark. Without a picture, the ink the selection read.
  let peak = highlight_peak(canvas_point, rows.y - rows.x, page, toward);
  let full = select(max(peak, faint), toward * (levels.y - page), peak < 0.0);
  // How much of the pixel the ink covers, in a straight line from the page to
  // the ink, so an edge keeps the share it has in the capture. The floor keeps
  // faint page texture off.
  let cover = saturate(stands / full);
  let share = saturate((cover - 0.04) / 0.96);
  // The ink taken back out of the page it was mixed with, the page read as a
  // grey of its brightness, so an edge is not inked in the page's colour.
  let text = highlight_encode((lit - page * (1.0 - cover)) / max(cover, 0.1));
  let text_luminance = dot(text, highlight_weights);
  var ink: vec3<f32>;
  if (dot(colour, highlight_weights) > 0.5) {
    ink = text * min(1.0, 0.2 / max(text_luminance, 1e-3));
  } else {
    ink = 1.0 - (1.0 - text) * min(1.0, 0.15 / max(1.0 - text_luminance, 1e-3));
  }
  let recoloured = highlight_encode(mix(highlight_light(colour), highlight_light(ink), share));
  if (!fills) {
    return recoloured;
  }
  let fill = highlight_fill(canvas_point, rows, page, toward, stands, faint);
  return mix(recoloured, highlight_tint(base, colour), fill);
}

/// Where one line `span` long is drawn from and to at the reveal `low` to
/// `high`: it draws itself in over `share` of the reveal, from `begin`.
fn highlight_window(low: f32, high: f32, begin: f32, share: f32, span: f32) -> vec2<f32> {
  return span * saturate((vec2<f32>(low, high) - begin) / share);
}

/// How far `probe` falls outside the line from `low` to `high` drawn over
/// `window` along it.
fn highlight_line(probe: vec2<f32>, low: vec2<f32>, high: vec2<f32>, window: vec2<f32>,
                  joined: vec2<f32>, hand: bool, stroke: u32, slant: f32) -> f32 {
  let span = max(high.x - low.x, 0.0);
  let height = high.y - low.y;
  // How far each end has settled: a reveal's moving tip is round, and it
  // eases into the end's own shape over the last line height it travels.
  let settled = saturate(1.0 - vec2<f32>(window.x, span - window.y) / max(height, 1.0));
  if (hand) {
    return highlight_stroke(probe, low, high, low.x + window.x, low.x + window.y, settled, slant,
                            stroke, joined);
  }
  // The drawn end of a band still being drawn is the marker's round tip.
  // Settled, a corner on an edge shared with a neighbour is square, so a box
  // laid in strokes has no notch where two of them meet.
  // A settled corner is rounded as a one-line text box's is, as a share of
  // its height: `CORNER` over `LINE_HEIGHT` and twice `PAD_Y`, in
  // `editor/annotations/text/geometry.rs` and `text/metrics.rs`.
  let tip = height * 0.5;
  let square = height * 0.21 *
      (1.0 - select(joined.y, joined.x, probe.y < (low.y + high.y) * 0.5));
  return highlight_box(probe, vec2<f32>(low.x + window.x, low.y),
                       vec2<f32>(low.x + window.y, high.y), mix(tip, square, settled.x),
                       mix(tip, square, settled.y));
}

/// One highlight over `rgba`, recolouring `base`. The editor hands the pixel
/// as every mark but the highlights drew it, so a highlight recolours what
/// lies under it and overlapping highlights merge; the live overlay's `rgba`
/// is transparent, and `base` is the desktop captured under it.
fn annotation_highlight_layer(rgba_in: vec4<f32>, base: vec4<f32>, annotation: PreviewArrow,
                              canvas_point: vec2<f32>, feather: f32) -> vec4<f32> {
  let colour = annotation_color(annotation);
  let pairs = annotation.data_count / 2u;
  if (colour.a <= 0.0 || pairs == 0u) {
    return rgba_in;
  }
  var rgba = rgba_in;
  let record = annotation.geometry;
  let origin = vec2<f32>(record.ax, record.ay);
  let unit = vec2<f32>(record.bx, record.by);
  let tone = vec2<f32>(record.cx, record.cy);
  let hand = (annotation.flags & annotation_highlight_hand_drawn) != 0u;
  let seed = u32(annotation.params[0]) | (u32(annotation.params[1]) << 16u);
  // One hand holds the marker for the whole highlight, so its chisel leans
  // the same way on every line, which way being the seed's. How far is each
  // line's own: the hand is lifted and put down again between lines.
  let lean = select(1.0, -1.0, highlight_random(seed, 8u) < 0.5);
  // Each line draws itself in over its own share of the reveal, starting a
  // little after the line before it.
  let lines = f32(pairs);
  let share = min(1.0, max(0.35, 2.0 / (lines + 1.0)));
  var spacing = 0.0;
  if (pairs > 1u) {
    spacing = (1.0 - share) / (lines - 1.0);
  }
  let halo = max(annotation.hover, 0.0);
  // The exposure's samples run steadily from the shutter opening to now, so
  // the first and the last hold every sample's ends between them, and their
  // opacities' mean is the mean of all of them.
  let taps = annotation.sample_count;
  var opened = PreviewSample(record, 1.0);
  var closed = opened;
  if (taps > 0u) {
    opened = annotation_samples[annotation.sample_first];
    closed = annotation_samples[annotation.sample_first + taps - 1u];
  }
  let opacity = (opened.opacity + closed.opacity) * 0.5;
  // The top and bottom of the line `canvas_point` lies on, or of the first
  // one near.
  var rows = vec2<f32>(0.0, 0.0);
  var coverage = 0.0;
  var nearest = 1e20;
  for (var row = 0u; row < pairs; row++) {
    let at = annotation.data_offset + row * 2u;
    let low = origin + annotation_points[at] * unit;
    let high = origin + annotation_points[at + 1u] * unit;
    let span = max(high.x - low.x, 0.0);
    let height = high.y - low.y;
    let begin = f32(row) * spacing;
    let now = highlight_window(record.low, record.high, begin, share, span);
    let start = highlight_window(opened.geometry.low, opened.geometry.high, begin, share, span);
    let end = highlight_window(closed.geometry.low, closed.geometry.high, begin, share, span);
    let drawn_from = min(now.x, min(start.x, end.x));
    let drawn_to = max(now.y, max(start.y, end.y));
    if (drawn_to <= drawn_from || height <= 0.0) {
      continue;
    }
    let reach = height + halo + feather + 1.0;
    if (canvas_point.y < low.y - reach || canvas_point.y > high.y + reach ||
        canvas_point.x < low.x + drawn_from - reach || canvas_point.x > low.x + drawn_to + reach) {
      continue;
    }
    if (rows.y <= rows.x || (canvas_point.y >= low.y && canvas_point.y <= high.y)) {
      rows = vec2<f32>(low.y, high.y);
    }
    // Whether its top and its bottom lie under a neighbouring band, as the
    // strokes laid over a box do.
    var joined = vec2<f32>(0.0, 0.0);
    if (row > 0u && origin.y + annotation_points[at - 1u].y * unit.y > low.y) {
      joined.x = 1.0;
    }
    if (row + 1u < pairs && origin.y + annotation_points[at + 2u].y * unit.y < high.y) {
      joined.y = 1.0;
    }
    let stroke = seed ^ (row * 0x9e3779b9u);
    let slant = lean * (0.08 + highlight_random(stroke, 10u) * 0.32);
    if ((taps == 0u || halo > 0.0) && now.y > now.x) {
      let edge = highlight_line(canvas_point, low, high, now, joined, hand, stroke, slant);
      nearest = min(nearest, edge);
      if (taps == 0u) {
        coverage = max(coverage, annotation_edge(edge, feather));
        continue;
      }
    }
    if (taps == 0u) {
      continue;
    }
    // More than two line heights inside every sample's ends, every sample
    // draws this pixel alike, so the last stands for them all.
    let along = canvas_point.x - low.x;
    let margin = height * 2.0 + feather + 1.0;
    var line_coverage = 0.0;
    if (along > max(start.x, end.x) + margin && along < min(start.y, end.y) - margin) {
      let edge = highlight_line(canvas_point, low, high, end, joined, hand, stroke, slant);
      line_coverage = annotation_edge(edge, feather) * opacity;
    } else {
      for (var tap = 0u; tap < taps; tap++) {
        let sample = annotation_samples[annotation.sample_first + tap];
        let window = highlight_window(sample.geometry.low, sample.geometry.high, begin, share,
                                      span);
        if (window.y <= window.x) {
          continue;
        }
        let edge = highlight_line(canvas_point, low, high, window, joined, hand, stroke, slant);
        line_coverage += annotation_edge(edge, feather) * sample.opacity;
      }
      line_coverage /= f32(taps);
    }
    coverage = max(coverage, line_coverage);
  }
  if (halo > 0.0 && nearest < 1e19) {
    let ring = (1.0 - annotation_edge(nearest, feather)) * annotation_edge(nearest - halo, feather);
    rgba = annotation_over(rgba, colour.rgb, ring * colour.a * annotation_hover_alpha);
  }
  if (coverage <= 0.0) {
    return rgba;
  }
  let alpha = coverage * colour.a;
  // A box's bands are strokes a marker tall, not lines with fills drawn under
  // them, so it recolours every pixel.
  let fills = (annotation.flags & annotation_highlight_laid_by_hand) == 0u;
  let recoloured = highlight_recolour(base.rgb, colour.rgb, tone, canvas_point, rows, fills);
  return vec4<f32>(recoloured * alpha + rgba.rgb * (1.0 - alpha), alpha + rgba.a * (1.0 - alpha));
}

/// Every highlight in `[first, last)` over `rgba`, each recolouring `base`:
/// the live overlay's pass, which puts its highlights under everything else it
/// draws.
fn composite_highlights(rgba_in: vec4<f32>, base: vec4<f32>, canvas_point: vec2<f32>,
                        first: u32, last: u32, feather: f32) -> vec4<f32> {
  var rgba = rgba_in;
  for (var index = annotation_next(canvas_point, first, last); index < last;
       index = annotation_next(canvas_point, index + 1u, last)) {
    let annotation = annotation_arrows[index];
    if (annotation.kind != annotation_highlight_kind) {
      continue;
    }
    rgba = annotation_highlight_layer(rgba, base, annotation, canvas_point, feather);
  }
  return rgba;
}

/// Where a highlight can reach: every band, each a line height and the halo
/// beyond its box, as far as `annotation_highlight_layer` looks for a line.
fn annotation_highlight_bounds(annotation: PreviewArrow, feather: f32) -> vec4<f32> {
  let pairs = annotation.data_count / 2u;
  if (annotation.alpha <= 0.0 || pairs == 0u) {
    return annotation_no_bounds;
  }
  let record = annotation.geometry;
  let origin = vec2<f32>(record.ax, record.ay);
  let unit = vec2<f32>(record.bx, record.by);
  let halo = max(annotation.hover, 0.0);
  var bounds = annotation_no_bounds;
  for (var row = 0u; row < pairs; row++) {
    let at = annotation.data_offset + row * 2u;
    let low = origin + annotation_points[at] * unit;
    let high = origin + annotation_points[at + 1u] * unit;
    let reach = abs(high.y - low.y) + halo + feather + 1.0;
    bounds = annotation_union(bounds, vec4<f32>(min(low, high) - reach, max(low, high) + reach));
  }
  return bounds;
}
