// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(test)]
#[path = "output/tests.rs"]
pub(crate) mod tests;

use serde::{Deserialize, Serialize};

#[cfg(any(test, not(target_os = "macos")))]
use super::placement::output_placement;
#[cfg(any(test, not(target_os = "macos")))]
use super::CapturedImage;
use super::NormalizedSourceRect;
#[cfg(not(target_os = "macos"))]
use super::{mesh::mesh_canvas, rounded_corners};
use super::{mesh::MeshGradientPoint, mesh_generator::default_generator};
use crate::editor::annotations::Annotation;

const MAX_OUTPUT_PIXELS: u64 = 120_000_000;

/// The crop tool's live result rectangle, in output pixels.
///
/// Crop mode previews the whole source so the part being cropped away stays
/// visible. This is where the cropped layer itself lands inside that canvas,
/// so the compositor can draw it a second time with its real corner radius
/// and drop shadow.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CropPreviewRect {
  pub height: f64,
  pub width: f64,
  pub x: f64,
  pub y: f64,
}

/// One layer's canvas and its placement in it.
///
/// Placement is in output pixels rather than in shares of the canvas, so the
/// canvas can be resized without moving or rescaling anything placed in it.
/// The crop is the visible rectangle; the image behind it is given by its top
/// left corner and its width, its height following the source's aspect.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenshotOutputSettings {
  /// Marks drawn over this layer, in the source's own pixel space.
  #[serde(default)]
  pub annotations: Vec<Annotation>,
  pub background_color: String,
  /// A picture of your own behind the layers. Absent for every background
  /// that is painted rather than loaded, and for settings written before
  /// pictures could be chosen.
  #[serde(default)]
  pub background_image_path: Option<String>,
  pub background_type: String,
  pub background_radius_percent: f64,
  /// The visible rectangle, in output pixels. Zero width or height marks a
  /// settings blob written before placement moved into pixels.
  #[serde(default)]
  pub crop_height: f64,
  /// While the crop tool is open the preview draws the whole uncropped source
  /// flat as a ghost; this is the committed crop rectangle, in output pixels,
  /// that the compositor draws over it with the real corner radius and drop
  /// shadow. Only the crop-mode preview payload carries one - it is never
  /// persisted and never reaches an export.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub crop_preview: Option<CropPreviewRect>,
  #[serde(default)]
  pub crop_width: f64,
  #[serde(default)]
  pub crop_x: f64,
  #[serde(default)]
  pub crop_y: f64,
  pub drop_shadow: bool,
  pub height: u32,
  /// The whole image's top left corner and width, in output pixels.
  #[serde(default)]
  pub image_width: f64,
  #[serde(default)]
  pub image_x: f64,
  #[serde(default)]
  pub image_y: f64,
  #[serde(default, rename = "mode", skip_serializing)]
  pub legacy_mode: Option<String>,
  pub mesh_colors: Vec<String>,
  /// Which picture the mesh background paints. Settings written before the
  /// ported generators existed carry none, and are the app's own blob mesh.
  #[serde(default = "default_generator")]
  pub mesh_generator: String,
  #[serde(default)]
  pub mesh_locked_colors: Vec<bool>,
  pub mesh_points: Vec<MeshGradientPoint>,
  pub mesh_seed: u32,
  pub mesh_warp_percent: f64,
  pub radius_percent: f64,
  #[serde(default)]
  pub recenter_inset_color: Option<String>,
  pub source_crop: NormalizedSourceRect,
  pub width: u32,
}

impl ScreenshotOutputSettings {
  /// Whether this carries a placement at all. Settings written before
  /// placement was measured in output pixels deserialize without one, and are
  /// discarded rather than converted.
  pub fn has_placement(&self) -> bool {
    self.crop_width > 0.0 && self.crop_height > 0.0 && self.image_width > 0.0
  }
}

pub(crate) fn parse_hex_colour(value: &str) -> Result<[u8; 4], String> {
  let value = value.strip_prefix('#').unwrap_or(value);
  if !matches!(value.len(), 2 | 3 | 6) || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
    return Err("The screenshot background colour is not valid".to_owned());
  }
  let expanded = match value.len() {
    2 => value.repeat(3),
    3 => value.chars().flat_map(|character| [character; 2]).collect(),
    _ => value.to_owned(),
  };
  let channel =
    |start| u8::from_str_radix(&expanded[start..start + 2], 16).map_err(|e| e.to_string());
  Ok([channel(0)?, channel(2)?, channel(4)?, u8::MAX])
}

pub(crate) fn output_dimensions(settings: &ScreenshotOutputSettings) -> Result<(u32, u32), String> {
  if settings.width < 64
    || settings.height < 64
    || u64::from(settings.width) * u64::from(settings.height) > MAX_OUTPUT_PIXELS
  {
    return Err("The screenshot output dimensions are not valid".to_owned());
  }
  Ok((settings.width, settings.height))
}

#[cfg(all(target_os = "macos", test))]
pub fn compose_screenshot(
  image: &CapturedImage,
  settings: &ScreenshotOutputSettings,
) -> Result<CapturedImage, String> {
  super::platform::compose_output_layers(
    image, settings, 0.0, true, None, None, None, None, false, false,
  )
}

#[cfg(not(target_os = "macos"))]
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
  let image_width = placement.image_width;
  let image_height = placement.image_height;
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
  let mut cropped = super::recenter::output_inset_layer(settings, crop_width, crop_height)?;
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
      .and_then(|path| super::background_image_canvas(path, output_width, output_height))
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
    super::recenter::foreground_bounds(placement, inset);
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
