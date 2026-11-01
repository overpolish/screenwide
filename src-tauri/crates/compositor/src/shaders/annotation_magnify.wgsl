// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The magnifier: one lens of clear glass enlarging what lies directly under
// it about its own middle, and the lens's shadow. `annotation_glass.wgsl`
// holds the glass. Every number it reads was prepared by
// `magnify::geometry::prepare_magnify`, which documents where the record
// keeps each part.
//
// What a lens shows comes from `annotation_lens_ground`, which each shader
// that draws annotations defines over its own picture: the preview's is the
// canvas as composed under every annotation - the background, the layers
// under this one, the camera behind the screen and the screen, cropped,
// inset, rounded and turned - and the live overlay's, which offers no
// magnifier, the desktop its highlights recolour. In the editor a lens shows
// every annotation below it as well, enlarged: `annotation_lens.wgsl`
// composes what it shows ahead of the canvas and hands it to
// `annotation_magnify_draw`.

const annotation_magnify_kind: u32 = 8u;
// The flag a lens that casts a shadow carries: `flags::SHADOW`.
const annotation_magnify_shadow_flag: u32 = 1u << 7u;
// The lens's shadow, by shares of its shorter half side: a soft one cast
// down by light from above, `drop` below it and `blur` its deviation, and a
// tight one hugging its edge, where it rests on the picture, so the glass
// sits on the picture rather than hovering over it.
const annotation_magnify_shadow_opacity: f32 = 0.16;
const annotation_magnify_shadow_blur: f32 = 0.1;
const annotation_magnify_shadow_drop: f32 = 0.05;
const annotation_magnify_shadow_contact: f32 = 0.35;

// Where a picture's texels lie on the canvas: the placed image's rectangle,
// the texels the canvas shows of it, first and last inclusive, and its size.
// The highlights read the picture through it to find each glyph's ink.
struct AnnotationMagnifyPlacement {
  image: vec4<f32>,
  texels: vec4<f32>,
  size: vec2<f32>,
}

// A rounded box's signed distance, by its centre and half size.
fn annotation_magnify_box(probe: vec2<f32>, centre: vec2<f32>, half_size: vec2<f32>,
                          radius: f32) -> f32 {
  let rounding = min(radius, min(half_size.x, half_size.y));
  let q = abs(probe - centre) - (half_size - rounding);
  return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - rounding;
}

// How far a lens's shadow reaches past its edge, and its deviation and
// drop, for a lens `half_size` each side of its centre.
fn annotation_magnify_shadow_spread(half_size: vec2<f32>) -> vec3<f32> {
  let size = min(half_size.x, half_size.y);
  let sigma = clamp(size * annotation_magnify_shadow_blur, 2.0, 110.0);
  let drop = size * annotation_magnify_shadow_drop;
  return vec3<f32>(sigma * 4.0 + drop, sigma, drop);
}

// How much of the picture's light the shadow of a lens takes at `probe`.
fn annotation_magnify_shadow(probe: vec2<f32>, centre: vec2<f32>, half_size: vec2<f32>,
                             rounding: f32) -> f32 {
  let spread = annotation_magnify_shadow_spread(half_size);
  let sigma = spread.y;
  let fall = max(annotation_magnify_box(probe - vec2<f32>(0.0, spread.z), centre, half_size,
                                        rounding), 0.0);
  let edge = max(annotation_magnify_box(probe, centre, half_size, rounding), 0.0);
  let contact = sigma * annotation_magnify_shadow_contact;
  return annotation_magnify_shadow_opacity *
      (0.65 * exp(-0.5 * fall * fall / (sigma * sigma)) +
       0.35 * exp(-0.5 * edge * edge / (contact * contact)));
}

// The glass of a prepared lens at `probe`, where `feather` is half a drawn
// pixel.
fn annotation_magnify_glass(shape: PreviewGeometry, probe: vec2<f32>, feather: f32)
    -> AnnotationGlass {
  return annotation_glass_at(probe, vec2<f32>(shape.ax, shape.ay), vec2<f32>(shape.bx, shape.by),
                             shape.rounding, saturate(shape.low), feather * 2.0);
}

