// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The editor's annotations in document order, the first at the bottom, each
// covering everything under it. A magnifier's loupe shows everything below
// it enlarged. The spotlights' one shared shade lies where the topmost of
// them sits: it darkens the picture and every mark below it, and their blur
// softens those marks as it softens the picture, whose blur the source
// already carries. The marks' blur is a Gaussian laid by
// `compositor/mark_blur.rs` into `annotation_blur_layer`, which a pass of this
// shader draws by stopping under the shade. A highlight recolours the pixel
// as everything under it drew it but the highlights, so overlapping
// highlights merge rather than one recolouring the other into its inverse.
//
// WGSL has no recursion, so what lies under a loupe is drawn by its own
// pass, a level down: the canvas (`composite_annotation_layers`) draws its
// loupes through `annotation_loupe_content`, which draws the loupes inside it
// with the picture alone.

/// A pixel as drawn so far, premultiplied, and the same drawn without the
/// highlights once one has reached it, which is what a highlight recolours.
struct AnnotationStack {
  rgba: vec4<f32>,
  bare: vec4<f32>,
  highlighted: bool,
}


/// The topmost spotlight in `[first, last)` that may reach `point`, or `last`.
fn annotation_shade_at(point: vec2<f32>, first: u32, last: u32) -> u32 {
  var shade_at = last;
  for (var index = annotation_next(point, first, last); index < last;
       index = annotation_next(point, index + 1u, last)) {
    if (annotation_arrows[index].kind == annotation_spotlight_kind) {
      shade_at = index;
    }
  }
  return shade_at;
}

/// Lays one annotation over `stack`, a magnifier showing the picture alone:
/// a highlight recolours it, a spotlight adds its hover halo, a sticker lays
/// its picture over it, and everything else draws itself, once over what is
/// drawn and again over `bare` while a highlight has made the two differ.
/// The draw has the one call site, looped over the two, because the GPU
/// compiler inlines every call site whole.
fn annotation_stack_draw(stack_in: AnnotationStack, annotation: PreviewArrow, point: vec2<f32>,
                         feather: f32, number_atlas: AnnotationTextAtlas, cursor: bool)
    -> AnnotationStack {
  var stack = stack_in;
  if (annotation_kind_drawn(annotation_highlight_kind) &&
      annotation.kind == annotation_highlight_kind) {
    let under = stack.rgba;
    stack.rgba = annotation_highlight_layer(stack.rgba, stack.bare, annotation, point, feather);
    stack.highlighted = stack.highlighted || any(stack.rgba != under);
    return stack;
  }
  // Only a highlight sets `highlighted`; asked of the constant, the second
  // side and its draw drop out of a module built without highlights.
  let sides = select(1u, 2u,
                     annotation_kind_drawn(annotation_highlight_kind) && stack.highlighted);
  for (var side = 0u; side < sides; side++) {
    let under = select(stack.rgba, stack.bare, side == 1u);
    var drawn: vec4<f32>;
    if (annotation_acts_on_picture(annotation.kind)) {
      drawn = annotation_picture_layer(under, annotation, point, feather, cursor);
    } else if (annotation_kind_drawn(annotation_sticker_kind) &&
               annotation.kind == annotation_sticker_kind) {
      drawn = annotation_sticker_layer(under, annotation, point, feather);
    } else {
      drawn = annotation_layer(under, annotation, point, feather, number_atlas);
    }
    if (side == 0u) {
      stack.rgba = drawn;
    } else {
      stack.bare = drawn;
    }
  }
  if (!stack.highlighted) {
    stack.bare = stack.rgba;
  }
  return stack;
}

/// Lays the spotlights' shade over `stack`, which `[first, shade_at)` drew
/// over `base`, the canvas at `point` before any annotation: where this pass
/// reads the mark blur's layer for this shade, what they drew blurred, as far
/// as the blur reaches, then the shade itself.
fn annotation_stack_shaded(stack_in: AnnotationStack, base: vec4<f32>, point: vec2<f32>,
                           first: u32, shade_at: u32, last: u32, feather: f32)
    -> AnnotationStack {
  var stack = stack_in;
  if (annotation_blur_mode() == 2u && annotation_blur_marks() &&
      shade_at == annotation_blur_spotlight()) {
    let share = annotation_spotlight_blur_share(point, annotation_spotlight_blur(),
                                                max(2.0 * feather, 1e-4));
    if (share > 0.0) {
      let soft = saturate(base + annotation_marks_blurred(point));
      // What lies under a highlight takes the same blur as what it drew over.
      let lift = (soft - stack.rgba) * share;
      stack.rgba += lift;
      stack.bare += lift;
    }
  }
  let dim = 1.0 - annotation_spotlight_dim *
      annotation_spotlight_cover(point, first, last, feather, false);
  stack.rgba = vec4<f32>(stack.rgba.rgb * dim, stack.rgba.a);
  stack.bare = vec4<f32>(stack.bare.rgb * dim, stack.bare.a);
  return stack;
}

