// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(any(test, not(target_os = "macos")))]
use super::parse_hex_colour;
#[cfg(not(target_os = "macos"))]
use super::{placement::OutputPlacement, ScreenshotOutputSettings};

#[cfg(not(target_os = "macos"))]
pub(super) fn output_inset_layer(
  settings: &ScreenshotOutputSettings,
  width: u32,
  height: u32,
) -> Result<image::RgbaImage, String> {
  inset_layer(settings.recenter_inset_color.as_deref(), width, height)
}

#[cfg(any(test, not(target_os = "macos")))]
pub(super) fn inset_layer(
  colour: Option<&str>,
  width: u32,
  height: u32,
) -> Result<image::RgbaImage, String> {
  let Some(colour) = colour else {
    return Ok(image::RgbaImage::new(width, height));
  };
  Ok(image::RgbaImage::from_pixel(
    width,
    height,
    image::Rgba(parse_hex_colour(colour)?),
  ))
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn foreground_bounds(placement: OutputPlacement, inset: bool) -> (f64, f64, f64, f64) {
  let crop_x = f64::from(placement.crop_x);
  let crop_y = f64::from(placement.crop_y);
  if inset {
    return (
      crop_x,
      crop_y,
      crop_x + f64::from(placement.crop_width),
      crop_y + f64::from(placement.crop_height),
    );
  }
  (
    crop_x.max(f64::from(placement.source_crop_x)),
    crop_y.max(f64::from(placement.source_crop_y)),
    (crop_x + f64::from(placement.crop_width))
      .min(f64::from(placement.source_crop_x) + f64::from(placement.source_crop_width)),
    (crop_y + f64::from(placement.crop_height))
      .min(f64::from(placement.source_crop_y) + f64::from(placement.source_crop_height)),
  )
}

#[cfg(target_os = "windows")]
pub(crate) fn foreground_bounds_f32(
  placement: OutputPlacement,
  inset: bool,
) -> (f32, f32, f32, f32) {
  let (left, top, right, bottom) = foreground_bounds(placement, inset);
  (left as f32, top as f32, right as f32, bottom as f32)
}

#[cfg(target_os = "windows")]
pub(crate) fn optional_colour_f32(colour: Option<&str>) -> Result<[f32; 4], String> {
  colour.map_or(Ok([0.0; 4]), colour_f32)
}

#[cfg(target_os = "windows")]
pub(crate) fn colour_f32(value: &str) -> Result<[f32; 4], String> {
  Ok(parse_hex_colour(value)?.map(|channel| f32::from(channel) / 255.0))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn inset_layer_uses_its_own_colour() {
    let image = inset_layer(Some("#445566"), 2, 2).unwrap();
    assert_eq!(image.get_pixel(0, 0).0, [68, 85, 102, 255]);
  }

  #[test]
  fn inset_stays_distinct_from_the_canvas_background() {
    let mut settings = super::super::output::tests::settings(400, 400);
    settings.recenter_inset_color = Some("#445566".to_owned());
    settings.screenshot_image_x_percent = 25.0;
    let output = super::super::output::compose_screenshot(
      &super::super::output::tests::solid_image(200, 100, [200, 100, 50, 255]),
      &settings,
    )
    .unwrap();
    let pixel = |x: u32, y: u32| {
      let start = ((y * output.width + x) * 4) as usize;
      &output.rgba[start..start + 4]
    };
    let assert = super::super::output::tests::assert_colour_close;
    assert(pixel(100, 200), [200, 100, 50, 255]);
    assert(pixel(300, 200), [68, 85, 102, 255]);
    assert(pixel(390, 200), [17, 34, 51, 255]);
  }

  #[test]
  fn source_crop_reveals_inset_without_changing_the_outer_frame() {
    let mut settings = super::super::output::tests::settings(400, 400);
    settings.recenter_inset_color = Some("#445566".to_owned());
    settings.source_crop.x = 0.5;
    settings.source_crop.width = 0.5;
    let output = super::super::output::compose_screenshot(
      &super::super::output::tests::solid_image(200, 100, [200, 100, 50, 255]),
      &settings,
    )
    .unwrap();
    let pixel = |x: u32, y: u32| {
      let start = ((y * output.width + x) * 4) as usize;
      &output.rgba[start..start + 4]
    };
    let assert = super::super::output::tests::assert_colour_close;
    assert(pixel(100, 200), [68, 85, 102, 255]);
    assert(pixel(300, 200), [200, 100, 50, 255]);
    assert(pixel(390, 200), [17, 34, 51, 255]);
  }
}