// How far past its edge a lens can draw: its outer hairline and, where it
// casts one, its shadow.
fn annotation_magnify_spread(half_size: vec2<f32>, flags: u32, feather: f32) -> f32 {
  let shadowed = (flags & annotation_magnify_shadow_flag) != 0u;
  return feather * 2.0 +
      annotation_glass_hairline_width(min(half_size.x, half_size.y), feather * 2.0) +
      select(0.0, annotation_magnify_shadow_spread(half_size).x, shadowed);
}

// Draws one lens, as `shape` places it and in `color`, over `rgba`: its
// shadow, its hover halo round its edge, and the lens, `content` seen through
// `glass`. `content` is what the lens shows at `probe`, premultiplied: what
// lies under it at the matching point nearer its middle, enlarged, as the
// caller composed it where the glass looks. It is solid throughout: a lens
// settling onto the picture at no zoom shows exactly what it covers, so it
// lifts off the picture rather than fading in over it. The shadow and the
// glass come with the lens's presence; the halo with its colour, which
// carries the presence already.
fn annotation_magnify_draw(rgba_in: vec4<f32>, shape: PreviewGeometry, color: vec4<f32>,
                           flags: u32, probe: vec2<f32>, feather: f32, halo: f32,
                           content: vec4<f32>, glass: AnnotationGlass) -> vec4<f32> {
  let centre = vec2<f32>(shape.ax, shape.ay);
  let half_size = vec2<f32>(shape.bx, shape.by);
  if (any(half_size <= vec2<f32>(0.0)) ||
      any(abs(probe - centre) > half_size + annotation_magnify_spread(half_size, flags, feather) +
          halo)) {
    return rgba_in;
  }
  var rgba = rgba_in;
  let presence = saturate(shape.low);
  let size = min(half_size.x, half_size.y);
  let lens = annotation_magnify_box(probe, centre, half_size, shape.rounding);
  let inside = annotation_edge(lens, feather);
  if ((flags & annotation_magnify_shadow_flag) != 0u && inside < 1.0) {
    let shadow = presence * annotation_magnify_shadow(probe, centre, half_size, shape.rounding);
    rgba = vec4<f32>(rgba.rgb * (1.0 - shadow * (1.0 - inside)), rgba.a);
  }
  if (halo > 0.0) {
    let band = (1.0 - annotation_edge(lens, feather)) * annotation_edge(lens - halo, feather);
    rgba = annotation_over(rgba, color.rgb, band * color.a * annotation_hover_alpha);
  }
  rgba = annotation_glass_outline_over(rgba, lens, size, presence, feather * 2.0);
  if (inside > 0.0) {
    rgba = content * inside + rgba * (1.0 - inside * content.a);
    rgba = annotation_glass_light_over(rgba, glass, inside);
  }
  return rgba;
}

// How many canvas pixels under a lens one pixel of it shows, on each axis.
fn annotation_magnify_shrink(shape: PreviewGeometry) -> vec2<f32> {
  return vec2<f32>(1.0 / max(shape.high, 1e-3));
}

// The point under a lens it shows at `probe`.
fn annotation_magnify_seen_at(shape: PreviewGeometry, probe: vec2<f32>) -> vec2<f32> {
  let centre = vec2<f32>(shape.ax, shape.ay);
  return centre + (probe - centre) * annotation_magnify_shrink(shape);
}

// What a lens shows at `probe` of the picture alone, premultiplied, and of
// the cursor lying on it where `cursor` says the layer carries it; nothing
// outside the lens.
fn annotation_magnify_picture(shape: PreviewGeometry, probe: vec2<f32>, feather: f32,
                              cursor: bool) -> vec4<f32> {
  if (annotation_magnify_inside(shape, probe, feather) <= 0.0) {
    return vec4<f32>(0.0);
  }
  let shrink = annotation_magnify_shrink(shape);
  let seen_at = annotation_magnify_seen_at(shape, probe);
  var content = annotation_lens_ground(seen_at, shrink * feather * 2.0);
  if (cursor) {
    let pointer = annotation_cursor_seen(seen_at, feather * 2.0 * min(shrink.x, shrink.y));
    content = annotation_over(content, pointer.rgb, pointer.a);
  }
  return content;
}

