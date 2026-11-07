// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::background::background_constants;
use super::draw_rows::{
  camera_rows, crop_preview_shadow_sigma, cursor_rows, screen_shadow_sigma, source_flags,
  LayerCamera,
};
use super::layer::crop_preview_rect;
use super::scene_motion::scene_motion_rows;
use super::scene_turn::scene_turn_rows;
use super::*;

impl Compositor {
  #[allow(clippy::too_many_arguments)]
  pub fn draw_with_camera(
    &self,
    target: &wgpu::TextureView,
    source: &SourceTexture,
    settings: &ScreenshotOutputSettings,
    composition: ComposedFrame,
    camera: LayerCamera<'_>,
    magnifier: Option<CropMagnifier>,
    // Prepared annotations in canvas pixels, those under the camera first, with
    // their exposure samples and the drawn pixel scale their edges feather
    // over. The shader draws `[0, below_camera)` before the camera layer and
    // the rest after it, so the ordering is a range rather than a flag.
    annotations: &PreparedArrows,
  ) -> Result<(), String> {
    self
      .draw_layer(
        target,
        source,
        settings,
        &CanvasGeometry::of(source.size, settings)?,
        composition,
        camera,
        magnifier,
        annotations,
        LayerDraw {
          placement: None,
          clear: !composition.foreground_only,
          redaction_slot: 0,
          retained_source: source.picture.is_some(),
        },
      )
      .map(drop)
  }

