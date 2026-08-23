// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::output::{output_dimensions, ScreenshotOutputSettings};

#[derive(Clone, Copy, Debug)]
pub(crate) struct OutputPlacement {
  pub crop_height: u32,
  pub crop_width: u32,
  pub crop_x: i32,
  pub crop_y: i32,
  pub image_height: u32,
  pub image_width: u32,
  pub image_x: f64,
  pub image_y: f64,
  pub source_crop_height: u32,
  pub source_crop_width: u32,
  pub source_crop_x: i32,
  pub source_crop_y: i32,
}

pub(crate) fn output_placement(
  source_width: u32,
  source_height: u32,
  settings: &ScreenshotOutputSettings,
) -> Result<OutputPlacement, String> {
  let (output_width, output_height) = output_dimensions(settings)?;
  let percentages = [
    settings.screenshot_crop_height_percent,
    settings.screenshot_crop_width_percent,
    settings.screenshot_crop_x_percent,
    settings.screenshot_crop_y_percent,
    settings.screenshot_image_width_percent,
    settings.screenshot_image_x_percent,
    settings.screenshot_image_y_percent,
  ];
  settings.source_crop.validate()?;
  let source_crop = settings.source_crop;
  if source_width == 0
    || source_height == 0
    || percentages.iter().any(|value| !value.is_finite())
    || !(1.0..=800.0).contains(&settings.screenshot_crop_width_percent)
    || !(1.0..=800.0).contains(&settings.screenshot_crop_height_percent)
    || settings.screenshot_crop_x_percent.abs() > 800.0
    || settings.screenshot_crop_y_percent.abs() > 800.0
    || !(1.0..=800.0).contains(&settings.screenshot_image_width_percent)
  {
    return Err("The screenshot placement is not valid".to_owned());
  }
  let image_width = (f64::from(output_width) * settings.screenshot_image_width_percent / 100.0)
    .round()
    .max(1.0) as u32;
  let image_height = (f64::from(image_width) * f64::from(source_height) / f64::from(source_width))
    .round()
    .max(1.0) as u32;
  let mut crop_height = (f64::from(output_height) * settings.screenshot_crop_height_percent / 100.0)
    .round()
    .max(1.0) as u32;
  let mut crop_width = (f64::from(output_width) * settings.screenshot_crop_width_percent / 100.0)
    .round()
    .max(1.0) as u32;
  let mut crop_x =
    (f64::from(output_width) * settings.screenshot_crop_x_percent / 100.0).round() as i32;
  let mut crop_y =
    (f64::from(output_height) * settings.screenshot_crop_y_percent / 100.0).round() as i32;
  let image_x = f64::from(output_width) * settings.screenshot_image_x_percent / 100.0
    - f64::from(image_width) / 2.0;
  let image_y = f64::from(output_height) * settings.screenshot_image_y_percent / 100.0
    - f64::from(image_height) / 2.0;
  let source_crop_x = image_x + f64::from(image_width) * source_crop.x;
  let source_crop_y = image_y + f64::from(image_height) * source_crop.y;
  let source_crop_width = (f64::from(image_width) * source_crop.width)
    .round()
    .max(1.0) as u32;
  let source_crop_height = (f64::from(image_height) * source_crop.height)
    .round()
    .max(1.0) as u32;
  // Preview scaling can round otherwise coincident frame/source edges apart.
  let image_left = image_x.round() as i32;
  let image_top = image_y.round() as i32;
  let image_right = image_left.saturating_add(image_width as i32);
  let image_bottom = image_top.saturating_add(image_height as i32);
  let crop_right = crop_x.saturating_add(crop_width as i32);
  let crop_bottom = crop_y.saturating_add(crop_height as i32);
  if (crop_x - image_left).abs() <= 1
    && (crop_right - image_right).abs() <= 1
    && (crop_y - image_top).abs() <= 1
    && (crop_bottom - image_bottom).abs() <= 1
  {
    crop_x = image_left;
    crop_y = image_top;
    crop_width = image_width;
    crop_height = image_height;
  }
  Ok(OutputPlacement {
    crop_height,
    crop_width,
    crop_x,
    crop_y,
    image_height,
    image_width,
    image_x,
    image_y,
    source_crop_height,
    source_crop_width,
    source_crop_x: source_crop_x.round() as i32,
    source_crop_y: source_crop_y.round() as i32,
  })
}
