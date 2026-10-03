// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The editor canvas: background, the placed source with its crop, rounding
// and shadow, the camera, the annotations, the keyboard overlay, the cursor
// and the crop magnifier, in one pass over a full-canvas triangle. Assembled
// with the background generators and the annotation pieces by
// `build/wgsl.rs`; this part comes first because it carries the module's
// directive.
//
// Several layers sample their textures inside branches on the pixel's own
// geometry. Neighbouring pixels of a quad take the same branch everywhere a
// texture is read, so the implicit derivatives are well defined, but WGSL's
// uniformity analysis cannot prove it.
diagnostic(off, derivative_uniformity);

// The twin of `Constants` in `compositor.rs`; every member is a `vec4` row.
struct Canvas {
  output_source: vec4<f32>, // output width/height, source width/height
  image_rect: vec4<f32>,
  crop_rect: vec4<f32>,
  source_crop_rect: vec4<f32>,
  // The crop tool's result layer: output-pixel rectangle, then its radius,
  // an enabled flag, the shadow sigma and one spare word.
  crop_preview_rect: vec4<f32>,
  crop_preview_effects: vec4<f32>,
  solid_color: vec4<f32>,
  base_color: vec4<f32>,
  recenter_inset_color: vec4<f32>,
  mesh_points: array<vec4<f32>, 8>,
  mesh_colors: array<vec4<f32>, 4>,
  effects: vec4<f32>, // image radius, background radius, warp, shadow sigma
  // Timeline seconds, generator speed, canvas pixels per drawn pixel, and
  // atlas pixels per canvas pixel for the annotations' type.
  motion: vec4<f32>,
  cursor_geometry: vec4<f32>, // source-space anchor x/y, artwork width/height
  cursor_effects: vec4<f32>, // opacity, reserved y, rotation radians, scale
  // Source-space frame delta x/y of the cursor, then the spotlights' blur the
  // cursor and the marks under it take: its deviation in output pixels and
  // how far it has arrived.
  cursor_blur: vec4<f32>,
  camera_frame: vec4<f32>, // output-space x/y/width/height
  camera_crop: vec4<f32>, // camera source-space x/y/width/height
  camera_effects: vec4<f32>, // radius, enabled, shadow sigma, camera on top
  magnifier: vec4<f32>,
  magnifier_options: vec4<f32>,
  magnifier_bounds: vec4<f32>,
  native_cursor_hotspots: array<vec4<f32>, 8>, // normalized atlas hotspot x/y
  options: vec4<u32>, // seed, mesh enabled, point count, shadow enabled
  cursor_options: vec4<u32>, // artwork, enabled, clip to video, foreground only
  background_options: vec4<u32>, // has background image, generator, palette size
  // Annotations below the camera are sorted ahead of those above it, so `x` is
  // both the below-camera count and where the above-camera run starts.
  annotation_options: vec4<u32>, // below-camera count, total count, number atlas size
  // The artwork a cursor handed its own drawings is drawn with (macOS):
  // the style's bitmap and design size, the hotspot in the recorded cursor
  // box and the design's origin, and whether the artwork model is in use,
  // whether the design is used, whether the box clips, and supersampling.
  // Here `cursor_geometry.xy` is the hotspot in output pixels.
  cursor_artwork: vec4<f32>,
  cursor_frame: vec4<f32>,
  cursor_model: vec4<u32>,
  // The annotation tiles: side in canvas pixels, grid width and height, and
  // words a tile's set takes. See `annotation_tiles.wgsl`.
  annotation_tiles: vec4<u32>,
  // Where the canvas is drawn in its target: the corner, and the canvas pixels
  // one target pixel covers. The identity draws it at its own size.
  placement: vec4<f32>,
  // The spotlights' blur of the annotations under their shade, which
  // `compositor/mark_blur.rs` lays into `annotation_blur_layer`: its texels per
  // canvas pixel, whether this pass draws that layer (1) or reads it (2), and
  // the index of the spotlight whose shade it lies under.
  annotation_blur: vec4<f32>,
  // How a scene moved the screen's box while the shutter was open: the scale
  // and the shift, in canvas pixels, that carry the box as drawn onto where
  // it was, then how many steps the frame averages. One step draws it sharp.
  scene_motion: vec4<f32>,
  // The same scale and shift for the screen's image, which a zoom carries
  // further than its box.
  scene_image_motion: vec4<f32>,
  // The camera's frame when the shutter opened, in canvas pixels, and the part
  // of its picture that frame showed, in shares of the picture.
  camera_motion_frame: vec4<f32>,
  camera_motion_crop: vec4<f32>,
  // How opaque the screen and the camera are drawn, which a scene fades as it
  // hides or shows them.
  scene_opacity: vec4<f32>,
}

// How many placements a frame averages while a scene moves its panes; one
// draws them sharp.
fn scene_samples() -> u32 {
  return max(u32(canvas.scene_motion.w), 1u);
}

// False in the lean pipelines a draw showing no annotation uses, whose module
// is this source with that one line changed: the counts read as zero, so the
// compiler leaves every annotation pass out rather than keeping its code in a
// shader that walks empty lists. The cursor and the keyboard overlay are
// drawn either way.
const annotations_drawn: bool = true;

// The twin of `KeyboardConstants`.
struct Keyboard {
  dimensions: vec4<u32>, // artwork width/height, key count, animation
  animation: vec4<f32>, // scale, layout progress, maximum width, requested scale
  position: vec4<f32>, // normalized centre x/y; negative keeps the default
  key_geometry: array<vec4<u32>, 8>, // artwork x, artwork width, visible, slot
  key_motion: array<vec4<f32>, 8>, // alpha, scale, progress, layout progress
  key_masks: array<vec4<u32>, 8>, // layout from mask, layout to mask
  // Group centre x/y; -1 inherits, <=-1.5 default; z group scale ratio.
  key_position: array<vec4<f32>, 8>,
}

@group(0) @binding(0) var<uniform> canvas: Canvas;
@group(0) @binding(1) var<uniform> keyboard: Keyboard;
@group(0) @binding(2) var source_image: texture_2d<f32>;
@group(0) @binding(3) var native_cursor_images: texture_2d_array<f32>;
@group(0) @binding(4) var camera_image: texture_2d<f32>;
@group(0) @binding(5) var keyboard_image: texture_2d<f32>;
@group(0) @binding(6) var background_image: texture_2d<f32>;
@group(0) @binding(12) var linear_sampler: sampler;
@group(0) @binding(13) var point_sampler: sampler;
// What the annotations under the spotlights' shade add to the canvas, and the
// cursor alone, each blurred over the whole canvas; see `annotation_blur`.
@group(0) @binding(15) var annotation_blur_layer: texture_2d<f32>;
@group(0) @binding(16) var annotation_cursor_layer: texture_2d<f32>;