/// The cursor over `rgba` at `point`, as `[first, last)` treat it: lying on
/// the picture, so darkened by the spotlights' shade and hidden by every
/// loupe, but drawn over every mark. `[0, shade_last)` holds the spotlights
/// whose shade it lies under here, none where the shade is laid in a later
/// run, which darkens it there. `pixel` is one drawn pixel there.
fn annotation_stack_cursor(rgba: vec4<f32>, point: vec2<f32>, first: u32, last: u32,
                           shade_last: u32, feather: f32, pixel: f32) -> vec4<f32> {
  var pointer = annotation_cursor_seen(point, pixel);
  if (pointer.a <= 0.0) {
    return rgba;
  }
  pointer = vec4<f32>(pointer.rgb * (1.0 - annotation_spotlight_dim *
      annotation_spotlight_cover(point, 0u, shade_last, feather, false)), pointer.a);
  var hidden = 0.0;
  for (var loupe = annotation_next(point, first, last); loupe < last;
       loupe = annotation_next(point, loupe + 1u, last)) {
    if (annotation_kind_drawn(annotation_magnify_kind) &&
        annotation_arrows[loupe].kind == annotation_magnify_kind) {
      hidden = max(hidden, annotation_magnify_cover(annotation_arrows[loupe], point, feather));
    }
  }
  pointer.a *= 1.0 - hidden;
  return vec4<f32>(mix(rgba.rgb, pointer.rgb, pointer.a), pointer.a + rgba.a * (1.0 - pointer.a));
}

/// What a loupe shows at canvas point `point` of its zoom area, premultiplied:
/// the picture and `[first, last)`, everything below the magnifier, shaded and
/// blurred as the canvas draws them, and the cursor where `cursor` says the
/// layer carries it. A loupe over the camera shows the camera too, laid
/// between the annotations under it and those over it as the canvas lays it.
/// One loupe pixel spans `span` canvas pixels there, and `feather` is half of
/// the smaller.
fn annotation_loupe_content(point: vec2<f32>, span: vec2<f32>, first: u32, last: u32,
                            feather: f32, number_atlas: AnnotationTextAtlas, cursor: bool)
    -> vec4<f32> {
  let picture = annotation_magnify_sample(point, span);
  let base = vec4<f32>(picture.rgb * picture.a, picture.a);
  let shade_at = annotation_shade_at(point, first, last);
  let camera_at = annotation_camera_run();
  var stack = AnnotationStack(base, base, false);
  // The camera lies under the annotation at `camera_at`, so under the
  // magnifier itself when that is the magnifier.
  var camera_laid = camera_at > last;
  for (var index = annotation_next(point, first, last); index < last;
       index = annotation_next(point, index + 1u, last)) {
    if (!camera_laid && index >= camera_at) {
      stack = annotation_stack_camera(stack, point);
      camera_laid = true;
    }
    if (index == shade_at) {
      stack = annotation_stack_shaded(stack, base, point, first, index, last, feather);
    }
    stack = annotation_stack_draw(stack, annotation_arrows[index], point, feather, number_atlas,
                                  cursor);
  }
  if (!camera_laid) {
    stack = annotation_stack_camera(stack, point);
  }
  if (!cursor) {
    return stack.rgba;
  }
  return annotation_stack_cursor(stack.rgba, point, first, last, last, feather, 2.0 * feather);
}

/// `stack` with the camera laid over it at `point`, both what is drawn and
/// what a highlight recolours.
fn annotation_stack_camera(stack: AnnotationStack, point: vec2<f32>) -> AnnotationStack {
  return AnnotationStack(annotation_camera_over(stack.rgba, point),
                         annotation_camera_over(stack.bare, point), stack.highlighted);
}

/// What one placed loupe shows at `probe` of everything in `[0, last)`.
fn annotation_loupe_at(shape: PreviewGeometry, probe: vec2<f32>, last: u32, feather: f32,
                       number_atlas: AnnotationTextAtlas, cursor: bool) -> vec4<f32> {
  if (annotation_magnify_inside(shape, probe, feather) <= 0.0) {
    return vec4<f32>(0.0);
  }
  let shrink = annotation_magnify_shrink(shape);
  return annotation_loupe_content(annotation_magnify_seen_at(shape, probe),
                                  shrink * feather * 2.0, 0u, last,
                                  feather * min(shrink.x, shrink.y), number_atlas, cursor);
}

