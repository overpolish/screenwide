// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A screenshot composed on the CPU: built everywhere but macOS, which
//! composes on the GPU, as Windows does too.

use super::super::placement::output_placement;
use super::super::recenter::{foreground_bounds, output_inset_layer};
use super::super::{
  background_image_canvas, mesh::mesh_canvas, parse_hex_colour, rounded_corners, CapturedImage,
};
use super::{output_dimensions, ScreenshotOutputSettings, MAX_OUTPUT_PIXELS};

// Windows composes screenshots on the GPU surface instead of this CPU path.
#[cfg_attr(target_os = "windows", allow(dead_code))]
pub fn compose_screenshot(
  image: &CapturedImage,
  settings: &ScreenshotOutputSettings,
) -> Result<CapturedImage, String> {
  let (output_width, output_height) = output_dimensions(settings)?;
  let source = image::RgbaImage::from_raw(image.width, image.height, image.rgba.clone())
    .ok_or_else(|| "The screenshot pixels are not valid".to_owned())?;
  if !settings.radius_percent.is_finite()
    || !(0.0..=50.0).contains(&settings.radius_percent)
    || !settings.background_radius_percent.is_finite()
    || !(0.0..=50.0).contains(&settings.background_radius_percent)
  {
    return Err("The screenshot canvas settings are not valid".to_owned());
  }
  let placement = output_placement(image.width, image.height, settings)?;
  // The CPU resamples to whole pixels.
  let image_width = placement.image_width.round() as u32;
  let image_height = placement.image_height.round() as u32;
  if u64::from(image_width) * u64::from(image_height) > MAX_OUTPUT_PIXELS * 4 {
    return Err("The scaled screenshot is too large".to_owned());
  }
  let image_x = placement.image_x;
  let image_y = placement.image_y;
  let crop_x = f64::from(placement.crop_x);
  let crop_y = f64::from(placement.crop_y);
  let crop_width = placement.crop_width;
  let crop_height = placement.crop_height;
  let resized = image::imageops::resize(
    &source,
    image_width,
    image_height,
    image::imageops::FilterType::Lanczos3,
  );
  // Crop and image are independently movable. Recenter gives uncovered crop
  // pixels their source-derived inset; ordinary placement leaves them clear.
  let inset = settings.recenter_inset_color.is_some();
  let mut cropped = output_inset_layer(settings, crop_width, crop_height)?;
  let source_crop_local_x =
    (placement.source_crop_x - image_x.round() as i32).clamp(0, image_width as i32 - 1) as u32;
  let source_crop_local_y =
    (placement.source_crop_y - image_y.round() as i32).clamp(0, image_height as i32 - 1) as u32;
  let source_crop = image::imageops::crop_imm(
    &resized,
    source_crop_local_x,
    source_crop_local_y,
    placement
      .source_crop_width
      .min(image_width - source_crop_local_x),
    placement
      .source_crop_height
      .min(image_height - source_crop_local_y),
  )
  .to_image();
  image::imageops::overlay(
    &mut cropped,
    &source_crop,
    i64::from(placement.source_crop_x) - crop_x.round() as i64,
    i64::from(placement.source_crop_y) - crop_y.round() as i64,
  );
  let rounded = rounded_corners(
    &CapturedImage {
      height: crop_height,
      rgba: cropped.into_raw(),
      width: crop_width,
    },
    settings.radius_percent,
  );
  let solid_canvas = || -> Result<image::RgbaImage, String> {
    Ok(image::RgbaImage::from_pixel(
      output_width,
      output_height,
      image::Rgba(parse_hex_colour(&settings.background_color)?),
    ))
  };
  let mut canvas = match settings.background_type.as_str() {
    // A picture that will not load leaves the canvas on the solid colour,
    // so a background whose file has moved still exports.
    "image" => match settings
      .background_image_path
      .as_deref()
      .and_then(|path| background_image_canvas(path, output_width, output_height))
    {
      Some(picture) => picture,
      None => solid_canvas()?,
    },
    "mesh" => mesh_canvas(
      output_width,
      output_height,
      &settings.mesh_generator,
      &settings.mesh_colors,
      &settings.mesh_points,
      settings.mesh_seed,
      settings.mesh_warp_percent,
      0.0,
    )?,
    "solid" => solid_canvas()?,
    _ => return Err("The screenshot background type is not valid".to_owned()),
  };
  let foreground = image::RgbaImage::from_raw(crop_width, crop_height, rounded.rgba)
    .ok_or_else(|| "The screenshot pixels are not valid".to_owned())?;
  let placement_x = crop_x.round() as i64;
  let placement_y = crop_y.round() as i64;
  let (visible_left, visible_top, visible_right, visible_bottom) =
    foreground_bounds(placement, inset);
  let visible_width = (visible_right - visible_left).max(0.0);
  let visible_height = (visible_bottom - visible_top).max(0.0);
  let shadow_margin = visible_left
    .min(visible_top)
    .min(f64::from(output_width) - visible_right)
    .min(f64::from(output_height) - visible_bottom)
    .max(0.0);
  if settings.drop_shadow
    && visible_width > 0.0
    && visible_height > 0.0
    && shadow_margin * 0.45 > 1.0
  {
    let sigma = (visible_width.min(visible_height) * 0.055)
      .clamp(10.0, 110.0)
      .min(shadow_margin * 0.45) as f32;
    let padding = (sigma * 3.0).ceil() as u32;
    let mut shadow = image::RgbaImage::new(
      crop_width.saturating_add(padding.saturating_mul(2)),
      crop_height.saturating_add(padding.saturating_mul(2)),
    );
    for (x, y, pixel) in foreground.enumerate_pixels() {
      shadow.put_pixel(
        x + padding,
        y + padding,
        image::Rgba([0, 0, 0, ((f32::from(pixel[3]) / 255.0) * 36.0) as u8]),
      );
    }
    let shadow = image::imageops::blur(&shadow, sigma);
    let offset = (sigma * 0.35).round() as i64;
    image::imageops::overlay(
      &mut canvas,
      &shadow,
      placement_x.saturating_sub(i64::from(padding)),
      placement_y
        .saturating_sub(i64::from(padding))
        .saturating_add(offset),
    );
  }
  image::imageops::overlay(&mut canvas, &foreground, placement_x, placement_y);
  Ok(rounded_corners(
    &CapturedImage {
      height: output_height,
      rgba: canvas.into_raw(),
      width: output_width,
    },
    settings.background_radius_percent,
  ))
}