// Where a magnifier reads the picture it enlarges: the source, after its
// redactions, placed at `image_rect` and showing where the canvas crop and the
// source's own crop, both in output pixels, overlap.
fn annotation_magnify_placement() -> AnnotationMagnifyPlacement {
  var at: AnnotationMagnifyPlacement;
  at.image = canvas.image_rect;
  at.size = canvas.output_source.zw;
  let per = at.size / max(canvas.image_rect.zw, vec2<f32>(1.0));
  let low = max(canvas.crop_rect.xy, canvas.source_crop_rect.xy);
  let high = min(canvas.crop_rect.xy + canvas.crop_rect.zw,
                 canvas.source_crop_rect.xy + canvas.source_crop_rect.zw);
  let last = max(at.size - 1.0, vec2<f32>(0.0));
  let first_texel = clamp(floor((low - canvas.image_rect.xy) * per + 1e-3), vec2<f32>(0.0), last);
  let last_texel = clamp(ceil((high - canvas.image_rect.xy) * per - 1e-3) - 1.0, first_texel,
                         last);
  at.texels = vec4<f32>(first_texel, last_texel);
  return at;
}

fn annotation_magnify_fetch(texel: vec2<i32>) -> vec4<f32> {
  return textureLoad(source_image, texel, 0);
}

fn hash(position: vec2<f32>, seed: u32) -> f32 {
  let value = sin(dot(position, vec2<f32>(127.1, 311.7)) + f32(seed) * 0.017) * 43758.5453;
  return fract(value) * 2.0 - 1.0;
}

fn noise(position: vec2<f32>, seed: u32) -> f32 {
  let cell = floor(position);
  let local = fract(position);
  let eased = local * local * (3.0 - 2.0 * local);
  let top = mix(hash(cell, seed), hash(cell + vec2<f32>(1.0, 0.0), seed), eased.x);
  let bottom = mix(hash(cell + vec2<f32>(0.0, 1.0), seed), hash(cell + 1.0, seed), eased.x);
  return mix(top, bottom, eased.y);
}

fn fractal_noise(position: vec2<f32>, seed: u32) -> f32 {
  return noise(position, seed) * 0.58
    + noise(position * 2.07 + vec2<f32>(11.3, -4.9), seed ^ 0x68bc21ebu) * 0.28
    + noise(position * 4.19 + vec2<f32>(-8.7, 13.1), seed ^ 0x02e5be93u) * 0.14;
}

fn rounded_distance(pixel: vec2<f32>, rect: vec4<f32>, radius: f32) -> f32 {
  let half_size = rect.zw * 0.5;
  let local = abs(pixel - (rect.xy + half_size)) - (half_size - radius);
  return length(max(local, vec2<f32>(0.0))) + min(max(local.x, local.y), 0.0) - radius;
}

fn rounded_coverage(pixel: vec2<f32>, rect: vec4<f32>, radius: f32) -> f32 {
  let distance = rounded_distance(pixel, rect, radius);
  if (radius <= 0.0) {
    return select(0.0, 1.0, distance < 0.0);
  }
  return 1.0 - smoothstep(-0.75, 0.75, distance);
}

fn drop_shadow(distance: f32, sigma: f32) -> f32 {
  return (36.0 / 255.0) * exp(-0.5 * distance * distance / (sigma * sigma));
}

// The screen's drop shadow, its box read at `box_pixel` and its image at
// `image_pixel`: one point while still, two while a zoom carries them apart.
fn visible_shadow(box_pixel: vec2<f32>, image_pixel: vec2<f32>, sigma: f32) -> f32 {
  let offset = vec2<f32>(0.0, sigma * 0.35);
  // The foreground is the intersection of the crop window and placed source.
  // A tall crop around a wide source must not cast a tall rectangular shadow.
  let crop_distance = rounded_distance(box_pixel - offset, canvas.crop_rect, canvas.effects.x);
  let image_distance = rounded_distance(image_pixel - offset, canvas.image_rect, 0.0);
  let distance = max(select(max(crop_distance, image_distance), crop_distance,
                            canvas.recenter_inset_color.a > 0.0), 0.0);
  return drop_shadow(distance, sigma);
}

/// The crop tool's result layer, drawn over the uncropped ghost.
///
/// Crop mode shows the whole source so what is being cropped away stays
/// visible, which leaves the ghost underneath flat: no rounding, no shadow.
/// The layer the crop actually produces is drawn a second time here, at its
/// place in the same canvas, carrying the radius and the shadow the ghost
/// gives up. It samples the same source through the same image mapping, so
/// the crop rectangle is a rounded window onto pixels already in place.
fn crop_preview_layer(result_in: vec4<f32>, pixel: vec2<f32>) -> vec4<f32> {
  if (canvas.crop_preview_effects.y == 0.0) {
    return result_in;
  }
  var result = result_in;
  let radius = canvas.crop_preview_effects.x;
  let coverage = rounded_coverage(pixel, canvas.crop_preview_rect, radius);
  let sigma = canvas.crop_preview_effects.z;
  if (sigma > 1.0) {
    let shadow_pixel = pixel - vec2<f32>(0.0, sigma * 0.35);
    let distance = max(rounded_distance(shadow_pixel, canvas.crop_preview_rect, radius), 0.0);
    // The shadow belongs to what lies outside the layer, so the layer itself
    // is never tinted by it.
    let shadow = drop_shadow(distance, sigma);
    result = vec4<f32>(result.rgb * (1.0 - shadow * (1.0 - coverage)), result.a);
  }
  if (coverage > 0.0) {
    let uv = (pixel - canvas.image_rect.xy) / canvas.image_rect.zw;
    let video = textureSample(source_image, linear_sampler, uv);
    result = mix(result, vec4<f32>(video.rgb, 1.0), video.a * coverage);
  }
  return result;
}

fn cursor_sample(source_pixel: vec2<f32>, anchor: vec2<f32>) -> vec4<f32> {
  // The preview's screen-space rotation convention is opposite to the
  // raster convention used to calculate the motion lean.
  var angle = -canvas.cursor_effects.z;
  if (canvas.cursor_options.x == 2u) {
    angle += 1.57079632679;
  }
  let delta = source_pixel - anchor;
  let cosine = cos(angle);
  let sine = sin(angle);
  let local = vec2<f32>(cosine * delta.x + sine * delta.y, -sine * delta.x + cosine * delta.y) /
      max(canvas.cursor_effects.w, 0.01);
  let atlas_uv = local / canvas.cursor_geometry.zw +
      canvas.native_cursor_hotspots[canvas.cursor_options.x].xy;
  if (any(atlas_uv < vec2<f32>(0.0)) || any(atlas_uv >= vec2<f32>(1.0))) {
    return vec4<f32>(0.0);
  }
  // The atlas has one level, and the layers sample it inside loops: a loupe's
  // exposure and the cursor's blur.
  return textureSampleLevel(native_cursor_images, linear_sampler, atlas_uv,
                            canvas.cursor_options.x, 0.0);
}

// The cursor drawn from the artwork it was handed, as the macOS canvas draws
// it: a port of `CursorRaster::sample` and `draw_blurred`
// (editor/cursor_effects/raster.rs) in output pixels.

