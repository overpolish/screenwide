// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::parse_hex_colour;
use super::placement::OutputPlacement;
#[cfg(not(target_os = "macos"))]
use super::ScreenshotOutputSettings;

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

#[cfg(any(target_os = "macos", target_os = "windows"))]
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

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub fn foreground_bounds_f32(placement: OutputPlacement, inset: bool) -> (f32, f32, f32, f32) {
  let (left, top, right, bottom) = foreground_bounds(placement, inset);
  (left as f32, top as f32, right as f32, bottom as f32)
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub fn optional_colour_f32(colour: Option<&str>) -> Result<[f32; 4], String> {
  colour.map_or(Ok([0.0; 4]), colour_f32)
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub fn colour_f32(value: &str) -> Result<[f32; 4], String> {
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
}
