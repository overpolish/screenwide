// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-FileCopyrightText: 2026 jeantimex
// SPDX-License-Identifier: GPL-3.0-or-later AND MIT

// The magnifier's lens as clear glass: a flat middle that shows what lies
// under it enlarged and undistorted, and a convex bezel round it that bends
// what it shows, parts the colours a little where it bends most, and catches
// a thin highlight along its rim. `annotation_magnify.wgsl` draws the lens
// through it and lays the lens's shadow under it.
//
// The bezel's height profile and the refraction through it (Snell's law down
// through the glass to the picture) are ported from glass-effect-webgpu by
// jeantimex, whose notice follows. Its other profiles, presets, blur pyramid
// and liquid animation are left out.
//
// Copyright (c) 2026 jeantimex
//
// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

// Every length is a share of the lens's shorter half side, so the glass
// looks the same at every size, in the preview and in an export alike. The
// numbers were picked from rendered sheets of candidates over text, colour,
// light and dark pictures. A squircle profile keeps the glass flat until
// close to its rim and rolls off there, so the bend gathers in a narrow band
// at the edge, as Apple's glass does, where a quarter circle or a straight
// ramp bends text well into the lens. A bezel of 0.3 is wide enough for
// that band to read as a lens at a small size, while the middle, where the
// squircle is flat, shows what lies under it as it is. Glass of index 1.5 and a
// fifth of the bezel thick, with the bend capped at 0.8 of the bezel, bends
// clearly without folding the picture over at the edge.
const annotation_glass_bezel: f32 = 0.3;
const annotation_glass_thickness: f32 = 0.2;
const annotation_glass_index: f32 = 1.5;
const annotation_glass_most_bend: f32 = 0.8;
// How far the colours part, as a share of the bend: red bends a little less
// than green and blue a little more, a fringe only where the bend is strong.
const annotation_glass_fringe: f32 = 0.08;
// How much the glass lifts towards white at its edge, as glass seen edge-on
// reflects more: only in the bezel, strongest under the upper rim, where the
// light falls, and gone at the sides and below, so the middle keeps the
// picture's own contrast.
const annotation_glass_sheen: f32 = 0.03;
// The highlight along the rim: lit from the upper left, so strongest at the
// upper left and lower right, where the rim faces the light, and faint
// along the rest. Its width is a share of the shorter half side, and never
// under one drawn pixel.
const annotation_glass_light: vec2<f32> = vec2<f32>(-0.5, -0.8660254);
const annotation_glass_rim_width: f32 = 0.01;
const annotation_glass_rim_opacity: f32 = 0.85;
const annotation_glass_rim_floor: f32 = 0.1;
// What the highlight saturates the picture it lies over by, as light through
// the rim does, before lifting it towards white.
const annotation_glass_rim_saturation: f32 = 2.5;
// The edge the glass keeps on any picture: a light hairline just inside it,
// clear on dark content and fading on light, where it would only be white on
// white, and a faint dark hairline just outside it, which reads on light
// content. Its width is a share of the shorter half side, about a point at
// a lens's usual size, and never under one drawn pixel.
const annotation_glass_hairline: f32 = 0.006;
const annotation_glass_hairline_on_dark: f32 = 0.55;
const annotation_glass_hairline_on_light: f32 = 0.1;
const annotation_glass_outline_on_dark: f32 = 0.05;
const annotation_glass_outline_on_light: f32 = 0.16;

/// Where the glass at one point of a lens looks: `through`, the point of
/// the lens whose content it shows there, `fringe`, how far red looks
/// short of it and blue past it, `rim`, how much of the highlight lies there,
/// `hairline`, how much of the light inner hairline, and `sheen`, how far
/// the glass lifts there.
struct AnnotationGlass {
  through: vec2<f32>,
  fringe: vec2<f32>,
  rim: f32,
  hairline: f32,
  sheen: f32,
}

/// How wide the glass's hairlines are on a lens whose shorter half side is
/// `size`, where one drawn pixel is `pixel` wide.
fn annotation_glass_hairline_width(size: f32, pixel: f32) -> f32 {
  return max(size * annotation_glass_hairline, pixel);
}