// One bilinear read of the style's bitmap at `point`, in its own pixels,
// weighted by alpha so transparent texels lend no colour.
fn artwork_texel(point_in: vec2<f32>) -> vec4<f32> {
  let bitmap = max(vec2<u32>(canvas.cursor_artwork.xy), vec2<u32>(1u));
  let last = vec2<f32>(bitmap - vec2<u32>(1u));
  let point = clamp(point_in, vec2<f32>(0.0), last);
  let low = vec2<u32>(floor(point));
  let high = min(low + vec2<u32>(1u), vec2<u32>(last));
  let fraction = point - vec2<f32>(low);
  let layer = i32(canvas.cursor_options.x);
  let samples = array<vec4<f32>, 4>(
    textureLoad(native_cursor_images, vec2<i32>(low), layer, 0),
    textureLoad(native_cursor_images, vec2<i32>(i32(high.x), i32(low.y)), layer, 0),
    textureLoad(native_cursor_images, vec2<i32>(i32(low.x), i32(high.y)), layer, 0),
    textureLoad(native_cursor_images, vec2<i32>(high), layer, 0),
  );
  let weights = array<f32, 4>(
    (1.0 - fraction.x) * (1.0 - fraction.y),
    fraction.x * (1.0 - fraction.y),
    (1.0 - fraction.x) * fraction.y,
    fraction.x * fraction.y,
  );
  var alpha = 0.0;
  var colour = vec3<f32>(0.0);
  for (var index = 0u; index < 4u; index++) {
    alpha += samples[index].a * weights[index];
    colour += samples[index].rgb * samples[index].a * weights[index];
  }
  if (alpha <= 0.0) {
    return vec4<f32>(0.0);
  }
  return vec4<f32>(colour / alpha, alpha);
}

// The output point turned and scaled into the recorded cursor box, then
// mapped onto the artwork; drawn fallback artwork keeps its design aspect.
fn artwork_sample(point: vec2<f32>, anchor: vec2<f32>) -> vec4<f32> {
  let delta = point - anchor;
  let cosine = cos(canvas.cursor_effects.z);
  let sine = sin(canvas.cursor_effects.z);
  let local = vec2<f32>(cosine * delta.x + sine * delta.y, -sine * delta.x + cosine * delta.y) /
      max(canvas.cursor_effects.w, 0.0001) + canvas.cursor_frame.xy;
  let box = canvas.cursor_geometry.zw;
  if (canvas.cursor_model.z != 0u && (any(local < vec2<f32>(0.0)) || any(local >= box))) {
    return vec4<f32>(0.0);
  }
  let bitmap = canvas.cursor_artwork.xy;
  if (canvas.cursor_model.y == 0u) {
    return artwork_texel(local / max(box, vec2<f32>(0.0001)) * bitmap);
  }
  let design_size = canvas.cursor_artwork.zw;
  let artwork_scale = max(min(box.x / design_size.x, box.y / design_size.y), 0.01);
  let design = local / artwork_scale + canvas.cursor_frame.zw;
  if (any(design < vec2<f32>(0.0)) || any(design >= design_size)) {
    return vec4<f32>(0.0);
  }
  return artwork_texel(design / design_size * bitmap);
}

// System artwork already carries an antialiased edge, so only the hard-edged
// drawn fallback is supersampled over the pixel's 4x4 box.
fn artwork_draw_sample(point: vec2<f32>, anchor: vec2<f32>) -> vec4<f32> {
  if (canvas.cursor_model.w == 0u) {
    return artwork_sample(point, anchor);
  }
  let offsets = array<f32, 4>(-0.375, -0.125, 0.125, 0.375);
  var alpha = 0.0;
  var colour = vec3<f32>(0.0);
  for (var y = 0u; y < 4u; y++) {
    for (var x = 0u; x < 4u; x++) {
      let sample = artwork_sample(point + vec2<f32>(offsets[x], offsets[y]), anchor);
      alpha += sample.a;
      colour += sample.rgb * sample.a;
    }
  }
  if (alpha <= 0.0) {
    return vec4<f32>(0.0);
  }
  return vec4<f32>(colour / alpha, alpha / 16.0);
}

// Whether `point` lies inside a box of `size` rounded by `radius`, judged by
// the pixel's own centre rather than by coverage.
fn rounded_pixel_visible(point: vec2<f32>, size: vec2<f32>, radius: f32) -> bool {
  if (radius <= 0.0) {
    return true;
  }
  let edge = min(point, size - point);
  let corner = max(vec2<f32>(0.0), vec2<f32>(radius) - edge);
  return length(corner) <= radius;
}

// The cursor at an output pixel, straight alpha before its opacity: exposure
// taps are Gaussian weighted along the frame's travel, no more than two
// output pixels apart, and a cursor clipped to the video is cut pixel by
// pixel at the crop's rounded edge.
fn artwork_cursor(point: vec2<f32>) -> vec4<f32> {
  if (canvas.cursor_artwork.x <= 0.0 || canvas.cursor_artwork.y <= 0.0) {
    return vec4<f32>(0.0);
  }
  let anchor = canvas.cursor_geometry.xy;
  let delta = canvas.cursor_blur.xy;
  let travel = length(delta);
  let distance = min(travel, 80.0);
  let radius = length(canvas.cursor_geometry.zw) * canvas.cursor_effects.w + distance + 4.0;
  if (any(abs(point - anchor) > vec2<f32>(radius))) {
    return vec4<f32>(0.0);
  }
  if (canvas.cursor_options.z != 0u) {
    let crop_point = point - canvas.crop_rect.xy;
    let crop_size = canvas.crop_rect.zw;
    if (any(crop_point < vec2<f32>(0.0)) || any(crop_point >= crop_size) ||
        !rounded_pixel_visible(crop_point, crop_size, canvas.effects.x)) {
      return vec4<f32>(0.0);
    }
  }
  if (!(distance > 1.25 && travel > 0.0)) {
    return artwork_draw_sample(point, anchor);
  }
  let direction = delta / travel;
  let count = u32(clamp(ceil(distance / 2.0) + 1.0, 8.0, 48.0));
  var total_weight = 0.0;
  var alpha = 0.0;
  var colour = vec3<f32>(0.0);
  for (var index = 0u; index < count; index++) {
    let progress = f32(index) / f32(count - 1u);
    let centered = (progress - 0.5) / 0.34;
    let weight = exp(-0.5 * centered * centered);
    let sample = artwork_draw_sample(point, anchor + direction * ((progress - 0.8) * distance));
    alpha += sample.a * weight;
    colour += sample.rgb * sample.a * weight;
    total_weight += weight;
  }
  alpha /= total_weight;
  if (alpha <= 0.0) {
    return vec4<f32>(0.0);
  }
  return vec4<f32>(colour / (total_weight * alpha), alpha);
}

