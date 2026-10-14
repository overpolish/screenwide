// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The text box: the box and its pointer read out of the slots an arrow fills
// with its curve, the distance to them, its type sampled from the atlas, and
// the layer that draws them.

/// A text box read out of the slots an arrow fills with its curve. Prepared
/// once per annotation by Rust's `text::geometry::prepare_text`, which
/// documents every slot.
struct PreviewTextBox {
  low: vec2<f32>,
  high: vec2<f32>,
  radius: f32,
  base: vec2<f32>,
  tip: vec2<f32>,
  base_radius: f32,
  tip_radius: f32,
  blend: f32,
  text_origin: vec2<f32>,
  text_size: vec2<f32>,
  text_centre: vec2<f32>,
}

fn annotation_text_box(prepared: PreviewGeometry) -> PreviewTextBox {
  var box: PreviewTextBox;
  box.low = vec2<f32>(prepared.ax, prepared.ay);
  box.high = vec2<f32>(prepared.bx, prepared.by);
  box.radius = prepared.rounding;
  box.base = vec2<f32>(prepared.end_head_ax, prepared.end_head_ay);
  box.tip = vec2<f32>(prepared.end_head_bx, prepared.end_head_by);
  box.base_radius = prepared.high;
  box.tip_radius = prepared.low;
  box.blend = prepared.end_head_cx;
  box.text_origin = vec2<f32>(prepared.start_head_ax, prepared.start_head_ay);
  box.text_size = vec2<f32>(prepared.start_head_bx, prepared.start_head_by);
  box.text_centre = vec2<f32>(prepared.start_head_cx, prepared.start_head_cy);
  return box;
}

fn annotation_rounded_box_distance(probe: vec2<f32>, low: vec2<f32>, high: vec2<f32>,
                                   radius: f32) -> f32 {
  let half_size = (high - low) * 0.5;
  let q = abs(probe - (low + high) * 0.5) - half_size + radius;
  return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - radius;
}

/// A stroke tapering from a circle at `base` to a smaller one at `tip`, its
/// sides tangent to both. The per-pixel twin of `tapered_distance` in
/// `annotations/text/geometry.rs`.
fn annotation_tapered_distance(probe: vec2<f32>, base: vec2<f32>, tip: vec2<f32>,
                               base_radius: f32, tip_radius: f32) -> f32 {
  let axis = tip - base;
  let span = dot(axis, axis);
  let taper = base_radius - tip_radius;
  let local = probe - base;
  if (span <= taper * taper) {
    return min(length(local) - base_radius, length(probe - tip) - tip_radius);
  }
  let q = vec2<f32>(abs(local.x * axis.y - local.y * axis.x), dot(local, axis)) / span;
  let side = vec2<f32>(sqrt(span - taper * taper), taper);
  let cross_value = side.x * q.y - side.y * q.x;
  if (cross_value < 0.0) {
    return sqrt(span * dot(q, q)) - base_radius;
  }
  if (cross_value > side.x) {
    return sqrt(span * (dot(q, q) + 1.0 - 2.0 * q.y)) - tip_radius;
  }
  return dot(side, q) - base_radius;
}

/// How far a point falls outside a text box, pointer included: the box and
/// the pointer blended over `blend`, so the pointer leaves the edge in a
/// curve. The per-pixel twin of `text_distance`.
fn annotation_text_distance(probe: vec2<f32>, box: PreviewTextBox) -> f32 {
  let body = annotation_rounded_box_distance(probe, box.low, box.high, box.radius);
  if (box.base_radius <= 0.0) {
    return body;
  }
  let pointer = annotation_tapered_distance(probe, box.base, box.tip, box.base_radius,
                                            box.tip_radius);
  if (box.blend <= 0.0) {
    return min(body, pointer);
  }
  let mix_amount = max(box.blend - abs(body - pointer), 0.0) / box.blend;
  return min(body, pointer) - mix_amount * mix_amount * box.blend * 0.25;
}

/// The atlas over this pixel, rasterised at `atlas.scale` pixels to the
/// canvas pixel and centred on the text block, drawn `growth` times the size
/// it was rasterised at: the type's coverage in alpha, and a caption word's
/// outline in red. `feather` is half a drawn pixel, in canvas pixels, which
/// is how far the atlas read spreads.
fn annotation_text_texel(probe: vec2<f32>, box: PreviewTextBox, atlas: AnnotationTextAtlas,
                         feather: f32, growth: f32) -> vec4<f32> {
  if (atlas.size.x == 0u || atlas.size.y == 0u || atlas.scale <= 0.0 ||
      box.text_size.x <= 0.0 || box.text_size.y <= 0.0 || growth <= 0.0) {
    return vec4<f32>(0.0);
  }
  let drawn = box.text_size / atlas.scale;
  let local = (probe - box.text_centre) / growth + drawn * 0.5;
  if (any(local < vec2<f32>(0.0)) || any(local > drawn)) {
    return vec4<f32>(0.0);
  }
  return annotation_atlas_footprint(atlas, box.text_origin, box.text_size,
                                    box.text_origin + local * atlas.scale,
                                    2.0 * feather * atlas.scale / growth);
}