// Draws one lens over `rgba`, showing the picture alone, looked at once
// where the glass keeps the colours together and once per colour where it
// parts them. One that changed while the shutter was open is drawn whole at
// every exposure sample, each at the opacity it had then, and the results
// averaged: what an open shutter records of a lens settling, picture and all.
fn annotation_magnify_layer(rgba: vec4<f32>, magnifier: PreviewArrow, probe: vec2<f32>,
                            feather: f32, halo: f32, cursor: bool) -> vec4<f32> {
  let color = annotation_color(magnifier);
  let taps = max(magnifier.sample_count, 1u);
  var total = vec4<f32>(0.0);
  for (var tap = 0u; tap < taps; tap++) {
    var shape = magnifier.geometry;
    var tint = color;
    if (magnifier.sample_count > 0u) {
      let sample = annotation_samples[magnifier.sample_first + tap];
      shape = sample.geometry;
      tint = vec4<f32>(color.rgb, color.a * sample.opacity);
    }
    let glass = annotation_magnify_glass(shape, probe, feather);
    let looks = annotation_glass_looks(glass);
    var content = vec4<f32>(0.0);
    for (var look = 0u; look < looks; look++) {
      let seen = annotation_magnify_picture(shape, annotation_glass_look(glass, look, looks),
                                            feather, cursor);
      content = annotation_glass_take(content, seen, look, looks);
    }
    total += annotation_magnify_draw(rgba, shape, tint, magnifier.flags, probe, feather, halo,
                                     content, glass);
  }
  return total / f32(taps);
}

// How much of `probe` one prepared lens covers.
fn annotation_magnify_inside(shape: PreviewGeometry, probe: vec2<f32>, feather: f32) -> f32 {
  let centre = vec2<f32>(shape.ax, shape.ay);
  let half_size = vec2<f32>(shape.bx, shape.by);
  if (any(half_size <= vec2<f32>(0.0)) || any(abs(probe - centre) > half_size + feather * 2.0)) {
    return 0.0;
  }
  return annotation_edge(annotation_magnify_box(probe, centre, half_size, shape.rounding),
                         feather);
}

// How much of `probe` a lens covers, over its exposure as
// `annotation_magnify_layer` draws it: what a cursor lying under the lens is
// hidden by.
fn annotation_magnify_cover(magnifier: PreviewArrow, probe: vec2<f32>, feather: f32) -> f32 {
  if (magnifier.sample_count == 0u) {
    return annotation_magnify_inside(magnifier.geometry, probe, feather);
  }
  var total = 0.0;
  for (var tap = 0u; tap < magnifier.sample_count; tap++) {
    total += annotation_magnify_inside(annotation_samples[magnifier.sample_first + tap].geometry,
                                       probe, feather);
  }
  return total / f32(magnifier.sample_count);
}

/// Where one prepared lens can reach: itself, its outer hairline, its shadow
/// and its halo.
fn annotation_magnify_reach(shape: PreviewGeometry, flags: u32, feather: f32,
                            halo: f32) -> vec4<f32> {
  let centre = vec2<f32>(shape.ax, shape.ay);
  let half_size = vec2<f32>(shape.bx, shape.by);
  if (any(half_size <= vec2<f32>(0.0))) {
    return annotation_no_bounds;
  }
  let reach = half_size + annotation_magnify_spread(half_size, flags, feather) + halo;
  return vec4<f32>(centre - reach, centre + reach);
}

/// Where a lens can reach, over its exposure as it is drawn.
fn annotation_magnify_bounds(magnifier: PreviewArrow, feather: f32) -> vec4<f32> {
  let halo = max(magnifier.hover, 0.0);
  var bounds = annotation_magnify_reach(magnifier.geometry, magnifier.flags, feather, halo);
  for (var tap = 0u; tap < magnifier.sample_count; tap++) {
    bounds = annotation_union(bounds, annotation_magnify_reach(
        annotation_samples[magnifier.sample_first + tap].geometry, magnifier.flags, feather, halo));
  }
  return bounds;
}