fn cursor_layer(pixel: vec2<f32>) -> vec4<f32> {
  if (canvas.cursor_options.y == 0u) {
    return vec4<f32>(0.0);
  }
  if (canvas.cursor_model.x != 0u) {
    return artwork_cursor(pixel);
  }
  let image_rect = canvas.image_rect;
  let source_size = canvas.output_source.zw;
  let source_pixel = (pixel - image_rect.xy) / image_rect.zw * source_size;
  // The recorded hotspot is source-space, but the atlas is sampled in the
  // pane's physical output. Snap that transformed hotspot to a pixel edge so
  // pixel-centred samples hit the cursor artwork consistently instead of
  // bilinearly splitting a thin I-beam or crosshair across two pixels.
  let display_anchor = image_rect.xy + canvas.cursor_geometry.xy / source_size * image_rect.zw;
  let anchor = (round(display_anchor) - image_rect.xy) / image_rect.zw * source_size;
  let travel = min(length(canvas.cursor_blur.xy), 80.0);
  let radius = length(canvas.cursor_geometry.zw) * canvas.cursor_effects.w + travel + 4.0;
  if (any(abs(source_pixel - anchor) > vec2<f32>(radius))) {
    return vec4<f32>(0.0);
  }
  if (travel <= 1.25) {
    return cursor_sample(source_pixel, anchor);
  }
  let direction = canvas.cursor_blur.xy / max(length(canvas.cursor_blur.xy), 0.001);
  var accumulated = vec4<f32>(0.0);
  var total_weight = 0.0;
  for (var index = 0u; index < 24u; index++) {
    let progress = f32(index) / 23.0;
    let centered = (progress - 0.5) / 0.34;
    let weight = exp(-0.5 * centered * centered);
    let sample_anchor = anchor + direction * ((progress - 0.8) * travel);
    let sample = cursor_sample(source_pixel, sample_anchor);
    accumulated += vec4<f32>(sample.rgb * sample.a * weight, sample.a * weight);
    total_weight += weight;
  }
  let alpha = accumulated.a / total_weight;
  if (alpha <= 0.0) {
    return vec4<f32>(0.0, 0.0, 0.0, alpha);
  }
  return vec4<f32>(accumulated.rgb / (total_weight * alpha), alpha);
}

// The cursor for the annotation layers, straight alpha: faded by its opacity
// and, where it is clipped to the video, held inside the shown picture.
fn annotation_cursor_sample(probe: vec2<f32>) -> vec4<f32> {
  var cursor = cursor_layer(probe);
  cursor.a *= canvas.cursor_effects.x;
  // The artwork model clips inside `artwork_cursor`, pixel by pixel.
  if (canvas.cursor_options.z != 0u && canvas.cursor_model.x == 0u) {
    cursor.a *= rounded_coverage(probe, canvas.crop_rect, canvas.effects.x) *
        rounded_coverage(probe, canvas.image_rect, 0.0) *
        rounded_coverage(probe, canvas.source_crop_rect, 0.0);
  }
  return cursor;
}

// Whether the cursor may draw within `margin` canvas pixels of `probe`, by
// the same reach `cursor_layer` turns pixels away at, so a probe farther off
// samples nothing however far a blur's taps spread.
fn annotation_cursor_near(probe: vec2<f32>, margin: f32) -> bool {
  if (canvas.cursor_options.y == 0u) {
    return false;
  }
  let travel = min(length(canvas.cursor_blur.xy), 80.0);
  let radius = length(canvas.cursor_geometry.zw) * canvas.cursor_effects.w + travel + 4.0;
  if (canvas.cursor_model.x != 0u) {
    return all(abs(probe - canvas.cursor_geometry.xy) <= vec2<f32>(radius + margin));
  }
  let image_rect = canvas.image_rect;
  let source_size = canvas.output_source.zw;
  let source_pixel = (probe - image_rect.xy) / image_rect.zw * source_size;
  let display_anchor = image_rect.xy + canvas.cursor_geometry.xy / source_size * image_rect.zw;
  let anchor = (round(display_anchor) - image_rect.xy) / image_rect.zw * source_size;
  return all(abs(source_pixel - anchor) <= radius + margin * source_size / image_rect.zw);
}

fn annotation_spotlight_blur() -> AnnotationSpotlightBlur {
  return AnnotationSpotlightBlur(canvas.cursor_blur.z,
                                 select(0.0, canvas.cursor_blur.w, annotations_drawn),
                                 select(0u, canvas.annotation_options.y, annotations_drawn));
}

// The first annotation drawn over the camera, which a loupe showing what
// lies under it lays the camera under; past every annotation where the camera
// is not drawn over them.
fn annotation_camera_run() -> u32 {
  if (!annotations_drawn || canvas.camera_effects.y == 0.0 || canvas.camera_effects.w == 0.0) {
    return 0xffffffffu;
  }
  return canvas.annotation_options.x;
}

// The camera laid over `rgba` at canvas point `point`, where the canvas
// draws it, without the averaging a moving layout adds: a loupe shows it
// enlarged, standing still.
fn annotation_camera_over(rgba: vec4<f32>, point: vec2<f32>) -> vec4<f32> {
  let picture = vec2<f32>(textureDimensions(camera_image));
  let crop = vec4<f32>(canvas.camera_crop.xy / picture, canvas.camera_crop.zw / picture);
  return mix(rgba, camera_layer_at(rgba, point, canvas.camera_frame, crop),
             canvas.scene_opacity.y);
}

// Whether this pass draws the annotations' blur layer (1), draws the cursor's
// (3), reads them (2), or none of these.
fn annotation_blur_mode() -> u32 {
  return u32(canvas.annotation_blur.y);
}

// Whether the annotations under a shade have a blur layer at all.
fn annotation_blur_marks() -> bool {
  return canvas.annotation_blur.z >= 0.0;
}

// The spotlight whose shade the annotations' blur layer lies under.
fn annotation_blur_spotlight() -> u32 {
  return u32(max(canvas.annotation_blur.z, 0.0));
}

// The point in a blur layer, which covers the canvas, that `point` falls on.
fn annotation_blur_at(point: vec2<f32>, layer: texture_2d<f32>) -> vec2<f32> {
  return point * canvas.annotation_blur.x / vec2<f32>(textureDimensions(layer));
}

// What the annotations under that shade add to the canvas at `point`,
// blurred.
fn annotation_marks_blurred(point: vec2<f32>) -> vec4<f32> {
  return textureSampleLevel(annotation_blur_layer, linear_sampler,
                            annotation_blur_at(point, annotation_blur_layer), 0.0);
}

// Whether this pass reads the cursor's blur layer.
fn annotation_cursor_blur_layer() -> bool {
  return annotation_blur_mode() == 2u && canvas.annotation_blur.w != 0.0;
}

// The cursor alone at `point`, blurred, premultiplied.
fn annotation_cursor_blurred(point: vec2<f32>) -> vec4<f32> {
  return textureSampleLevel(annotation_cursor_layer, linear_sampler,
                            annotation_blur_at(point, annotation_cursor_layer), 0.0);
}

// A chosen picture covers the canvas: it is scaled until both sides reach,
// centred, and the overflowing axis is trimmed evenly, so it never letterboxes
// and never stretches. The same framing as `cover_fit` in the CPU compose
// path, done here in UVs so one upload serves every canvas size.
fn background_picture(pixel: vec2<f32>) -> vec3<f32> {
  let picture = vec2<f32>(textureDimensions(background_image));
  if (picture.x == 0.0 || picture.y == 0.0) {
    return canvas.solid_color.rgb;
  }
  let output = canvas.output_source.xy;
  let scale = max(output.x / picture.x, output.y / picture.y);
  let covered = picture * scale;
  let origin = (output - covered) * 0.5;
  return textureSample(background_image, linear_sampler, (pixel - origin) / covered).rgb;
}