/// How much of the type covers this pixel; see [`annotation_text_texel`].
fn annotation_text_coverage(probe: vec2<f32>, box: PreviewTextBox, atlas: AnnotationTextAtlas,
                            feather: f32) -> f32 {
  return annotation_text_texel(probe, box, atlas, feather, 1.0).a;
}

/// A text box averaged over its exposure samples, as a counter is while it
/// grows into place. A turned caption piece is read where each sample's
/// turn takes this pixel back onto it.
fn annotation_text_exposure(probe: vec2<f32>, annotation: PreviewArrow, feather: f32) -> f32 {
  let turned = (annotation.flags & annotation_text_turned) != 0u;
  var total = 0.0;
  for (var tap = 0u; tap < annotation.sample_count; tap++) {
    let sample = annotation_samples[annotation.sample_first + tap];
    let at = annotation_caption_unturned(sample.geometry, turned, probe, feather);
    if (!at.seen) {
      continue;
    }
    let distance = annotation_text_distance(at.point, annotation_text_box(sample.geometry));
    total += annotation_edge(distance, at.feather) * sample.opacity;
  }
  return total / f32(annotation.sample_count);
}

/// `bounds` (low in `xy`, high in `zw`) grown to hold a text box and its
/// pointer, `reach` beyond.
fn annotation_text_bounds(box: PreviewTextBox, reach: f32, bounds: vec4<f32>) -> vec4<f32> {
  var low = min(bounds.xy, box.low - reach);
  var high = max(bounds.zw, box.high + reach);
  if (box.base_radius > 0.0) {
    low = min(low, box.tip - box.tip_radius - reach);
    high = max(high, box.tip + box.tip_radius + reach);
  }
  return vec4<f32>(low, high);
}

/// Where a text box and its pointer can reach. The exposure samples run
/// steadily from last frame's reveal to this one's, so the first and last
/// hold every one between: a pointer drawing back in trails past where its
/// tip is now. A caption's word reaches as far again as its shadow falls
/// and spreads. A turned caption piece carries the bounds every sample
/// reaches, its shadow's included, as the CPU found them.
fn annotation_text_reach(annotation: PreviewArrow, halo: f32, feather: f32) -> vec4<f32> {
  var reach = halo + feather + 1.0;
  if ((annotation.flags & annotation_text_turned) != 0u) {
    let turn = annotation.geometry.turn;
    return vec4<f32>(turn[9], turn[10], turn[11], turn[12]) + vec4<f32>(-reach, -reach, reach, reach);
  }
  if ((annotation.flags & annotation_text_casts) != 0u) {
    let growth = max(annotation_caption_growth(annotation.geometry, annotation.geometry), 1.0);
    let shadow = annotation_caption_shadow_of(annotation);
    reach += (length(shadow.offset) + shadow.softness) * growth;
  }
  var bounds = annotation_text_bounds(annotation_text_box(annotation.geometry), reach,
                                      annotation_no_bounds);
  if (annotation.sample_count > 0u) {
    let last = annotation.sample_first + annotation.sample_count - 1u;
    bounds = annotation_text_bounds(
        annotation_text_box(annotation_samples[annotation.sample_first].geometry), reach, bounds);
    bounds = annotation_text_bounds(annotation_text_box(annotation_samples[last].geometry), reach,
                                    bounds);
  }
  return bounds;
}

/// The flags a text box reads; the twins of `flags.rs`.
const annotation_text_outlined: u32 = 1u << 7u;
const annotation_text_caption: u32 = 1u << 10u;
const annotation_text_bare: u32 = 1u << 11u;
const annotation_text_casts: u32 = 1u << 12u;
const annotation_text_turned: u32 = 1u << 13u;

/// A pixel taken back onto a caption piece as it lies unturned, through
/// the inverse of its turn that `geometry` carries, with how far half a
/// pixel there spreads, the larger of across and down: a piece turned near
/// edge on is read over the many of its texels each pixel covers. Not seen
/// where the pixel lies past the piece's horizon. An unturned piece is read
/// where the pixel is.
struct CaptionProbe {
  point: vec2<f32>,
  feather: f32,
  seen: bool,
}