fn annotation_glass_luma(rgb: vec3<f32>) -> f32 {
  return dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
}

/// The bezel's height across it, 0 at the outer edge and 1 where it meets the
/// flat middle, and its slope: a squircle's.
fn annotation_glass_height(x: f32) -> f32 {
  return pow(max(1.0 - pow(1.0 - x, 4.0), 0.0), 0.25);
}

fn annotation_glass_slope(x: f32) -> f32 {
  let rest = 1.0 - x;
  return rest * rest * rest * pow(max(1.0 - pow(rest, 4.0), 1e-4), -0.75);
}

/// How far light through the bezel at `t`, 0 at the rim and 1 at the middle,
/// lands inwards of where it entered, by Snell's law through the surface
/// there and on down through the glass, a bezel `bezel` wide.
fn annotation_glass_refraction(t: f32, bezel: f32) -> f32 {
  let eta = 1.0 / annotation_glass_index;
  let slope = annotation_glass_slope(t);
  let normal = vec2<f32>(-slope, -1.0) / sqrt(slope * slope + 1.0);
  let k = max(1.0 - eta * eta * (1.0 - normal.y * normal.y), 0.0);
  let along = eta * normal.y + sqrt(k);
  let refracted = vec2<f32>(-along * normal.x, eta - along * normal.y);
  if (abs(refracted.y) < 1e-3) {
    return 0.0;
  }
  let depth = annotation_glass_height(t) * bezel + annotation_glass_thickness * bezel;
  return refracted.x * depth / refracted.y;
}

/// The way out of a rounded box at `offset` from its centre.
fn annotation_glass_normal(offset: vec2<f32>, half_size: vec2<f32>, radius: f32) -> vec2<f32> {
  let q = abs(offset) - (half_size - radius);
  var normal = select(vec2<f32>(0.0, 1.0), vec2<f32>(1.0, 0.0), q.x > q.y);
  if (q.x > 0.0 && q.y > 0.0) {
    normal = normalize(q);
  }
  return normal * select(vec2<f32>(-1.0), vec2<f32>(1.0), offset >= vec2<f32>(0.0));
}

/// The glass of a lens centred on `centre`, `half_size` each side of it and
/// rounded by `radius`, at `probe`, `presence` of the way settled onto the
/// picture, where one drawn pixel is `pixel` wide. A lens not yet settled is
/// flat: the glass comes with it as it lifts off the picture.
fn annotation_glass_at(probe: vec2<f32>, centre: vec2<f32>, half_size: vec2<f32>, radius: f32,
                       presence: f32, pixel: f32) -> AnnotationGlass {
  var glass = AnnotationGlass(probe, vec2<f32>(0.0), 0.0, 0.0, 0.0);
  let size = min(half_size.x, half_size.y);
  let offset = probe - centre;
  let rounding = min(radius, size);
  let inside = -annotation_magnify_box(probe, centre, half_size, rounding);
  if (presence <= 0.0 || size <= 0.0 || inside < -pixel) {
    return glass;
  }
  let bezel = size * annotation_glass_bezel;
  // The bend follows corners rounded at least as far as the bezel is wide,
  // so a square lens's bezel turns its corners smoothly rather than in a
  // seam along each diagonal.
  let bend_rounding = min(max(rounding, bezel), size);
  let normal = annotation_glass_normal(offset, half_size, bend_rounding);
  let depth = max(-annotation_magnify_box(probe, centre, half_size, bend_rounding), 0.0);
  let t = saturate(depth / bezel);
  if (t < 1.0) {
    let bend = min(annotation_glass_refraction(t, bezel) * 0.5,
                   bezel * annotation_glass_most_bend) * presence;
    glass.through = probe - normal * bend;
    // A part of a pixel's fringe would only cost two more looks.
    if (bend * annotation_glass_fringe > pixel * 0.05) {
      glass.fringe = normal * bend * annotation_glass_fringe;
    }
    let above = saturate(0.5 - 0.5 * normal.y);
    glass.sheen = annotation_glass_sheen * (1.0 - t) * (1.0 - t) * above * above * presence;
  }
  let facing = abs(dot(normal, annotation_glass_light));
  let width = max(size * annotation_glass_rim_width, pixel);
  let line = 1.0 - smoothstep(width * 0.5, width * 1.5, inside);
  glass.rim = line * mix(annotation_glass_rim_floor, 1.0, facing * facing * facing) * presence;
  // The hairline follows the lens's own edge, square corners and all, and
  // is a little brighter where the rim faces the light.
  let hairline = annotation_glass_hairline_width(size, pixel);
  glass.hairline = (1.0 - smoothstep(hairline - pixel * 0.5, hairline + pixel * 0.5, inside)) *
      (0.7 + 0.3 * facing) * presence;
  return glass;
}