fn background(pixel: vec2<f32>) -> vec3<f32> {
  if (canvas.background_options.x != 0u) {
    return background_picture(pixel);
  }
  if (canvas.options.y == 0u) {
    return canvas.solid_color.rgb;
  }
  // A ported generator reads its palette from the front of the mesh colours,
  // the seed, and the same timeline seconds the classic mesh drifts with,
  // which `gen_pixel_shifted` scales by the generator's own speed in `motion.y`.
  if (canvas.background_options.y != 0u) {
    let palette = GenPalette(canvas.mesh_colors[0].rgb, canvas.mesh_colors[1].rgb,
                             canvas.mesh_colors[2].rgb, canvas.mesh_colors[3].rgb,
                             canvas.background_options.z);
    // Ported generators reuse the classic mesh point slot for the seed's
    // domain shift, resolved on the CPU unless its last word asks for the
    // shader's own hash; only the classic generator reads point geometry.
    var shift = canvas.mesh_points[0].xyz;
    if (canvas.mesh_points[0].w != 0.0) {
      shift = gen_seed_shift(canvas.options.x);
    }
    return gen_pixel_shifted(canvas.background_options.y, pixel, canvas.output_source.xy, palette,
                             shift, canvas.motion.x, canvas.motion.y);
  }
  let shortest = min(canvas.output_source.x, canvas.output_source.y);
  let dimensions = canvas.output_source.xy;
  let aspect = dimensions / shortest;
  let frequency = 3.5 / shortest;
  let phase = canvas.motion.x * 0.28;
  let drift = vec2<f32>(sin(phase), cos(phase * 0.83)) * shortest * 0.012;
  let warped_pixel = pixel + drift;
  let warp_scale = shortest * canvas.effects.z / 100.0;
  let seed = canvas.options.x;
  let warp = vec2<f32>(
      fractal_noise(warped_pixel * frequency + phase * 0.035, seed),
      fractal_noise(warped_pixel * frequency + vec2<f32>(19.7, -7.3) - phase * 0.03,
                    seed ^ 0xa511e9b3u)) * warp_scale;
  var weighted = canvas.base_color.rgb * 0.18;
  var total = 0.18;
  for (var index = 0u; index < canvas.options.z; index++) {
    let first = canvas.mesh_points[index * 2u];
    let second = canvas.mesh_points[index * 2u + 1u];
    let local_phase = phase + f32(index) * 1.73;
    let animated_center = first.xy + vec2<f32>(sin(local_phase), cos(local_phase * 0.91)) * 0.012;
    let delta = (pixel + warp) / shortest - animated_center * aspect;
    let rotated = vec2<f32>(delta.x * second.x + delta.y * second.y,
                            -delta.x * second.y + delta.y * second.x);
    let distance = length(rotated / max(first.zw, vec2<f32>(0.01)));
    let weight = 1.0 / (pow(max(distance, 0.025), 3.5) + 0.012);
    weighted += canvas.mesh_colors[index].rgb * weight;
    total += weight;
  }
  let result = weighted / total;
  let depth = fractal_noise((pixel + drift) * frequency * 0.7, seed ^ 0xd1b54a35u) * 13.0 / 255.0;
  return saturate(result + depth);
}

@vertex
fn vs_main(@builtin(vertex_index) id: u32) -> @builtin(position) vec4<f32> {
  let position = vec2<f32>(f32((id << 1u) & 2u), f32(id & 2u));
  return vec4<f32>(position * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 0.0, 1.0);
}

// The camera drawn in `frame`, in canvas pixels, showing `crop` of its
// picture, in shares of it.
fn camera_layer_at(result_in: vec4<f32>, pixel: vec2<f32>, frame: vec4<f32>,
                   crop: vec4<f32>) -> vec4<f32> {
  var result = result_in;
  let camera_alpha = rounded_coverage(pixel, frame, canvas.camera_effects.x);
  let sigma = canvas.camera_effects.z;
  if (sigma > 1.0) {
    let shadow_pixel = pixel - vec2<f32>(0.0, sigma * 0.35);
    let distance = max(rounded_distance(shadow_pixel, frame, canvas.camera_effects.x), 0.0);
    let shadow = drop_shadow(distance, sigma);
    result = vec4<f32>(result.rgb * (1.0 - shadow * (1.0 - camera_alpha)), result.a);
  }
  let camera_local = (pixel - frame.xy) / frame.zw;
  if (camera_alpha > 0.0 && all(camera_local >= vec2<f32>(0.0)) &&
      all(camera_local <= vec2<f32>(1.0))) {
    let camera_uv = crop.xy + camera_local * crop.zw;
    // Sampled inside the motion's loop as well, so from the one level there is.
    let camera = textureSampleLevel(camera_image, linear_sampler, camera_uv, 0.0);
    let alpha = camera.a * camera_alpha;
    result = vec4<f32>(mix(result.rgb, camera.rgb, alpha), alpha + result.a * (1.0 - alpha));
  }
  return result;
}

// The camera, averaged over every step a layout moved it through while the
// shutter was open. A layout changes its frame's shape, so each step has a
// frame and a crop of its own rather than one transform of the drawn camera.
// The camera's drawing is laid over `result_in` as opaque as the scene has it.
fn camera_layer(result_in: vec4<f32>, pixel: vec2<f32>) -> vec4<f32> {
  let opacity = canvas.scene_opacity.y;
  if (canvas.camera_effects.y == 0.0 || opacity <= 0.0) {
    return result_in;
  }
  let picture = vec2<f32>(textureDimensions(camera_image));
  let drawn_crop = vec4<f32>(canvas.camera_crop.xy / picture, canvas.camera_crop.zw / picture);
  let samples = scene_samples();
  if (samples == 1u) {
    return mix(result_in, camera_layer_at(result_in, pixel, canvas.camera_frame, drawn_crop),
               opacity);
  }
  var sum = vec4<f32>(0.0);
  for (var index = 0u; index < samples; index++) {
    let step = f32(index) / f32(samples - 1u);
    sum += camera_layer_at(result_in, pixel, mix(canvas.camera_motion_frame, canvas.camera_frame, step),
                           mix(canvas.camera_motion_crop, drawn_crop, step));
  }
  return mix(result_in, sum / f32(samples), opacity);
}