fn annotation_caption_unturned(geometry: PreviewGeometry, turned: bool, probe: vec2<f32>,
                               feather: f32) -> CaptionProbe {
  var at = CaptionProbe(probe, feather, true);
  if (!turned) {
    return at;
  }
  let turn = geometry.turn;
  let across = vec3<f32>(turn[0], turn[1], turn[2]);
  let down = vec3<f32>(turn[3], turn[4], turn[5]);
  let depth = vec3<f32>(turn[6], turn[7], turn[8]);
  let point = vec3<f32>(probe, 1.0);
  let w = dot(depth, point);
  if (w <= 0.0) {
    at.seen = false;
    return at;
  }
  at.point = vec2<f32>(dot(across, point), dot(down, point)) / w;
  // The turn's derivative along each canvas axis, a homography's being its
  // linear part less the point's share of the depth, over the depth.
  let along_x = (vec2<f32>(across.x, down.x) - at.point * depth.x) / w;
  let along_y = (vec2<f32>(across.y, down.y) - at.point * depth.y) / w;
  at.feather = feather * max(length(along_x), length(along_y));
  return at;
}

/// The shadow a caption's word casts, read out of the slots a text box
/// leaves to its pointer, which a word has none of: how far it falls across
/// and down and how far its edge spreads, in canvas pixels at the word's
/// settled size, and its colour, packed as eight bits a channel.
struct CaptionShadow {
  offset: vec2<f32>,
  softness: f32,
  color: vec4<f32>,
}

fn annotation_caption_shadow_of(annotation: PreviewArrow) -> CaptionShadow {
  var shadow: CaptionShadow;
  shadow.offset = vec2<f32>(annotation.geometry.end_head_ax, annotation.geometry.end_head_ay);
  shadow.softness = annotation.geometry.end_head_bx;
  shadow.color = unpack4x8unorm(bitcast<u32>(annotation.geometry.end_head_by));
  return shadow;
}

/// How many times its settled size, which its arrow keeps in `cx`, a word
/// set as `geometry` is drawn.
fn annotation_caption_growth(arrow: PreviewGeometry, geometry: PreviewGeometry) -> f32 {
  return select(1.0, geometry.width / arrow.cx, arrow.cx > 0.0);
}

/// A caption word's type over this pixel as `box` sets it, `growth` times
/// its settled size: its type in alpha, its outline in red, and in green the
/// shadow it casts, its outline blurred in the atlas and read set off by
/// `offset`, grown with the word.
fn annotation_caption_texel(probe: vec2<f32>, box: PreviewTextBox, atlas: AnnotationTextAtlas,
                            feather: f32, growth: f32, casts: bool,
                            offset: vec2<f32>) -> vec4<f32> {
  var texel = annotation_text_texel(probe, box, atlas, feather, growth);
  texel.g = 0.0;
  if (casts) {
    texel.g = annotation_text_texel(probe - offset * growth, box, atlas, feather, growth).g;
  }
  return texel;
}

/// `texel`, a caption word's type, outline and shadow over a pixel, laid
/// over `rgba_in`, each as solid as `solid` and the shadow as its colour
/// says: the shadow under the outline under the type.
fn annotation_caption_layers(rgba_in: vec4<f32>, annotation: PreviewArrow, color: vec4<f32>,
                             texel: vec4<f32>, solid: f32) -> vec4<f32> {
  var rgba = rgba_in;
  if ((annotation.flags & annotation_text_casts) != 0u) {
    let shadow = annotation_caption_shadow_of(annotation).color;
    rgba = annotation_over(rgba, shadow.rgb, texel.g * shadow.a * solid);
  }
  if ((annotation.flags & annotation_text_outlined) != 0u) {
    let outline = vec3<f32>(annotation.params[0], annotation.params[1], annotation.params[2]);
    rgba = annotation_over(rgba, outline, texel.r * solid * 0.85);
  }
  return annotation_over(rgba, color.rgb, texel.a * solid);
}

/// A still caption word: its type alone in the annotation's colour, over an
/// outline where it is outlined, in the colour its parameters carry, and
/// the shadow it casts, with no box drawn. The outline and the shadow are
/// set in the atlas beside the type and read with it, so all share one
/// smooth edge. `solid` is how solid the word is.
///
/// A word's type is rasterised once, at the size it settles at, which its
/// geometry keeps in `cx`; a word growing in is that type drawn larger or
/// smaller, so growing never rasterises it again.
fn annotation_caption_word(rgba_in: vec4<f32>, annotation: PreviewArrow, color: vec4<f32>,
                           canvas_point: vec2<f32>, box: PreviewTextBox, solid: f32,
                           feather: f32, atlas: AnnotationTextAtlas) -> vec4<f32> {
  let growth = annotation_caption_growth(annotation.geometry, annotation.geometry);
  let casts = (annotation.flags & annotation_text_casts) != 0u;
  let offset = annotation_caption_shadow_of(annotation).offset;
  let texel = annotation_caption_texel(canvas_point, box, atlas, feather, growth, casts, offset);
  return annotation_caption_layers(rgba_in, annotation, color, texel, solid);
}

