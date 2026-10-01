// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

/// What a highlight does to one pixel under it: recolour it from the page the
/// selection read, or tint it where no page was read. Goes after
/// `..._annotation_magnify.h`, whose taps it reads the picture through, and
/// before `..._annotation_highlight.h`, which lays the bands. The twin of
/// `highlight_recolour` in `annotation_highlight.hlsl`.
///
/// The compositor's textures hold encoded values, but a glyph's anti-aliased
/// edge is a mix of page and ink in light. Recolouring measures how much of
/// each pixel the ink covers in linear light and lays the new ink over the
/// highlight's colour by the same share, the way type is drawn. Measured and
/// laid in encoded values instead, light text on a dark page comes out bolder
/// once it is dark text on a light highlight.
///
/// A pixel's brightness alone cannot tell the faint edge of bright text from
/// the solid middle of dim text, so each pixel is measured against the
/// strongest pixel of the glyph around it, read from the picture: dim and
/// coloured text is inked as fully as the brightest on its line.
#define GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_HIGHLIGHT_INK @R"METAL(
/// Encoded channels in linear light.
static float3 highlight_light(float3 encoded) {
  float3 channel = saturate(encoded);
  return select(pow((channel + 0.055) / 1.055, 2.4), channel / 12.92, channel <= 0.04045);
}

/// Linear light as encoded channels.
static float3 highlight_encode(float3 light) {
  float3 channel = saturate(light);
  return select(1.055 * pow(channel, 1.0 / 2.4) - 0.055, channel * 12.92, channel <= 0.0031308);
}

/// How far the strongest pixel around `point` stands out from `page`, in
/// linear light and in the direction ink lies, `toward`: the picture `tap`
/// reads, five by five texels spread over a reach that grows with the
/// highlighted line's height, `line_height` canvas pixels. That reaches the
/// core of a glyph from anywhere on its edge, however soft a zoomed or blurred
/// capture left it, with taps spaced closer than type of that size is thick.
/// Negative where the tap has no picture.
template <typename Tap>
static float highlight_peak(Tap tap, float2 point, float line_height, float page,
                            float toward) {
  AnnotationMagnifyPlacement at = tap.at;
  if (any(at.image.zw <= 0.0) || any(at.size < 1.0)) return -1.0;
  const float3 weights = float3(0.2126, 0.7152, 0.0722);
  float2 per = at.size / at.image.zw;
  float2 centre = (point - at.image.xy) * per;
  float spread = clamp(line_height * per.y * 0.12, 2.0, 8.0) * 0.5;
  float peak = 0.0;
  for (int y = -2; y <= 2; ++y)
    for (int x = -2; x <= 2; ++x) {
      float2 probe = floor(centre + float2(x, y) * spread);
      int2 texel = int2(clamp(probe, at.texels.xy, at.texels.zw));
      float3 lit = highlight_light(annotation_magnify_fetch(tap, texel).rgb);
      peak = max(peak, toward * (dot(lit, weights) - page));
    }
  return peak;
}

/// The picture `tap` reads at canvas point `q`, encoded; past what the canvas
/// shows, the edge the crop left carries on.
template <typename Tap>
static float3 highlight_fetch(Tap tap, float2 q) {
  AnnotationMagnifyPlacement at = tap.at;
  float2 per = at.size / at.image.zw;
  int2 texel = int2(clamp(floor((q - at.image.xy) * per), at.texels.xy, at.texels.zw));
  return annotation_magnify_fetch(tap, texel).rgb;
}

/// Whether `rgb` stands out from `page` towards the ink by at least half of
/// `own`, eased in from three tenths of it.
static float highlight_inked(float3 rgb, float page, float toward, float own) {
  float stands = toward * (dot(highlight_light(rgb), float3(0.2126, 0.7152, 0.0722)) - page);
  return smoothstep(0.3, 0.6, stands / own);
}