// The screen layer over `result_in`: its shadow, its recentre inset and its
// picture, its box read at `box_pixel` and its image at `image_pixel`.
fn screen_layer(result_in: vec4<f32>, box_pixel: vec2<f32>, image_pixel: vec2<f32>,
                foreground_only: bool) -> vec4<f32> {
  var result = result_in;
  let crop_alpha = rounded_coverage(box_pixel, canvas.crop_rect, canvas.effects.x);
  // Axis-aligned zero-radius source edges are already pixel-exact. Smoothing
  // them leaks a fractional row of canvas colour around a default crop.
  let image_rect_alpha = rounded_coverage(image_pixel, canvas.image_rect, 0.0);
  let image_alpha = crop_alpha * image_rect_alpha *
      rounded_coverage(image_pixel, canvas.source_crop_rect, 0.0);
  let inset = canvas.recenter_inset_color;
  let frame_alpha = select(image_alpha, crop_alpha, inset.a > 0.0);
  if (canvas.options.w != 0u && canvas.effects.w > 1.0) {
    let shadow = visible_shadow(box_pixel, image_pixel, canvas.effects.w);
    if (foreground_only) {
      result.a = shadow * (1.0 - frame_alpha);
    } else {
      result = vec4<f32>(result.rgb * (1.0 - shadow * (1.0 - frame_alpha)), result.a);
    }
  }
  if (inset.a > 0.0) {
    result = mix(result, vec4<f32>(inset.rgb, 1.0), crop_alpha);
  }
  let uv = (image_pixel - canvas.image_rect.xy) / canvas.image_rect.zw;
  if (image_alpha > 0.0) {
    // Sampled inside the motion's loop as well, so from the one level there is.
    let video = textureSampleLevel(source_image, linear_sampler, uv, 0.0);
    result = mix(result, vec4<f32>(video.rgb, 1.0), video.a * image_alpha);
  }
  return result;
}

// The screen, averaged over every step a scene moved it through while the
// shutter was open. Its box and its image each only move and scale, so each
// step reads the drawn screen at the points that step carried here; the
// background under it stays sharp.
fn moving_screen_layer(result_in: vec4<f32>, pixel: vec2<f32>, foreground_only: bool)
    -> vec4<f32> {
  let samples = scene_samples();
  if (samples == 1u) {
    return screen_layer(result_in, pixel, pixel, foreground_only);
  }
  var sum = vec4<f32>(0.0);
  for (var index = 0u; index < samples; index++) {
    let step = f32(index) / f32(samples - 1u);
    let box_scale = mix(canvas.scene_motion.x, 1.0, step);
    let box_shift = mix(canvas.scene_motion.yz, vec2<f32>(0.0), step);
    let image_scale = mix(canvas.scene_image_motion.x, 1.0, step);
    let image_shift = mix(canvas.scene_image_motion.yz, vec2<f32>(0.0), step);
    sum += screen_layer(result_in, (pixel - box_shift) / box_scale,
                        (pixel - image_shift) / image_scale, foreground_only);
  }
  return sum / f32(samples);
}

fn keyboard_key_count() -> u32 {
  return min(keyboard.dimensions.z, 8u);
}

fn keyboard_effective_scale(canvas_dimensions: vec2<f32>) -> f32 {
  let animation = keyboard.animation;
  let requested = select(animation.x, animation.w, animation.w > 0.0);
  if (!(animation.z > 0.0) || any(canvas_dimensions <= vec2<f32>(0.0))) {
    return requested;
  }
  let design_height = 20.0;
  let edge_margin = 0.055;
  let animation_extent = 1.12;
  let available_width = canvas_dimensions.x * (1.0 - edge_margin * 2.0);
  let width_at_unit_scale = canvas_dimensions.y * (60.0 / 1080.0) * animation.z / design_height;
  let fitted = available_width / max(width_at_unit_scale * animation_extent, 0.0001);
  return min(requested, fitted);
}

fn keyboard_texel(source_in: vec2<f32>) -> vec4<f32> {
  let last = vec2<f32>(keyboard.dimensions.xy) - 1.0;
  let source = clamp(source_in, vec2<f32>(0.0), last);
  let low = vec2<i32>(floor(source));
  let high = min(low + 1, vec2<i32>(last));
  let fraction = fract(source);
  let a = textureLoad(keyboard_image, vec2<i32>(low.x, low.y), 0);
  let b = textureLoad(keyboard_image, vec2<i32>(high.x, low.y), 0);
  let c = textureLoad(keyboard_image, vec2<i32>(low.x, high.y), 0);
  let d = textureLoad(keyboard_image, vec2<i32>(high.x, high.y), 0);
  return mix(mix(a, b, fraction.x), mix(c, d, fraction.x), fraction.y);
}

// A key's group centre on one axis: non-negative is explicit, -1 follows the
// overlay centre, and at or below -1.5 the key keeps the built-in default.
fn keyboard_key_axis(key_value: f32, overlay_value: f32) -> f32 {
  if (key_value >= 0.0) {
    return key_value;
  }
  return select(-1.0, overlay_value, key_value > -1.5);
}

fn keyboard_key_ratio(index: u32) -> f32 {
  let ratio = keyboard.key_position[index].z;
  return select(1.0, ratio, ratio > 0.0);
}

fn keyboard_key_pixel(index: u32, canvas_point: vec2<f32>, canvas_dimensions: vec2<f32>,
                      animation_scale: f32, x_offset: f32) -> vec4<f32> {
  if (animation_scale <= 0.0001) {
    return vec4<f32>(0.0);
  }
  let geometry = keyboard.key_geometry[index];
  let artwork = vec2<f32>(keyboard.dimensions.xy);
  let height = canvas_dimensions.y * (60.0 / 1080.0) *
      keyboard_effective_scale(canvas_dimensions) * keyboard_key_ratio(index);
  let width = height * artwork.x / max(artwork.y, 1.0);
  let bottom = canvas_dimensions.y * 0.055;
  let position_x = keyboard_key_axis(keyboard.key_position[index].x, keyboard.position.x);
  let position_y = keyboard_key_axis(keyboard.key_position[index].y, keyboard.position.y);
  let center_x = select(canvas_dimensions.x * 0.5, position_x * canvas_dimensions.x,
                        position_x >= 0.0);
  let center_y = select(canvas_dimensions.y - bottom - height * 0.5,
                        position_y * canvas_dimensions.y, position_y >= 0.0);
  let row_x = center_x - width * 0.5;
  let key_x = row_x + width * f32(geometry.x) / artwork.x;
  let key_width = width * f32(geometry.y) / artwork.x;
  let key_size = vec2<f32>(key_width, height) * animation_scale;
  let center = vec2<f32>(key_x + key_width * 0.5 + x_offset, center_y);
  let uv = (canvas_point - (center - key_size * 0.5)) / key_size;
  if (any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0))) {
    return vec4<f32>(0.0);
  }
  // `keyboard_texel` treats integer coordinates as texel centres. Convert
  // from rectangle-edge UVs accordingly and clamp to this key so linear
  // sampling cannot pull colour from its neighbouring gap or key.
  var source = vec2<f32>(f32(geometry.x) + uv.x * f32(geometry.y) - 0.5,
                         uv.y * artwork.y - 0.5);
  source.x = clamp(source.x, f32(geometry.x), f32(geometry.x + max(geometry.y, 1u) - 1u));
  source.y = clamp(source.y, 0.0, f32(keyboard.dimensions.y - 1u));
  return keyboard_texel(source);
}