/// The faint dark hairline round the outside of a lens whose shorter half
/// side is `size`, over `rgba`, at `outside` from its edge, `presence` of the
/// way settled onto the picture, where one drawn pixel is `pixel` wide: darker
/// over light content, which the inner hairline does not reach.
fn annotation_glass_outline_over(rgba: vec4<f32>, outside: f32, size: f32, presence: f32,
                                 pixel: f32) -> vec4<f32> {
  let width = annotation_glass_hairline_width(size, pixel);
  let band = saturate(0.5 + outside / pixel) *
      (1.0 - smoothstep(width - pixel * 0.5, width + pixel * 0.5, outside));
  if (band <= 0.0 || presence <= 0.0) {
    return rgba;
  }
  let light = saturate(annotation_glass_luma(rgba.rgb) / max(rgba.a, 1e-4));
  let dark = mix(annotation_glass_outline_on_dark, annotation_glass_outline_on_light, light);
  return vec4<f32>(rgba.rgb * (1.0 - dark * band * presence), rgba.a);
}

/// How many looks the glass takes at a point: one, or one per colour where
/// it parts them.
fn annotation_glass_looks(glass: AnnotationGlass) -> u32 {
  return select(1u, 3u, any(glass.fringe != vec2<f32>(0.0)));
}

/// The point of the lens look `look` of `looks` reads: red, green, then
/// blue where the colours part.
fn annotation_glass_look(glass: AnnotationGlass, look: u32, looks: u32) -> vec2<f32> {
  if (looks == 1u) {
    return glass.through;
  }
  return glass.through + glass.fringe * (1.0 - f32(look));
}

/// `content` with what look `look` of `looks` saw, `seen`, taken into it:
/// all of it from a single look, else its own colour, and the coverage from
/// green's.
fn annotation_glass_take(content: vec4<f32>, seen: vec4<f32>, look: u32, looks: u32)
    -> vec4<f32> {
  if (looks == 1u) {
    return seen;
  }
  switch look {
    case 0u: { return vec4<f32>(seen.r, content.gba); }
    case 1u: { return vec4<f32>(content.r, seen.g, content.b, seen.a); }
    default: { return vec4<f32>(content.rg, seen.b, content.a); }
  }
}

/// The glass's own light over `rgba`, the lens drawn there, which covers
/// `cover` of the pixel: the sheen under its upper rim, the highlight on its
/// rim, which first saturates what it lies over, and the light hairline,
/// clearer the darker what it lies over.
fn annotation_glass_light_over(rgba_in: vec4<f32>, glass: AnnotationGlass, cover: f32)
    -> vec4<f32> {
  if (cover <= 0.0 || (glass.rim <= 0.0 && glass.sheen <= 0.0 && glass.hairline <= 0.0)) {
    return rgba_in;
  }
  var rgba = rgba_in;
  let grey = vec3<f32>(annotation_glass_luma(rgba.rgb));
  let vivid = clamp(mix(grey, rgba.rgb, annotation_glass_rim_saturation), vec3<f32>(0.0),
                    vec3<f32>(rgba.a));
  rgba = vec4<f32>(mix(rgba.rgb, vivid, min(glass.rim * 2.0, 1.0) * 0.5 * cover), rgba.a);
  let light = saturate(grey.x / max(rgba.a, 1e-4));
  let hairline = glass.hairline *
      mix(annotation_glass_hairline_on_dark, annotation_glass_hairline_on_light, light);
  let lift = saturate(glass.sheen + max(glass.rim * annotation_glass_rim_opacity, hairline)) *
      cover;
  return annotation_over(rgba, vec3<f32>(1.0), lift);
}
