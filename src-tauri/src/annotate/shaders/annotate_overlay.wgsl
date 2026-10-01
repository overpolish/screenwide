// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The live overlay's one pass, the twin of `native_overlay_macos_shader.h`.
// Assembled with the editor's annotation pieces by `build/wgsl.rs`; this part
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
// desktop, and its blur is `annotate_softened`, the same desktop softened once
// in Rust, laid under the shade where the blur reaches. A 1x1 softened texture
// is none at all.
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
  spare: f32,
}

fn overlay_target() -> vec2<f32> {
  return vec2<f32>(overlay.target_width, overlay.target_height);
}

@group(0) @binding(0) var<uniform> overlay: Overlay;
// What the highlights recolour, laid over the whole target.
@group(0) @binding(1) var annotate_underlay: texture_2d<f32>;
// The same softened for the spotlights' blur, stretched over the target.
@group(0) @binding(2) var annotate_softened: texture_2d<f32>;

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

fn annotation_cursor_blur() -> AnnotationCursorBlur {
  return AnnotationCursorBlur(0.0, 0.0, 0u);
}

@vertex
fn vs_main(@builtin(vertex_index) id: u32) -> @builtin(position) vec4<f32> {
  let position = vec2<f32>(f32((id << 1u) & 2u), f32(id & 2u));
  return vec4<f32>(position * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 0.0, 1.0);
}

// The softened desktop at `across`, a share of the target, read between its
// four nearest pixels: it is a fraction of the target's size, and read pixel
// by pixel it would show its blocks.
fn annotate_softened_at(across: vec2<f32>, size: vec2<u32>) -> vec3<f32> {
  let texel = across * vec2<f32>(size) - 0.5;
  let base = floor(texel);
  let t = texel - base;
  let last = vec2<i32>(size) - 1;
  let low = clamp(vec2<i32>(base), vec2<i32>(0), last);
  let high = clamp(vec2<i32>(base) + 1, vec2<i32>(0), last);
  let top = mix(textureLoad(annotate_softened, vec2<i32>(low.x, low.y), 0).rgb,
                textureLoad(annotate_softened, vec2<i32>(high.x, low.y), 0).rgb, t.x);
  let bottom = mix(textureLoad(annotate_softened, vec2<i32>(low.x, high.y), 0).rgb,
                   textureLoad(annotate_softened, vec2<i32>(high.x, high.y), 0).rgb, t.x);
  return mix(top, bottom, t.y);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
  let size = textureDimensions(annotate_underlay);
  let across = position.xy / max(overlay_target(), vec2<f32>(1.0));
  let texel = min(vec2<u32>(across * vec2<f32>(size)), size - 1u);
  let base = vec4<f32>(textureLoad(annotate_underlay, vec2<i32>(texel), 0).rgb, 1.0);
  // Composed over nothing, so the result is already premultiplied - which is
  // what DirectComposition expects of a premultiplied swap chain. The blur is
  // the softened desktop laid over the live one as far as it reaches, and the
  // shade darkens both: what shows through is the desktop times what the blur
  // leaves of it times what the shade leaves.
  var result = vec4<f32>(0.0);
  let soft_size = textureDimensions(annotate_softened);
  if (soft_size.x > 1u) {
    let blur = annotation_spotlight_cover(position.xy, 0u, overlay.count, overlay.feather, true);
    if (blur > 0.0) {
      result = vec4<f32>(annotate_softened_at(across, soft_size) * blur, blur);
    }
  }
  let lift = 1.0 - annotation_spotlight_dim *
      annotation_spotlight_cover(position.xy, 0u, overlay.count, overlay.feather, false);
  result = vec4<f32>(result.rgb * lift, 1.0 - (1.0 - result.a) * lift);
  result = composite_highlights(result, base, position.xy, 0u, overlay.count, overlay.feather);
  let atlas = AnnotationTextAtlas(overlay.number_atlas, overlay.number_scale);
  return composite_annotations(result, position.xy, 0u, overlay.count, overlay.feather, atlas);
}