fn keyboard_motion_spring(progress: f32) -> f32 {
  let t = saturate(progress);
  let phase = 6.0 * t;
  if (t >= 1.0) {
    return 1.0;
  }
  return 1.0 - exp(-5.0 * t) * (cos(phase) + (5.0 / 6.0) * sin(phase));
}

fn keyboard_slot_count() -> u32 {
  let count = keyboard_key_count();
  var slots = 0u;
  for (var index = 0u; index < count; index++) {
    slots = max(slots, keyboard.key_geometry[index].w + 1u);
  }
  return slots;
}

fn keyboard_gap() -> f32 {
  let count = keyboard_key_count();
  var gap = 1e30;
  for (var index = 1u; index < count; index++) {
    let previous = keyboard.key_geometry[index - 1u];
    let candidate = f32(keyboard.key_geometry[index].x) - f32(previous.x + previous.y);
    if (candidate > 0.0) {
      gap = min(gap, candidate);
    }
  }
  return select(0.0, gap, gap < 1e30);
}

fn keyboard_slot_width(slot: u32) -> f32 {
  let count = keyboard_key_count();
  var width = 0.0;
  for (var index = 0u; index < count; index++) {
    if (keyboard.key_geometry[index].w == slot) {
      width = max(width, f32(keyboard.key_geometry[index].y));
    }
  }
  return width;
}

fn keyboard_slot_left(slot: u32, mask: u32) -> f32 {
  let slots = keyboard_slot_count();
  let gap = keyboard_gap();
  let included = countOneBits(mask);
  var total = gap * f32(max(i32(included) - 1, 0));
  for (var candidate = 0u; candidate < slots; candidate++) {
    if ((mask & (1u << candidate)) != 0u) {
      total += keyboard_slot_width(candidate);
    }
  }
  var left = (f32(keyboard.dimensions.x) - total) * 0.5;
  for (var walked = 0u; walked < slot; walked++) {
    if ((mask & (1u << walked)) != 0u) {
      left += keyboard_slot_width(walked) + gap;
    }
  }
  return left;
}

fn keyboard_layout_offset(index: u32, canvas_dimensions: vec2<f32>, progress_delta: f32) -> f32 {
  let count = keyboard_key_count();
  if (count < 2u) {
    return 0.0;
  }
  let artwork = vec2<f32>(keyboard.dimensions.xy);
  let height = canvas_dimensions.y * (60.0 / 1080.0) * keyboard_effective_scale(canvas_dimensions);
  let full_width = height * artwork.x / max(artwork.y, 1.0);
  let geometry = keyboard.key_geometry[index];
  let masks = keyboard.key_masks[index];
  let slot_width = keyboard_slot_width(geometry.w);
  let from_center = keyboard_slot_left(geometry.w, masks.x) + slot_width * 0.5;
  let to_center = keyboard_slot_left(geometry.w, masks.y) + slot_width * 0.5;
  let progress = keyboard_motion_spring(max(keyboard.key_motion[index].w - progress_delta, 0.0));
  let target_center = mix(from_center, to_center, progress);
  let source_offset = target_center - (f32(geometry.x) + f32(geometry.y) * 0.5);
  return source_offset * full_width * keyboard_key_ratio(index) / max(artwork.x, 1.0);
}

// Whether key `index` drawn at up to `scale` may reach `canvas_point`: within
// half its height of its row's centre, and within three row widths of the
// row's centre across, which holds the key wherever its layout carries it,
// spring and all. `keyboard_key_pixel` is transparent everywhere else, so a
// pixel turned away here composites exactly as one walked through it.
fn keyboard_key_may_reach(index: u32, canvas_point: vec2<f32>, canvas_dimensions: vec2<f32>,
                          scale: f32) -> bool {
  let artwork = vec2<f32>(keyboard.dimensions.xy);
  let height = canvas_dimensions.y * (60.0 / 1080.0) *
      keyboard_effective_scale(canvas_dimensions) * keyboard_key_ratio(index);
  let width = height * artwork.x / max(artwork.y, 1.0);
  let position_x = keyboard_key_axis(keyboard.key_position[index].x, keyboard.position.x);
  let position_y = keyboard_key_axis(keyboard.key_position[index].y, keyboard.position.y);
  let center_x = select(canvas_dimensions.x * 0.5, position_x * canvas_dimensions.x,
                        position_x >= 0.0);
  let center_y = select(canvas_dimensions.y - canvas_dimensions.y * 0.055 - height * 0.5,
                        position_y * canvas_dimensions.y, position_y >= 0.0);
  let reach = vec2<f32>(width * (3.0 + scale), height * scale * 0.5) + 1.0;
  return all(abs(canvas_point - vec2<f32>(center_x, center_y)) <= reach);
}

fn composite_keyboard(rgba_in: vec4<f32>, canvas_point: vec2<f32>,
                      dimensions: vec2<f32>) -> vec4<f32> {
  if (keyboard.dimensions.z == 0u || keyboard.dimensions.x == 0u ||
      keyboard.dimensions.y == 0u) {
    return rgba_in;
  }
  var rgba = rgba_in;
  let count = keyboard_key_count();
  for (var index = 0u; index < count; index++) {
    let geometry = keyboard.key_geometry[index];
    let motion = keyboard.key_motion[index];
    if (geometry.z == 0u || motion.x <= 0.0) {
      continue;
    }
    let pop_blur = keyboard.dimensions.w == 0u && motion.z < 1.0;
    let requested_scale = select(keyboard.animation.x, keyboard.animation.w,
                                 keyboard.animation.w > 0.0);
    let current_scale = motion.y / max(requested_scale, 0.001);
    let previous_progress = select(max(motion.z - 0.08, 0.0), min(motion.z + 0.08, 1.0),
                                   geometry.z == 2u);
    let previous_scale = select(current_scale, keyboard_motion_spring(previous_progress),
                                pop_blur);
    // Every sample is drawn at a scale between these two.
    if (!keyboard_key_may_reach(index, canvas_point, dimensions,
                                max(current_scale, previous_scale))) {
      continue;
    }
    var value = vec4<f32>(0.0);
    var total = 0.0;
    let layout_offset = keyboard_layout_offset(index, dimensions, 0.0);
    let previous_offset = keyboard_layout_offset(index, dimensions, 0.12);
    let layout_delta = previous_offset - layout_offset;
    let layout_blur = abs(layout_delta) > 0.25;
    let key_height = dimensions.y * (60.0 / 1080.0) * keyboard_effective_scale(dimensions);
    let key_width = key_height * f32(geometry.y) / max(f32(keyboard.dimensions.y), 1.0);
    let radial_travel = 0.5 * length(vec2<f32>(key_width, key_height)) *
        abs(previous_scale - current_scale);
    let travel = max(abs(layout_delta), radial_travel);
    var samples = 1u;
    if (pop_blur || layout_blur) {
      samples = min(max(u32(ceil(travel / 0.75)) + 1u, 8u), 48u);
    }
    for (var step_index = 0u; step_index < samples; step_index++) {
      let amount = select(f32(step_index) / f32(samples - 1u), 0.0, samples == 1u);
      let weight = exp(-2.5 * amount * amount);
      value += keyboard_key_pixel(index, canvas_point, dimensions,
                                  mix(current_scale, previous_scale, amount),
                                  layout_offset + layout_delta * amount) * weight;
      total += weight;
    }
    value /= max(total, 1.0);
    let opacity = saturate(motion.x);
    rgba = vec4<f32>(value.rgb * opacity + rgba.rgb * (1.0 - value.a * opacity),
                     value.a * opacity + rgba.a * (1.0 - value.a * opacity));
  }
  return rgba;
}