/// Lays the magnifier at `index` over `stack`, its loupe showing everything
/// below it. Under the camera, the loupe shows what the camera covers, the
/// annotations under it too; over it, the camera as the canvas draws it. One
/// that moved while the shutter was open is drawn whole at every exposure
/// sample, as `annotation_magnify_layer` draws it.
fn annotation_stack_magnifier(stack: AnnotationStack, index: u32, point: vec2<f32>,
                              feather: f32, number_atlas: AnnotationTextAtlas, cursor: bool)
    -> AnnotationStack {
  let magnifier = annotation_arrows[index];
  let color = annotation_color(magnifier);
  let halo = max(magnifier.hover, 0.0);
  let taps = max(magnifier.sample_count, 1u);
  var rgba = vec4<f32>(0.0);
  var bare = vec4<f32>(0.0);
  for (var tap = 0u; tap < taps; tap++) {
    var shape = magnifier.geometry;
    var tint = color;
    if (magnifier.sample_count > 0u) {
      let sample = annotation_samples[magnifier.sample_first + tap];
      shape = sample.geometry;
      tint = vec4<f32>(color.rgb, color.a * sample.opacity);
    }
    let content = annotation_loupe_at(shape, point, index, feather, number_atlas, cursor);
    let sides = select(1u, 2u, stack.highlighted);
    for (var side = 0u; side < sides; side++) {
      let drawn = annotation_magnify_draw(select(stack.rgba, stack.bare, side == 1u), shape,
                                          tint, magnifier.flags, point, feather, halo, content);
      if (side == 0u) {
        rgba += drawn;
      } else {
        bare += drawn;
      }
    }
    if (!stack.highlighted) {
      bare = rgba;
    }
  }
  return AnnotationStack(rgba / f32(taps), bare / f32(taps), stack.highlighted);
}

/// Draws the prepared annotations in `[first, last)` over `rgba_in`, in
/// document order, and last the cursor where `cursor` says this layer carries
/// it: over every mark, but lying on the picture, so the spotlights' shade
/// darkens it, their blur softens it, and a loupe hides it. `base` is the
/// canvas at `canvas_point` before any annotation. The pass that draws the
/// mark blur's layer stops under the shade the layer is for, and answers the
/// pixel as drawn there.
///
/// The range is how the camera ordering is expressed: Rust sorts the
/// screen's annotations ahead of the camera's, keeping each one's order, so
/// each pass draws one contiguous run rather than testing a flag per
/// annotation per pixel. `[0, all)` is every run: the spotlights' one shade
/// lies at the topmost of them in any run, so it darkens everything drawn
/// before it, the camera too when a camera spotlight holds it, and nothing
/// after; every spotlight cuts its hole in it, whichever run it is in.
fn composite_annotation_layers(rgba_in: vec4<f32>, base: vec4<f32>, canvas_point: vec2<f32>,
                               first: u32, last: u32, all: u32, feather: f32,
                               number_atlas: AnnotationTextAtlas, cursor: bool) -> vec4<f32> {
  let shade_at = annotation_shade_at(canvas_point, 0u, all);
  var stack = AnnotationStack(rgba_in, rgba_in, false);
  for (var index = annotation_next(canvas_point, first, last); index < last;
       index = annotation_next(canvas_point, index + 1u, last)) {
    if (annotation_kind_drawn(annotation_spotlight_kind) && index == shade_at) {
      if (annotation_blur_mode() == 1u && index == annotation_blur_spotlight()) {
        return stack.rgba;
      }
      stack = annotation_stack_shaded(stack, base, canvas_point, 0u, index, all, feather);
    }
    let annotation = annotation_arrows[index];
    if (annotation_kind_drawn(annotation_magnify_kind) &&
        annotation.kind == annotation_magnify_kind) {
      stack = annotation_stack_magnifier(stack, index, canvas_point, feather, number_atlas,
                                         cursor);
    } else {
      stack = annotation_stack_draw(stack, annotation, canvas_point, feather, number_atlas,
                                    cursor);
    }
  }
  if (!cursor) {
    return stack.rgba;
  }
  let shaded_here = shade_at >= first && shade_at < last;
  return annotation_stack_cursor(stack.rgba, canvas_point, first, last,
                                 select(0u, all, shaded_here), feather,
                                 max(2.0 * feather, 1e-4));
}
