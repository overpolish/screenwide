// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::output::{output_dimensions, ScreenshotOutputSettings};

#[derive(Clone, Copy, Debug)]
pub struct OutputPlacement {
  pub crop_height: u32,
  pub crop_width: u32,
  pub crop_x: i32,
  pub crop_y: i32,
  /// The image's size as placed, not rounded: its height follows its width
  /// by the source's aspect, so the picture keeps one scale on both axes,
  /// as a scaled video export relies on (see `FrameFit` in the export).
  pub image_height: f64,
  pub image_width: f64,
  pub image_x: f64,
  pub image_y: f64,
  pub source_crop_height: u32,
  pub source_crop_width: u32,
  pub source_crop_x: i32,
  pub source_crop_y: i32,
}

/// Resolve a layer's stored pixel placement into whole output pixels.
///
/// Placement arrives in the output's own pixels, so the work here is limited
/// to validating it, deriving the image's height from the source's aspect and
/// rounding the crops to the integer rectangles the compositor clips by.
pub fn output_placement(
  source_width: u32,
  source_height: u32,
  settings: &ScreenshotOutputSettings,
) -> Result<OutputPlacement, String> {
  let (output_width, output_height) = output_dimensions(settings)?;
  let pixels = [
    settings.crop_height,
    settings.crop_width,
    settings.crop_x,
    settings.crop_y,
    settings.image_width,
    settings.image_x,
    settings.image_y,
  ];
  settings.source_crop.validate()?;
  let source_crop = settings.source_crop;
  // Placement may sit outside the canvas, but never by more than the same
  // eight canvases the percentages used to allow.
  let width_limit = f64::from(output_width) * 8.0;
  let height_limit = f64::from(output_height) * 8.0;
  if source_width == 0
    || source_height == 0
    || pixels.iter().any(|value| !value.is_finite())
    || !(1.0..=width_limit).contains(&settings.crop_width)
    || !(1.0..=height_limit).contains(&settings.crop_height)
    || settings.crop_x.abs() > width_limit
    || settings.crop_y.abs() > height_limit
    || !(1.0..=width_limit).contains(&settings.image_width)
    || settings.image_x.abs() > width_limit
    || settings.image_y.abs() > height_limit
  {
    return Err("The screenshot placement is not valid".to_owned());
  }
  let image_width = settings.image_width.max(1.0);
  let image_height = (image_width * f64::from(source_height) / f64::from(source_width)).max(1.0);
  let mut crop_height = settings.crop_height.round().max(1.0) as u32;
  let mut crop_width = settings.crop_width.round().max(1.0) as u32;
  let mut crop_x = settings.crop_x.round() as i32;
  let mut crop_y = settings.crop_y.round() as i32;
  let image_x = settings.image_x;
  let image_y = settings.image_y;
  let source = [
    image_x + image_width * source_crop.x,
    image_y + image_height * source_crop.y,
    image_width * source_crop.width,
    image_height * source_crop.height,
  ];
  // A scene that pads its box shows the picture only in the window it
  // padded; the rest of the crop is the inset colour.
  let [source_crop_x, source_crop_y, source_width, source_height] = settings
    .scene_window
    .map_or(source, |window| windowed(source, window));
  let source_crop_width = source_width.round().max(1.0) as u32;
  let source_crop_height = source_height.round().max(1.0) as u32;
  // Preview scaling can round otherwise coincident frame/source edges apart.
  let image_left = image_x.round() as i32;
  let image_top = image_y.round() as i32;
  let image_right = image_left.saturating_add(image_width.round() as i32);
  let image_bottom = image_top.saturating_add(image_height.round() as i32);
  let crop_right = crop_x.saturating_add(crop_width as i32);
  let crop_bottom = crop_y.saturating_add(crop_height as i32);
  if (crop_x - image_left).abs() <= 1
    && (crop_right - image_right).abs() <= 1
    && (crop_y - image_top).abs() <= 1
    && (crop_bottom - image_bottom).abs() <= 1
  {
    crop_x = image_left;
    crop_y = image_top;
    crop_width = image_width.round() as u32;
    crop_height = image_height.round() as u32;
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

/// `source`, a left, top, width and height, cut to what of it lies inside
/// `window`; a source wholly outside it keeps no width or height, at the
/// window's nearest edge.
fn windowed(source: [f64; 4], window: [f64; 4]) -> [f64; 4] {
  let left = source[0].max(window[0]);
  let top = source[1].max(window[1]);
  let right = (source[0] + source[2]).min(window[0] + window[2]);
  let bottom = (source[1] + source[3]).min(window[1] + window[3]);
  [left, top, (right - left).max(0.0), (bottom - top).max(0.0)]
}
