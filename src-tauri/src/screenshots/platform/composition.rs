// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(crate) fn native_canvas(
  source_width: u32,
  source_height: u32,
  settings: &ScreenshotOutputSettings,
  transparent_background: bool,
) -> Result<NativeCanvas, String> {
  super::super::validate_output_settings(source_width, source_height, settings)?;
  let placement = output_placement(source_width, source_height, settings)?;
  let colour = parse_hex_colour(&settings.background_color)?;
  let inset_colour = settings
    .recenter_inset_color
    .as_deref()
    .map(parse_hex_colour)
    .transpose()?;
  let generator =
    mesh_generator(&settings.mesh_generator).ok_or("The mesh generator is unknown")?;
  // The shader works in straight RGBA, and every canvas colour is opaque.
  let opaque = |colour: [u8; 4]| {
    let [red, green, blue, _] = colour.map(|value| f32::from(value) / 255.0);
    [red, green, blue, 1.0]
  };
  let mut canvas = NativeCanvas {
    background_color: opaque(colour),
    recenter_inset_color: inset_colour.map_or([0.0; 4], opaque),
    background_radius: (f64::from(settings.width.min(settings.height))
      * settings.background_radius_percent
      / 100.0)
      .round() as u32,
    crop_x: placement.crop_x,
    crop_y: placement.crop_y,
    crop_width: placement.crop_width,
    crop_height: placement.crop_height,
    image_x: placement.image_x as f32,
    image_y: placement.image_y as f32,
    image_width: placement.image_width,
    image_height: placement.image_height,
    source_crop_x: placement.source_crop_x,
    source_crop_y: placement.source_crop_y,
    source_crop_width: placement.source_crop_width,
    source_crop_height: placement.source_crop_height,
    // Crop mode flattens the ghost underneath: its rounding and shadow move
    // to the cropped layer drawn over it, below.
    radius: if settings.crop_preview.is_some() {
      0
    } else {
      (f64::from(placement.crop_width.min(placement.crop_height)) * settings.radius_percent / 100.0)
        .round() as u32
    },
    drop_shadow: u32::from(settings.drop_shadow && settings.crop_preview.is_none()),
    mesh_enabled: u32::from(settings.background_type == "mesh"),
    mesh_seed: settings.mesh_seed,
    mesh_warp_percent: settings.mesh_warp_percent as f32,
    mesh_point_count: settings.mesh_points.len() as u32,
    mesh_generator: generator.id,
    mesh_generator_color_count: generator.color_count as u32,
    mesh_generator_speed: generator.speed,
    transparent_background: u32::from(transparent_background),
    ..Default::default()
  };
  if let Some(crop) = settings.crop_preview {
    // The radius is a percentage of the layer's shorter side, and in crop mode
    // that layer is the crop rectangle rather than the whole source.
    canvas.crop_preview = 1;
    canvas.crop_preview_x = crop.x as f32;
    canvas.crop_preview_y = crop.y as f32;
    canvas.crop_preview_width = crop.width.max(0.0) as f32;
    canvas.crop_preview_height = crop.height.max(0.0) as f32;
    canvas.crop_preview_radius = (crop.width.min(crop.height).max(0.0)
      * settings.radius_percent.clamp(0.0, 50.0)
      / 100.0) as f32;
    canvas.crop_preview_drop_shadow = u32::from(settings.drop_shadow);
  }
  (canvas.has_background_image, canvas.background_image_id) =
    super::super::background_image::native::canvas_picture(settings);
  for (index, point) in settings.mesh_points.iter().take(4).enumerate() {
    let angle = point.rotation.to_radians() as f32;
    canvas.mesh_points[index] = [
      point.x as f32 / 100.0,
      point.y as f32 / 100.0,
      point.radius_x as f32 / 100.0,
      point.radius_y as f32 / 100.0,
      angle.cos(),
      angle.sin(),
      0.0,
      0.0,
    ];
  }
  canvas.mesh_colors = canvas_colors(generator, &settings.mesh_colors)?;
  Ok(canvas)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn compose_output_layers(
  image: &CapturedImage,
  settings: &ScreenshotOutputSettings,
  seconds: f64,
  transparent_background: bool,
  cursor: Option<(&GpuCursor, &[GpuArtwork])>,
  camera: Option<&CapturedImage>,
  overlay: Option<&StillOverlay>,
  keyboard: Option<&KeyboardOverlay>,
  clip_cursor_at_video_edge: bool,
  foreground_only: bool,
) -> Result<CapturedImage, String> {
  let mut canvas = native_canvas(image.width, image.height, settings, transparent_background)?;
  canvas.clip_cursor_at_video_edge = u32::from(clip_cursor_at_video_edge);
  canvas.foreground_only = u32::from(foreground_only);
  let mut rgba = vec![0_u8; settings.width as usize * settings.height as usize * 4];
  let mut error = vec![0_i8; 2_048];
  let native_cursor = NativeGpuCursor::from(cursor.map(|(cursor, _)| *cursor));
  let native_artworks = cursor
    .map_or(&[][..], |(_, artworks)| artworks)
    .iter()
    .map(NativeGpuArtwork::from)
    .collect::<Vec<_>>();
  let result = unsafe {
    screenwide_gpu_composite_still(
      image.rgba.as_ptr(),
      image.width,
      image.height,
      &canvas,
      settings.width,
      settings.height,
      seconds,
      &native_cursor,
      native_artworks.as_ptr(),
      native_artworks.len().try_into().unwrap_or(u32::MAX),
      camera.map_or(std::ptr::null(), |image| image.rgba.as_ptr()),
      overlay.map_or(std::ptr::null(), std::ptr::from_ref),
      keyboard.map_or(std::ptr::null(), std::ptr::from_ref),
      rgba.as_mut_ptr(),
      error.as_mut_ptr(),
      error.len(),
    )
  };
  if result == 0 {
    let message = unsafe { std::ffi::CStr::from_ptr(error.as_ptr()) }
      .to_string_lossy()
      .into_owned();
    return Err(if message.is_empty() {
      "The native screenshot compositor failed".to_owned()
    } else {
      message
    });
  }
  Ok(CapturedImage {
    height: settings.height,
    rgba,
    width: settings.width,
  })
}

pub(crate) fn alpha_composite(
  base: &CapturedImage,
  overlay: &CapturedImage,
) -> Result<CapturedImage, String> {
  if base.width != overlay.width || base.height != overlay.height {
    return Err("The screenshot layers do not share a canvas size".to_owned());
  }
  let mut rgba = vec![0_u8; base.rgba.len()];
  let mut error = vec![0_i8; 2_048];
  let result = unsafe {
    screenwide_gpu_alpha_composite(
      base.rgba.as_ptr(),
      overlay.rgba.as_ptr(),
      base.width,
      base.height,
      rgba.as_mut_ptr(),
      error.as_mut_ptr(),
      error.len(),
    )
  };
  if result == 0 {
    let message = unsafe { std::ffi::CStr::from_ptr(error.as_ptr()) }
      .to_string_lossy()
      .into_owned();
    return Err(if message.is_empty() {
      "The native screenshot layer compositor failed".to_owned()
    } else {
      message
    });
  }
  Ok(CapturedImage {
    height: base.height,
    rgba,
    width: base.width,
  })
}
