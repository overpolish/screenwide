// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Compositor {
  #[allow(clippy::too_many_arguments)]
  pub(in crate::editor::preview_platform::surface) fn draw_with_camera(
    &self,
    context: &ID3D11DeviceContext,
    target: &ID3D11Texture2D,
    source: &SourceTexture,
    settings: &ScreenshotOutputSettings,
    composition: super::super::ComposedFrame,
    camera: Option<(&SourceTexture, BakeGeometry, bool, bool)>,
    magnifier: Option<super::super::recenter::CropMagnifier>,
    // Prepared marks in canvas pixels, those under the camera first, with
    // their exposure samples and the drawn pixel scale their edges feather
    // over. The shader draws `[0, below_camera)` before the camera layer and
    // the rest after it, so the ordering is a range rather than a flag.
    annotations: &PreparedArrows,
  ) -> Result<(), String> {
    let placement = output_placement(source.size.0, source.size.1, settings)?;
    let mut mesh_points = [[0.0; 4]; 8];
    let mut mesh_colors = [[0.0; 4]; 4];
    let mesh = settings.background_type == "mesh";
    // A chosen picture is a background of its own: the shader fills it to the
    // canvas from `background_image`, falling back to the solid colour when
    // the file behind it cannot be read.
    if !mesh && !matches!(settings.background_type.as_str(), "image" | "solid") {
      return Err("The screenshot background type is not valid".to_owned());
    }
    if !settings.radius_percent.is_finite()
      || !(0.0..=50.0).contains(&settings.radius_percent)
      || !settings.background_radius_percent.is_finite()
      || !(0.0..=50.0).contains(&settings.background_radius_percent)
    {
      return Err("The screenshot canvas settings are not valid".to_owned());
    }
    // Zero is the app's own blob mesh, which alone reads the points and the
    // warp; every other id names a generator that reads only its palette and
    // the seed.
    let mut generator_id = 0;
    let mut generator_color_count = 0;
    // The classic mesh reads the seconds as they come, so a canvas with no
    // mesh at all carries its speed rather than a zero.
    let mut generator_speed = 1.0;
    if mesh {
      validate_mesh(
        &settings.mesh_generator,
        &settings.mesh_colors,
        &settings.mesh_points,
        settings.mesh_warp_percent,
      )?;
      let generator = mesh_generator(&settings.mesh_generator)
        .ok_or("The screenshot mesh background is not valid")?;
      generator_id = generator.id;
      generator_color_count = generator.color_count as u32;
      generator_speed = generator.speed;
      if generator.id == 0 {
        for (index, point) in settings.mesh_points.iter().enumerate() {
          let radians = point.rotation.to_radians();
          mesh_points[index * 2] = [
            (point.x / 100.0) as f32,
            (point.y / 100.0) as f32,
            (point.radius_x / 100.0) as f32,
            (point.radius_y / 100.0) as f32,
          ];
          mesh_points[index * 2 + 1] = [radians.cos() as f32, radians.sin() as f32, 0.0, 0.0];
          mesh_colors[index] = colour_f32(&settings.mesh_colors[index])?;
        }
      } else {
        mesh_colors = generator_palette(&settings.mesh_colors)?;
        // Ported generators reuse the classic mesh point slot for their
        // CPU-resolved seed domain shift; they do not read classic points.
        let shift = crate::screenshots::generator_seed_shift(settings.mesh_seed);
        mesh_points[0] = [shift[0], shift[1], shift[2], 0.0];
      }
    }
    let shortest_output = settings.width.min(settings.height) as f32;
    let (visible_left, visible_top, visible_right, visible_bottom) =
      foreground_bounds_f32(placement, settings.recenter_inset_color.is_some());
    let visible_width = (visible_right - visible_left).max(0.0);
    let visible_height = (visible_bottom - visible_top).max(0.0);
    let shadow_margin = visible_left
      .min(visible_top)
      .min(settings.width as f32 - visible_right)
      .min(settings.height as f32 - visible_bottom)
      .max(0.0);
    let shadow_sigma = (visible_width.min(visible_height) * 0.055)
      .clamp(10.0, 110.0)
      .min(shadow_margin * 0.45);
    let shadow =
      settings.drop_shadow && visible_width > 0.0 && visible_height > 0.0 && shadow_sigma > 1.0;
    let shortest_crop = placement.crop_width.min(placement.crop_height) as f32;
    let device = unsafe { target.GetDevice() }.map_err(|error| error.to_string())?;
    // A chosen picture is filled to the canvas by the shader, from a texture
    // uploaded once per file. One that cannot be read leaves the flag clear
    // and the solid colour paints, so a background whose file has moved still
    // previews and still exports.
    let picture = (settings.background_type == "image")
      .then_some(settings.background_image_path.as_deref())
      .flatten()
      .and_then(|path| self.background_cache.resolve(&device, path));
    let values = Constants {
      output_source: [
        settings.width as f32,
        settings.height as f32,
        source.size.0 as f32,
        source.size.1 as f32,
      ],
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
      solid_color: colour_f32(&settings.background_color)?,
      base_color: if mesh && generator_id == 0 {
        colour_f32(
          settings
            .mesh_colors
            .last()
            .expect("a validated mesh has a base colour"),
        )?
      } else {
        [0.0; 4]
      },
      recenter_inset_color: optional_colour_f32(settings.recenter_inset_color.as_deref())?,
      mesh_points,
      mesh_colors,
      effects: [
        shortest_crop * (settings.radius_percent as f32 / 100.0),
        shortest_output * (settings.background_radius_percent as f32 / 100.0),
        settings.mesh_warp_percent as f32,
        shadow_sigma,
      ],
      // The third word is how many canvas pixels one drawn pixel covers,
      // which the arrow edges feather over.
      motion: [
        composition.seconds as f32,
        generator_speed,
        if annotations.pixel_scale > 0.0 {
          annotations.pixel_scale
        } else {
          1.0
        },
        0.0,
      ],
      cursor_geometry: composition.cursor.map_or([0.0; 4], |cursor| {
        [cursor.x, cursor.y, cursor.width, cursor.height]
      }),
      cursor_effects: composition.cursor.map_or([0.0; 4], |cursor| {
        [cursor.opacity, 0.0, cursor.rotation_radians, cursor.scale]
      }),
      cursor_blur: composition.cursor.map_or([0.0; 4], |cursor| {
        [cursor.blur_delta_x, cursor.blur_delta_y, 0.0, 0.0]
      }),
      camera_frame: camera.map_or([0.0; 4], |(_, geometry, _, _)| {
        [
          geometry.frame_x as f32,
          geometry.frame_y as f32,
          geometry.frame_width as f32,
          geometry.frame_height as f32,
        ]
      }),
      camera_crop: camera.map_or([0.0; 4], |(_, geometry, _, _)| {
        [
          geometry.crop_x as f32,
          geometry.crop_y as f32,
          geometry.crop_width as f32,
          geometry.crop_height as f32,
        ]
      }),
      camera_effects: camera.map_or([0.0; 4], |(_, geometry, drop_shadow, camera_on_top)| {
        let shortest = geometry.frame_width.min(geometry.frame_height) as f32;
        let sigma = if drop_shadow {
          (shortest * 0.055).clamp(3.0, 110.0)
        } else {
          0.0
        };
        [
          geometry.radius as f32,
          1.0,
          sigma,
          if camera_on_top { 1.0 } else { 0.0 },
        ]
      }),
      magnifier: magnifier.map_or([0.0; 4], |value| value.geometry),
      magnifier_options: magnifier.map_or([0.0; 4], |value| value.options),
      magnifier_bounds: magnifier.map_or([0.0, 0.0, 1.0, 1.0], |value| value.bounds),
      native_cursor_hotspots: self.cursor_hotspots,
      options: [
        settings.mesh_seed,
        u32::from(mesh),
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
        generator_id,
        generator_color_count,
        0,
      ],
      annotation_options: [
        annotations.below_camera,
        annotations.arrows.len() as u32,
        0,
        0,
      ],
    };
    self.submit(
      context,
      &device,
      target,
      source,
      settings,
      composition,
      camera,
      picture,
      values,
      &annotations.arrows,
      &annotations.samples,
    )
  }
}