// The crop magnifier: a square lens over the canvas showing the source or
// the camera pixel for pixel.
fn crop_magnifier(result: vec4<f32>, pixel: vec2<f32>) -> vec4<f32> {
  let magnifier = canvas.magnifier;
  if (magnifier.z <= 0.0) {
    return result;
  }
  let center = magnifier.xy;
  let box_size = magnifier.zz;
  let box_origin = center - box_size * 0.5;
  let corner_radius = max(magnifier.z / 24.0, 1.0);
  let distance = rounded_distance(pixel, vec4<f32>(box_origin, box_size), corner_radius);
  if (distance > 0.0) {
    return result;
  }
  let options = canvas.magnifier_options;
  var lens = vec4<f32>(0.15, 0.15, 0.16, 1.0);
  if (options.x != 0.0) {
    let frame = canvas.camera_frame;
    let local = (center - frame.xy) / frame.zw;
    let source_pixel = canvas.camera_crop.xy + local * canvas.camera_crop.zw +
        (pixel - center) / max(magnifier.w, 1.0);
    let dimensions = vec2<f32>(textureDimensions(camera_image));
    if (all(source_pixel >= vec2<f32>(0.0)) && all(source_pixel < dimensions)) {
      lens = textureSample(camera_image, point_sampler, source_pixel / dimensions);
    }
  } else {
    let local = (center - canvas.image_rect.xy) / canvas.image_rect.zw;
    let dimensions = canvas.output_source.zw;
    let source_pixel = local * dimensions + (pixel - center) / max(magnifier.w, 1.0);
    let source_uv = source_pixel / dimensions;
    let bounds = canvas.magnifier_bounds;
    let in_effective_source = all(source_uv >= bounds.xy) && all(source_uv <= bounds.xy + bounds.zw);
    if (all(source_pixel >= vec2<f32>(0.0)) && all(source_pixel < dimensions) &&
        in_effective_source) {
      lens = textureSample(source_image, point_sampler, source_uv);
    }
  }
  let edges = u32(options.y);
  let shade = ((edges & 1u) != 0u && pixel.x < center.x) ||
      ((edges & 2u) != 0u && pixel.x >= center.x) ||
      ((edges & 4u) != 0u && pixel.y < center.y) ||
      ((edges & 8u) != 0u && pixel.y >= center.y);
  var rgb = lens.rgb;
  if (shade) {
    rgb = mix(rgb, select(vec3<f32>(1.0), vec3<f32>(0.0), options.z != 0.0), 0.1);
  }
  rgb = mix(rgb, vec3<f32>(0.15, 0.15, 0.16), smoothstep(-1.5, -0.5, distance));
  return vec4<f32>(rgb, 1.0);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
  let pixel = (position.xy - canvas.placement.xy) * canvas.placement.zw;
  if (annotation_blur_mode() == 3u) {
    // Drawing the cursor's blur layer: the cursor alone, premultiplied.
    if (!annotation_cursor_near(pixel, 1.0)) {
      return vec4<f32>(0.0);
    }
    let pointer = annotation_cursor_sample(pixel);
    return vec4<f32>(pointer.rgb * pointer.a, pointer.a);
  }
  let foreground_only = canvas.cursor_options.w != 0u;
  let background_alpha = rounded_coverage(pixel, vec4<f32>(0.0, 0.0, canvas.output_source.xy),
                                          canvas.effects.y);
  var result = vec4<f32>(0.0);
  if (!foreground_only) {
    result = vec4<f32>(background(pixel), 1.0);
  }
  if (canvas.camera_effects.w == 0.0) {
    result = camera_layer(result, pixel);
  }
  // A scene fades the screen as it hides or shows it, its annotations and
  // the cursor it carries with it.
  let screen_opacity = canvas.scene_opacity.x;
  if (screen_opacity > 0.0) {
    result = mix(result, moving_screen_layer(result, pixel, foreground_only), screen_opacity);
  }
  result = crop_preview_layer(result, pixel);
  // Every annotation edge feathers over one *drawn* pixel, not one canvas
  // pixel: a canvas shown smaller than its resolution would otherwise take its
  // whole antialiasing band from inside a single drawn pixel and come out
  // jagged. `motion.z` is how many canvas pixels one drawn pixel covers.
  let annotation_feather = max(canvas.motion.z, 1e-4) * 0.5;
  // The atlas's size and, in `motion.w`, how many atlas pixels it holds per
  // canvas pixel.
  let annotation_atlas = AnnotationTextAtlas(canvas.annotation_options.zw, canvas.motion.w);
  // The screen layer carries the cursor, over every mark but shaded and
  // hidden by what acts on the picture. The run below the camera is drawn
  // first, then the camera, then the run above it. Both runs go through the
  // one call: the composite is large, and the GPU compiler inlines every call
  // site whole, which multiplies how long the pipeline takes to build.
  let below = select(0u, canvas.annotation_options.x, annotations_drawn);
  let total = select(0u, canvas.annotation_options.y, annotations_drawn);
  let runs = select(1u, 2u, total > below);
  // The canvas before any annotation, which the mark blur's layer is the
  // difference from.
  let base = result;
  for (var run = 0u; run < runs; run++) {
    let first = select(below, 0u, run == 0u);
    let last = select(total, below, run == 0u);
    let unannotated = result;
    result = composite_annotation_layers(result, base, pixel, first, last, total,
                                         annotation_feather, annotation_atlas, run == 0u);
    // The run over the camera is the camera's own annotations, which a scene
    // fades with the camera rather than with the screen.
    result = mix(unannotated, result, select(screen_opacity, canvas.scene_opacity.y, run == 1u));
    let shade = annotation_blur_spotlight();
    if (annotation_blur_mode() == 1u && shade >= first && shade < last) {
      // Drawing the layer: the run stopped under the shade.
      return result - base;
    }
    if (run == 0u && canvas.camera_effects.w != 0.0) {
      result = camera_layer(result, pixel);
    }
  }
  result = composite_keyboard(result, pixel, canvas.output_source.xy);
  if (!foreground_only) {
    // Hashed at the drawn pixel's corner, as every macOS canvas has dithered.
    result = vec4<f32>(saturate(result.rgb + hash(floor(position.xy), 0x9e3779b9u) / 255.0), result.a);
  }
  result = crop_magnifier(result, pixel);
  // DirectComposition consumes premultiplied alpha. Clip every composed layer
  // to the rounded canvas and premultiply only once at the final boundary.
  return result * background_alpha;
}