/// A moving caption word: its type, outline and shadow read at every exposure
/// sample, where, as large and as far turned as the word was then, each as
/// solid as the word was then, and averaged, so the type itself smears along
/// the way it came, as a moving annotation's shape does. The atlas places the
/// type once, for the word as it is now, and every sample reads it from there.
fn annotation_caption_word_exposure(rgba_in: vec4<f32>, annotation: PreviewArrow,
                                    color: vec4<f32>, canvas_point: vec2<f32>, feather: f32,
                                    atlas: AnnotationTextAtlas) -> vec4<f32> {
  let current = annotation_text_box(annotation.geometry);
  let casts = (annotation.flags & annotation_text_casts) != 0u;
  let turned = (annotation.flags & annotation_text_turned) != 0u;
  let offset = annotation_caption_shadow_of(annotation).offset;
  var ink = vec4<f32>(0.0);
  for (var tap = 0u; tap < annotation.sample_count; tap++) {
    let sample = annotation_samples[annotation.sample_first + tap];
    let at = annotation_caption_unturned(sample.geometry, turned, canvas_point, feather);
    if (!at.seen) {
      continue;
    }
    var box = annotation_text_box(sample.geometry);
    box.text_origin = current.text_origin;
    box.text_size = current.text_size;
    let growth = annotation_caption_growth(annotation.geometry, sample.geometry);
    ink += annotation_caption_texel(at.point, box, atlas, at.feather, growth, casts, offset) *
           sample.opacity;
  }
  ink /= f32(annotation.sample_count);
  return annotation_caption_layers(rgba_in, annotation, color, ink, color.a);
}

/// One text box: its box and pointer in the annotation's own colour, and its
/// type in whichever of black or white reads on that colour, clipped to the
/// box's own coverage so the two share one antialiased edge. A caption's
/// word draws its type alone, and a caption's backdrop its box alone; either
/// read, where it turns, where its turn takes the pixel back onto it.
fn annotation_text_layer(rgba_in: vec4<f32>, annotation: PreviewArrow, color: vec4<f32>,
                         canvas_point: vec2<f32>, feather: f32, halo: f32,
                         atlas: AnnotationTextAtlas) -> vec4<f32> {
  if (annotation_outside(canvas_point, annotation_text_reach(annotation, halo, feather))) {
    return rgba_in;
  }
  let turned = (annotation.flags & annotation_text_turned) != 0u;
  let at = annotation_caption_unturned(annotation.geometry, turned, canvas_point, feather);
  // Edge on now, a still piece shows nothing; a moving one is drawn by its
  // samples alone.
  if (!at.seen && annotation.sample_count == 0u) {
    return rgba_in;
  }
  let box = annotation_text_box(annotation.geometry);
  let distance = annotation_text_distance(at.point, box);
  var rgba = annotation_halo(rgba_in, color, distance, halo, at.feather);
  // A caption's word draws no box: its type, outline and shadow are its
  // own, and the shadow falls outside the box it is set in.
  if ((annotation.flags & annotation_text_caption) != 0u) {
    if (annotation.sample_count > 0u) {
      return annotation_caption_word_exposure(rgba, annotation, color, canvas_point, feather,
                                              atlas);
    }
    return annotation_caption_word(rgba, annotation, color, at.point, box, color.a, at.feather,
                                   atlas);
  }
  var coverage: f32;
  if (annotation.sample_count == 0u) {
    coverage = annotation_edge(distance, at.feather);
  } else {
    coverage = annotation_text_exposure(canvas_point, annotation, feather);
  }
  if (coverage <= 0.0) {
    return rgba;
  }
  let alpha = coverage * color.a;
  rgba = annotation_over(rgba, color.rgb, alpha);
  if ((annotation.flags & annotation_text_bare) != 0u) {
    return rgba;
  }
  let ink = annotation_text_coverage(canvas_point, box, atlas, feather) * alpha;
  if (ink <= 0.0) {
    return rgba;
  }
  // The counter's rule, so a text box and a counter in one colour carry the
  // same ink.
  return annotation_over(rgba, annotation_label_tint(color.rgb), ink);
}