  /// [`Self::draw_with_camera`] for a canvas whose `geometry` may run ahead
  /// of its settings, drawn where `layer` says. Answers the picture the
  /// canvas sampled, after its redactions, which stays readable until the
  /// same redaction slot is drawn again.
  #[allow(clippy::too_many_arguments)]
  pub fn draw_layer(
    &self,
    target: &wgpu::TextureView,
    source: &SourceTexture,
    settings: &ScreenshotOutputSettings,
    geometry: &CanvasGeometry,
    composition: ComposedFrame,
    camera: LayerCamera<'_>,
    magnifier: Option<CropMagnifier>,
    annotations: &PreparedArrows,
    layer: LayerDraw,
  ) -> Result<wgpu::TextureView, String> {
    let placement = geometry.placement;
    let (width, height) = (geometry.size.0 as f32, geometry.size.1 as f32);
    if !settings.radius_percent.is_finite()
      || !(0.0..=50.0).contains(&settings.radius_percent)
      || !settings.background_radius_percent.is_finite()
      || !(0.0..=50.0).contains(&settings.background_radius_percent)
    {
      return Err("The screenshot canvas settings are not valid".to_owned());
    }
    let background = background_constants(settings)?;
    let (visible_left, visible_top, visible_right, visible_bottom) =
      foreground_bounds_f32(placement, settings.recenter_inset_color.is_some());
    let shadow_sigma = screen_shadow_sigma(
      [visible_left, visible_top, visible_right, visible_bottom],
      (width, height),
      source.cut_out,
    );
    // While the crop tool is open the ghost is the whole uncropped source, so
    // the rounding and the shadow move to the cropped layer drawn over it.
    let crop_preview = crop_preview_rect(settings).map(|rect| rect.map(|value| value as f32));
    let shadow = settings.drop_shadow
      && crop_preview.is_none()
      && visible_right > visible_left
      && visible_bottom > visible_top
      && shadow_sigma > 1.0;
    let preview_shadow_sigma = crop_preview
      .filter(|_| settings.drop_shadow)
      .map_or(0.0, |rect| crop_preview_shadow_sigma(rect, (width, height)));
    // A chosen picture is filled to the canvas by the shader, from a texture
    // uploaded once per file. One that cannot be read leaves the flag clear
    // and the solid colour paints, so a background whose file has moved still
    // previews and still exports.
    let picture = (settings.background_type == "image")
      .then_some(settings.background_image_path.as_deref())
      .flatten()
      .and_then(|path| self.background_cache.resolve(&self.gpu, path));
    let (cursor_artwork, cursor_frame, cursor_model) =
      self.cursor_artwork_constants(composition.cursor)?;
    let [scene_motion, scene_image_motion, camera_motion_frame, camera_motion_crop] =
      scene_motion_rows(settings, (width, height));
    let [cursor_geometry, cursor_effects, cursor_blur] =
      cursor_rows(composition.cursor, annotations.spotlight_blur);
    let [camera_frame, camera_crop, camera_effects] = camera_rows(camera, settings);
    let values = Constants {
      output_source: [width, height, source.size.0 as f32, source.size.1 as f32],
      image_rect: [
        placement.image_x as f32,
        placement.image_y as f32,
        placement.image_width as f32,
        placement.image_height as f32,
      ],
      crop_rect: [
        placement.crop_x as f32,
        placement.crop_y as f32,
        placement.crop_width as f32,
        placement.crop_height as f32,
      ],
      source_crop_rect: [
        placement.source_crop_x as f32,
        placement.source_crop_y as f32,
        placement.source_crop_width as f32,
        placement.source_crop_height as f32,
      ],
      crop_preview_rect: crop_preview.unwrap_or([0.0; 4]),
      // The radius is a percentage of the layer's shorter side, and in crop
      // mode that layer is the crop rectangle rather than the whole source.
      crop_preview_effects: crop_preview.map_or([0.0; 4], |[_, _, rect_width, rect_height]| {
        [
          rect_width.min(rect_height) * (settings.radius_percent as f32 / 100.0),
          1.0,
          preview_shadow_sigma,
          0.0,
        ]
      }),
      solid_color: colour_f32(&settings.background_color)?,
      base_color: background.base,
      recenter_inset_color: optional_colour_f32(settings.recenter_inset_color.as_deref())?,
      mesh_points: background.points,
      mesh_colors: background.colors,
      effects: [
        geometry.radius,
        geometry.background_radius,
        settings.mesh_warp_percent as f32,
        shadow_sigma,
      ],
      // The third word is how many canvas pixels one drawn pixel covers,
      // which the arrow edges feather over. `submit` fills the fourth once
      // the atlas has chosen how finely to rasterise the annotations' type.
      motion: [
        composition.seconds as f32,
        background.speed,
        if annotations.pixel_scale > 0.0 {
          annotations.pixel_scale
        } else {
          1.0
        },
        0.0,
      ],
      cursor_geometry,
      cursor_effects,
      cursor_blur,
      camera_frame,
      camera_crop,
      camera_effects,
      magnifier: magnifier.map_or([0.0; 4], |value| value.geometry),
      magnifier_options: magnifier.map_or([0.0; 4], |value| value.options),
      magnifier_bounds: magnifier.map_or([0.0, 0.0, 1.0, 1.0], |value| value.bounds),
      options: [
        settings.mesh_seed,
        u32::from(background.mesh),
        settings.mesh_points.len() as u32,
        u32::from(shadow),
      ],
      cursor_options: [
        composition.cursor.map_or(0, |cursor| cursor.style),
        u32::from(composition.cursor.is_some()),
        u32::from(
          composition
            .cursor
            .is_some_and(|cursor| cursor.clip_at_video_edge),
        ),
        u32::from(composition.foreground_only),
      ],
      background_options: [
        u32::from(picture.is_some()),
        background.generator,
        background.palette_size,
        0,
      ],
      // `submit` fills the last two: the number atlas's size is only known
      // once it has rasterised the counters' numbers.
      annotation_options: [
        annotations.below_camera,
        annotations.arrows.len() as u32,
        0,
        0,
      ],
      cursor_artwork,
      cursor_frame,
      cursor_model,
      // Laid out in `submit`, once the counters' numbers are known.
      annotation_tiles: [0; 4],
      placement: LayerPlacement::row(layer.placement, geometry.size),
      // Planned in `submit`, which decides whether the layer is drawn.
      annotation_blur: [0.0; 4],
      scene_motion,
      scene_image_motion,
      camera_motion_frame,
      camera_motion_crop,
      // How opaque a scene draws the screen and the camera, then how strong
      // each casts its shadow while the preview fades one in or out.
      scene_opacity: {
        let [screen, camera] = settings.scene_opacity.unwrap_or([1.0, 1.0]);
        [
          screen,
          camera,
          settings.shadow_strength(),
          settings.camera_shadow_strength.unwrap_or(1.0),
        ]
      },
      source_flags: source_flags(source, camera),
      scene_turn: scene_turn_rows(settings, &placement, geometry.size),
      // Set in `submit` for the passes a focused frame takes.
      focus: [0.0; 4],
      overflow: [
        if layer.placement.is_some() {
          LayerPlacement::OVERFLOW_OPACITY
        } else {
          0.0
        },
        0.0,
        0.0,
        0.0,
      ],
      // The source's own round corners, in canvas pixels: a window recorded
      // on its own, black outside them in its movie.
      source_shape: [
        (settings.capture_corner_radius * placement.image_width / f64::from(source.size.0.max(1)))
          as f32,
        0.0,
        0.0,
        0.0,
      ],
    };
    self.submit(
      target,
      source,
      geometry.size,
      composition,
      camera,
      picture,
      values,
      annotations,
      layer,
    )
  }
}