/// How surely `point` lies on a fill rather than on type: a cell's colour, a
/// label or a panel the line is drawn on, or the type printed on one.
///
/// The band, `rows`, reaches past its line's ink by an eighth of its height
/// either side, which is page wherever the line is printed on the page. Where
/// the margin above and the margin below are both one fill, at `point`'s
/// column and a stroke's width beside it, the line is printed on that fill
/// here: everything between the two is the fill or its type and is tinted
/// whole, and past them only what is the fill's own colour. Decided for the
/// column rather than the pixel, nothing on the fill can fall back to being
/// recoloured one pixel at a time - a letter's dark edge turning the page's
/// colour, a speck of the fill turning ink - however the picture is scaled.
///
/// Where only one margin reaches a fill, the line is not centred on it, and a
/// pixel's neighbourhood cannot tell a label from a big glyph. So three things
/// are asked of the picture `tap` reads, each of which type fails:
///
/// - A fill under the line reaches into the margin, above or below, at
///   `point`'s column and a stroke's width either side; the width keeps a
///   neighbouring line's tall glyph from passing for one.
/// - `point` stands out from `page` itself, or has ink to its left and to its
///   right along its row, as type printed on the fill does and the page the
///   band reaches over beside a fill does not.
/// - The fill's colour, the margin's, is most of what lies around `point` a
///   quarter and a half of the band away. Around even the heaviest type, its
///   colour is the lesser part.
///
/// `stands` is how far `point` stands out from the page, and `faint` the
/// least that counts as ink.
template <typename Tap>
static float highlight_fill(Tap tap, float2 point, float2 rows, float page, float toward,
                            float stands, float faint) {
  AnnotationMagnifyPlacement at = tap.at;
  float height = rows.y - rows.x;
  if (any(at.image.zw <= 0.0) || any(at.size < 1.0) || height <= 0.0) return 0.0;
  float own = max(stands, faint);
  float2 span = rows.x + height * float2(0.06, 0.94);
  float2 margin = float2(1.0);
  float2 centre = float2(0.0), beside = float2(0.0);
  float3 above = float3(0.0), below = float3(0.0);
  for (int side = -1; side <= 1; ++side) {
    float x = point.x + float(side) * height * 0.1;
    float3 top = highlight_fetch(tap, float2(x, span.x));
    float3 bottom = highlight_fetch(tap, float2(x, span.y));
    margin *= float2(highlight_inked(top, page, toward, own),
                     highlight_inked(bottom, page, toward, own));
    // Measured against the least ink rather than the pixel's own, so every
    // pixel in the column reads its margins alike.
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
    if (point.y >= span.x && point.y <= span.y) return both;
    return all(abs(highlight_fetch(tap, point) - above) < 0.12) ? both : 0.0;
  }
  float reaches = max(margin.x, margin.y);
  if (reaches <= 0.0) return 0.0;
  float enclosed = 1.0;
  if (stands <= faint) {
    float2 sides = float2(0.0);
    for (int tenth = 1; tenth <= 4; ++tenth) {
      float along = height * 0.1 * float(tenth);
      sides = max(sides, float2(
          highlight_inked(highlight_fetch(tap, point - float2(along, 0.0)), page, toward, own),
          highlight_inked(highlight_fetch(tap, point + float2(along, 0.0)), page, toward, own)));
    }
    enclosed = sides.x * sides.y;
    if (enclosed <= 0.0) return 0.0;
  }
  float3 fill_colour = margin.x >= margin.y ? above : below;
  float alike = 0.0;
  for (int ring = 1; ring <= 2; ++ring)
    for (int turn = 0; turn < 16; ++turn) {
      float angle = float(turn) * (M_PI_F / 8.0);
      float2 probe = point + float2(cos(angle), sin(angle)) * height * 0.25 * float(ring);
      alike += all(abs(highlight_fetch(tap, probe) - fill_colour) < 0.12) ? 1.0 : 0.0;
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
static float3 highlight_tint(float3 base, float3 colour) {
  float dark = 1.0 - saturate((dot(base, float3(0.2126, 0.7152, 0.0722)) - 0.2) / 0.4);
  float3 laid = base * mix(float3(1.0, 1.0, 1.0), colour, 0.7);
  float3 lifted = base + colour * (1.0 - base) * 0.25;
  return mix(laid, lifted, dark);
}

/// `base` recoloured under a highlight in `colour`: the page becomes the
/// highlight's colour and its ink - whatever stands out from the page - a
/// colour that reads on it. The ink keeps its hue, pushed dark on a light
/// highlight and light on a dark one. `tone` is the page's luminance and its
/// ink's, encoded; `tap` reads the picture around `point`, on a line whose
/// top and bottom are `rows`. Where `fills`, a fill the line is drawn on is
/// tinted instead, type and all, so it reads as the marker laid over it.
template <typename Tap>
static float3 highlight_recolour(float3 base, float3 colour, float2 tone, Tap tap,
                                 float2 point, float2 rows, bool fills) {
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
  // less than the quarter mark, so faint page texture is not taken for text.
  // Without a picture, the ink the selection read.
  float peak = highlight_peak(tap, point, rows.y - rows.x, page, toward);
  float full = peak < 0.0 ? toward * (levels.y - page) : max(peak, faint);
  // How much of the pixel the ink covers, in a straight line from the page to
  // the ink, so an edge keeps the share it has in the capture: any steeper
  // curve swells each glyph and traces the capture's pixel grid. The floor
  // keeps faint page texture off.
  float cover = saturate(stands / full);
  float share = saturate((cover - 0.04) / 0.96);
  // The ink itself, taken back out of the page it was mixed with, the page
  // being read as a grey of its brightness. An edge pixel is mostly page, and
  // inked in its own colour it would rim each glyph in the page's.
  float3 text = highlight_encode((lit - page * (1.0 - cover)) / max(cover, 0.1));
  float text_luminance = dot(text, weights);
  float3 ink = dot(colour, weights) > 0.5
      ? text * min(1.0, 0.2 / max(text_luminance, 1e-3))
      : 1.0 - (1.0 - text) * min(1.0, 0.15 / max(1.0 - text_luminance, 1e-3));
  float3 recoloured = highlight_encode(mix(highlight_light(colour), highlight_light(ink), share));
  if (!fills) return recoloured;
  float fill = highlight_fill(tap, point, rows, page, toward, stands, faint);
  return fill > 0.0 ? mix(recoloured, highlight_tint(base, colour), fill) : recoloured;
}
)METAL"
