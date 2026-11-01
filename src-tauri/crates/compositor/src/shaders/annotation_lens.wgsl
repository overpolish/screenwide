// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The editor's lenses: what each shows of everything below it, and the lens
// laid over the canvas through it.
//
// What a lens shows is the canvas composed under it, the annotations below
// it included, which costs as much as drawing the canvas there. A lens
// settling onto the picture is drawn whole at every exposure sample, up to
// forty-eight a frame, and its glass looks once per colour where it parts
// them, so composing it where each look lands held a 1080p preview's frame
// to tens of milliseconds. Instead a pass ahead of the canvas composes it
// once a texel into `lens_grounds` (`compositor/lens_ground.rs`), one texel
// for each pixel the lens draws, where the record says, and every look
// reads it from there. The texels lie on the points the lens shows at its
// drawn pixels, so a still lens reads each one whole and shows what the
// composition would have; a sample nearer or further reads between them.
//
// WGSL has no recursion, so that pass draws the lenses inside a lens's
// ground over the composed canvas alone (`annotation_magnify_layer`).

/// Where a lens's ground lies: the point its first texel shows and the
/// step from one texel to the next, both where the lens's annotations lie,
/// the texel it starts at in `lens_grounds` and how many it covers, whether
/// the lens lies in the camera's run, and whether its ground shows the
/// cursor. The record of a lens keeps these in its `turn`, which only a
/// turning caption piece otherwise reads.
struct AnnotationLensGround {
  origin: vec2<f32>,
  step: vec2<f32>,
  atlas: vec2<f32>,
  size: vec2<f32>,
  camera_run: bool,
  cursor: bool,
}

fn annotation_lens_ground_of(lens: PreviewArrow) -> AnnotationLensGround {
  let turn = lens.geometry.turn;
  return AnnotationLensGround(vec2<f32>(turn[0], turn[1]), vec2<f32>(turn[2], turn[3]),
                              vec2<f32>(turn[4], turn[5]), vec2<f32>(turn[6], turn[7]),
                              turn[8] != 0.0, turn[9] != 0.0);
}

/// What a lens shows at point `point` under it, premultiplied: the canvas as
/// composed under every annotation, and `[first, last)`, everything below
/// the lens, shaded and blurred as the canvas draws them, and the cursor
/// where `cursor` says the layer carries it. A lens over the camera shows
/// the camera too, laid between the annotations under it and those over it
/// as the canvas lays it. One lens pixel spans `span` canvas pixels there,
/// and `feather` is half of the smaller.
fn annotation_lens_content(point: vec2<f32>, span: vec2<f32>, first: u32, last: u32,
                           feather: f32, number_atlas: AnnotationTextAtlas, cursor: bool)
    -> vec4<f32> {
  let base = annotation_lens_ground(point, span);
  let shade_at = annotation_shade_at(point, first, last);
  let camera_at = annotation_camera_run();
  var stack = AnnotationStack(base, base, false);
  // The camera lies under the annotation at `camera_at`, so under the lens
  // itself when that is the lens.
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

/// While the grounds are drawn: what the lens whose ground holds the texel
/// at `texel` of `lens_grounds` shows there, of everything below it, or
/// nothing past every ground. `feather` is half a drawn pixel of the canvas,
/// which one texel shows a zoom's share of.
fn annotation_lens_grounds_at(texel: vec2<f32>, feather: f32, number_atlas: AnnotationTextAtlas)
    -> vec4<f32> {
  let total = canvas.annotation_options.y;
  for (var index = 0u; index < total; index++) {
    let lens = annotation_arrows[index];
    if (lens.kind != annotation_magnify_kind) {
      continue;
    }
    let ground = annotation_lens_ground_of(lens);
    if (any(texel < ground.atlas) || any(texel >= ground.atlas + ground.size)) {
      continue;
    }
    let point = ground.origin + (texel - ground.atlas - 0.5) * ground.step;
    let shrink = 1.0 / max(lens.geometry.high, 1e-3);
    probing_turned = !ground.camera_run && screen_turned();
    let content = annotation_lens_content(point, vec2<f32>(shrink * feather * 2.0), 0u, index,
                                          feather * shrink, number_atlas, ground.cursor);
    probing_turned = false;
    return content;
  }
  return vec4<f32>(0.0);
}

/// What `lens`, placed as `shape`, shows at `probe`, read from its ground.
fn annotation_lens_at(lens: PreviewArrow, shape: PreviewGeometry, probe: vec2<f32>,
                      feather: f32) -> vec4<f32> {
  if (annotation_magnify_inside(shape, probe, feather) <= 0.0) {
    return vec4<f32>(0.0);
  }
  let ground = annotation_lens_ground_of(lens);
  if (any(ground.size <= vec2<f32>(0.0))) {
    return vec4<f32>(0.0);
  }
  let at = (annotation_magnify_seen_at(shape, probe) - ground.origin) / ground.step + 0.5;
  let texel = ground.atlas + clamp(at, vec2<f32>(0.5), ground.size - 0.5);
  return textureSampleLevel(lens_grounds, linear_sampler,
                            texel / vec2<f32>(textureDimensions(lens_grounds)), 0.0);
}

/// Lays the lens at `index` over `stack`, showing everything below it,
/// looked at once where its glass keeps the colours together and once per
/// colour where it parts them. Under the camera, the lens shows what the
/// camera covers, the annotations under it too; over it, the camera as the
/// canvas draws it. One that changed while the shutter was open is drawn
/// whole at every exposure sample, as `annotation_magnify_layer` draws it.
fn annotation_stack_magnifier(stack: AnnotationStack, index: u32, point: vec2<f32>,
                              feather: f32) -> AnnotationStack {
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
    let glass = annotation_magnify_glass(shape, point, feather);
    let looks = annotation_glass_looks(glass);
    var content = vec4<f32>(0.0);
    for (var look = 0u; look < looks; look++) {
      let seen = annotation_lens_at(magnifier, shape, annotation_glass_look(glass, look, looks),
                                    feather);
      content = annotation_glass_take(content, seen, look, looks);
    }
    let sides = select(1u, 2u, stack.highlighted);
    for (var side = 0u; side < sides; side++) {
      let drawn = annotation_magnify_draw(select(stack.rgba, stack.bare, side == 1u), shape,
                                          tint, magnifier.flags, point, feather, halo, content,
                                          glass);
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
