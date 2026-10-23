// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The live overlay's one pass. Assembled with the editor's annotation pieces
// and its focus falloff by `build/wgsl.rs`; this part
// comes first because it carries the module's directive.
//
// Everything that decides what an annotation looks like - the curve solve, the
// head geometry, the coverage and its feathering - is the editor's own
// `composite_annotations`, from the same source the still and the export
// compose through. This shader only chooses the surface: a transparent
// target, one annotation list, no camera layer.
//
// A highlight recolours what is under it, which the overlay never sees: it
// reads `annotate_underlay` instead, the desktop captured when it was drawn, or
// the still it is baked into. A spotlight's shade is black laid over the
// desktop. A focus's blur is `annotate_focus`, the same desktop with its mip
// chain, read as the editor's focus reads its frame (`focus_falloff.wgsl`,
// `disc_blur.wgsl`) and laid over the live desktop where the blur reaches.
//
// Annotations arrive in this display's layer pixels, so there is no canvas
// placement to apply and one drawn pixel is one annotation pixel.
//
// The annotation pieces sample their textures inside branches on the pixel's
// own geometry, where WGSL's uniformity analysis cannot prove the implicit
// derivatives are well defined; they are, as in the editor's canvas.
diagnostic(off, derivative_uniformity);

// The twin of `Constants` in `renderer.rs`. The target's size is two scalars:
// a `vec2` there would be aligned to eight bytes and move off Rust's offset.
struct Overlay {
  // How many prepared annotations `annotation_arrows` holds.
  count: u32,
  // How wide an edge is smoothed, in layer pixels.
  feather: f32,
  // The size of the texture the counters' numbers were rasterised into, or
  // zeroes on a frame with no counter to rasterise one for.
  number_atlas: vec2<u32>,
  // How many atlas pixels that texture holds per layer pixel.
  number_scale: f32,
  // The target's size in pixels, which the underlay is read across.
  target_width: f32,
  target_height: f32,
  // How many focus boxes `annotate_focus_regions` holds, none without a
  // chain to blur; how far past a box its blur takes to arrive; and how wide
  // the middle strength's blur is, both in target pixels.
  focus_count: u32,
  focus_reach: f32,
  focus_widest: f32,
  spare_x: f32,
  spare_y: f32,
}

fn overlay_target() -> vec2<f32> {
  return vec2<f32>(overlay.target_width, overlay.target_height);
}

@group(0) @binding(0) var<uniform> overlay: Overlay;
// What the highlights recolour, laid over the whole target.
@group(0) @binding(1) var annotate_underlay: texture_2d<f32>;
// The same with its mip chain, for a focus's blur.
@group(0) @binding(2) var annotate_focus: texture_2d<f32>;
@group(0) @binding(3) var annotate_focus_sampler: sampler;
@group(0) @binding(4) var<storage, read> annotate_focus_regions: array<FocusRegion>;

// The overlay offers no magnifier, but its highlights read the picture to
// find each glyph's full ink: the underlay, laid over the whole target.
fn annotation_magnify_placement() -> AnnotationMagnifyPlacement {
  let size = vec2<f32>(textureDimensions(annotate_underlay));
  var at: AnnotationMagnifyPlacement;
  at.size = size;
  at.image = vec4<f32>(0.0, 0.0, overlay_target());
  at.texels = vec4<f32>(0.0, 0.0, size - 1.0);
  return at;
}

fn annotation_magnify_fetch(texel: vec2<i32>) -> vec4<f32> {
  return textureLoad(annotate_underlay, texel, 0);
}

// Nor a cursor: the live desktop's own pointer is not drawn into the overlay.
fn annotation_cursor_sample(probe: vec2<f32>) -> vec4<f32> {
  return vec4<f32>(0.0);
}

fn annotation_spotlight_blur() -> AnnotationSpotlightBlur {
  return AnnotationSpotlightBlur(0.0, 0.0, 0u);
}

fn annotation_cursor_blur_layer() -> bool {
  return false;
}

fn annotation_cursor_blurred(point: vec2<f32>) -> vec4<f32> {
  return vec4<f32>(0.0);
}

fn annotation_cursor_near(probe: vec2<f32>, margin: f32) -> bool {
  return false;
}

// The overlay draws a handful of annotations, so every pass walks them all.
fn annotation_next(probe: vec2<f32>, start: u32, last: u32) -> u32 {
  return min(start, last);
}

@vertex
fn vs_main(@builtin(vertex_index) id: u32) -> @builtin(position) vec4<f32> {
  let position = vec2<f32>(f32((id << 1u) & 2u), f32(id & 2u));
  return vec4<f32>(position * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 0.0, 1.0);
}

// How wide the focus blur is at target pixel `pixel`, in target pixels: the
// least any box asks for, so a pixel any box keeps sharp stays sharp.
fn annotate_focus_width(pixel: vec2<f32>) -> f32 {
  var width = overlay.focus_widest * 4.0;
  for (var index = 0u; index < overlay.focus_count; index++) {
    let region = annotate_focus_regions[index];
    let amount = focus_falloff(focus_box_distance(pixel, region), overlay.focus_reach);
    width = min(width, focus_width(region, overlay.focus_widest, amount));
  }
  return width;
}

// The desktop under target pixel `pixel` as a focus softens it, laid over
// the live desktop as far as the blur reaches. Where the blur is under a
// pixel wide nothing is laid, so the live desktop shows rather than the
// still the blur is read from.
fn annotate_focus_layer(pixel: vec2<f32>) -> vec4<f32> {
  if (overlay.focus_count == 0u) {
    return vec4<f32>(0.0);
  }
  let width = annotate_focus_width(pixel);
  let shown = smoothstep(0.5, 1.5, width);
  if (shown <= 0.0) {
    return vec4<f32>(0.0);
  }
  let size = vec2<f32>(textureDimensions(annotate_focus));
  let per = size / max(overlay_target(), vec2<f32>(1.0));
  let blurred = disc_blur(annotate_focus, annotate_focus_sampler, pixel * per, width * per.x,
                          vec2<f32>(0.5), size - 0.5);
  return vec4<f32>(blurred.rgb * shown, shown);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
  let size = textureDimensions(annotate_underlay);
  let across = position.xy / max(overlay_target(), vec2<f32>(1.0));
  let texel = min(vec2<u32>(across * vec2<f32>(size)), size - 1u);
  let base = vec4<f32>(textureLoad(annotate_underlay, vec2<i32>(texel), 0).rgb, 1.0);
  // Composed over nothing, so the result is already premultiplied - which is
  // what DirectComposition expects of a premultiplied swap chain. The focus's
  // blur is laid over the live desktop as far as it reaches, and a dimmed
  // focus's shade darkens both: what shows through is the desktop times what
  // the blur leaves of it times what the shade leaves.
  var result = annotate_focus_layer(position.xy);
  let lift = 1.0 - annotation_spotlight_dim *
      annotation_spotlight_cover(position.xy, 0u, overlay.count, overlay.feather, false);
  result = vec4<f32>(result.rgb * lift, 1.0 - (1.0 - result.a) * lift);
  result = composite_highlights(result, base, position.xy, 0u, overlay.count, overlay.feather);
  let atlas = AnnotationTextAtlas(overlay.number_atlas, overlay.number_scale);
  return composite_annotations(result, position.xy, 0u, overlay.count, overlay.feather, atlas);
}
