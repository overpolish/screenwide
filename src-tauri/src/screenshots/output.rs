// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Deserialize, Serialize};

use super::mesh::MeshGradientPoint;
use super::mesh_generator::default_generator;
#[cfg(any(test, not(target_os = "macos")))]
use super::placement::output_placement;
#[cfg(any(test, not(target_os = "macos")))]
use super::CapturedImage;
use super::NormalizedSourceRect;
#[cfg(not(target_os = "macos"))]
use super::{mesh::mesh_canvas, rounded_corners};

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

#[cfg(test)]
pub(crate) mod tests {
  use super::*;

  pub(crate) fn solid_image(width: u32, height: u32, colour: [u8; 4]) -> CapturedImage {
    CapturedImage {
      height,
      rgba: colour.repeat((width * height) as usize),
      width,
    }
  }

  pub(crate) fn assert_colour_close(actual: &[u8], expected: [u8; 4]) {
    assert!(actual
      .iter()
      .zip(expected)
      .all(|(actual, expected)| actual.abs_diff(expected) <= 1));
  }

  pub(crate) fn settings(width: u32, height: u32) -> ScreenshotOutputSettings {
    // Eighty percent of the canvas wide, and the height a 2:1 source takes at
    // that width, placed ten percent in from the left and vertically centred.
    let placed_width = f64::from(width) * 0.8;
    let placed_height = placed_width / 2.0;
    let placed_x = f64::from(width) * 0.1;
    let placed_y = (f64::from(height) - placed_height) / 2.0;
    ScreenshotOutputSettings {
      background_color: "#112233".to_owned(),
      background_image_path: None,
      background_type: "solid".to_owned(),
      background_radius_percent: 0.0,
      crop_height: placed_height,
      crop_width: placed_width,
      crop_preview: None,
      crop_x: placed_x,
      crop_y: placed_y,
      drop_shadow: false,
      height,
      image_width: placed_width,
      image_x: placed_x,
      image_y: placed_y,
      legacy_mode: None,
      mesh_colors: vec![
        "#FF0000".to_owned(),
        "#00FF00".to_owned(),
        "#0000FF".to_owned(),
        "#FFFFFF".to_owned(),
        "#000000".to_owned(),
      ],
      mesh_locked_colors: vec![false; 5],
      mesh_points: vec![
        MeshGradientPoint {
          radius_x: 70.0,
          radius_y: 50.0,
          rotation: 20.0,
          x: 15.0,
          y: 15.0,
        },
        MeshGradientPoint {
          radius_x: 45.0,
          radius_y: 70.0,
          rotation: -30.0,
          x: 85.0,
          y: 15.0,
        },
        MeshGradientPoint {
          radius_x: 70.0,
          radius_y: 60.0,
          rotation: 80.0,
          x: 15.0,
          y: 85.0,
        },
        MeshGradientPoint {
          radius_x: 50.0,
          radius_y: 70.0,
          rotation: 0.0,
          x: 85.0,
          y: 85.0,
        },
      ],
      mesh_generator: default_generator(),
      mesh_seed: 42,
      mesh_warp_percent: 9.0,
      radius_percent: 0.0,
      recenter_inset_color: None,
      source_crop: NormalizedSourceRect {
        height: 1.0,
        width: 1.0,
        x: 0.0,
        y: 0.0,
      },
      width,
    }
  }

  #[test]
  fn fits_a_screenshot_inside_a_custom_coloured_canvas() {
    let output = compose_screenshot(
      &solid_image(200, 100, [200, 100, 50, 255]),
      &settings(400, 400),
    )
    .unwrap();
    let pixel = |x: u32, y: u32| {
      let start = ((y * output.width + x) * 4) as usize;
      &output.rgba[start..start + 4]
    };

    assert_eq!((output.width, output.height), (400, 400));
    assert_colour_close(pixel(0, 0), [17, 34, 51, 255]);
    assert_colour_close(pixel(200, 200), [200, 100, 50, 255]);
    assert_colour_close(pixel(200, 100), [17, 34, 51, 255]);
  }

  #[test]
  fn snaps_a_one_pixel_preview_rounding_gap_but_keeps_a_real_crop() {
    let mut output_settings = settings(401, 401);
    let exact = output_placement(200, 100, &output_settings).unwrap();
    output_settings.crop_height += 1.0;
    let snapped = output_placement(200, 100, &output_settings).unwrap();
    assert_eq!(snapped.crop_x, exact.image_x.round() as i32);
    assert_eq!(snapped.crop_y, exact.image_y.round() as i32);
    assert_eq!(snapped.crop_width, exact.image_width);
    assert_eq!(snapped.crop_height, exact.image_height);

    output_settings.crop_height += 3.0;
    let deliberate = output_placement(200, 100, &output_settings).unwrap();
    assert_ne!(deliberate.crop_height, deliberate.image_height);
  }

  #[test]
  fn places_a_source_crop_inside_the_scaled_image() {
    let mut output_settings = settings(400, 400);
    output_settings.source_crop = NormalizedSourceRect {
      x: 0.25,
      y: 0.1,
      width: 0.5,
      height: 0.6,
    };

    let placement = output_placement(200, 100, &output_settings).unwrap();
    assert_eq!(placement.source_crop_x, 120);
    assert_eq!(placement.source_crop_y, 136);
    assert_eq!(placement.source_crop_width, 160);
    assert_eq!(placement.source_crop_height, 96);
  }

  #[test]
  fn clips_an_artistically_placed_screenshot_at_the_canvas_edge() {
    let mut output_settings = settings(400, 400);
    output_settings.crop_x = -80.0;
    // The image's centre at a fifth of the canvas: its left edge is off it.
    output_settings.image_x = 80.0 - output_settings.image_width / 2.0;
    let output = compose_screenshot(
      &solid_image(200, 100, [200, 100, 50, 255]),
      &output_settings,
    )
    .unwrap();
    let pixel = |x: u32, y: u32| {
      let start = ((y * output.width + x) * 4) as usize;
      &output.rgba[start..start + 4]
    };

    assert_colour_close(pixel(0, 200), [200, 100, 50, 255]);
    assert_colour_close(pixel(300, 200), [17, 34, 51, 255]);
  }

  #[test]
  fn allows_the_image_to_cover_only_part_of_its_crop_window() {
    let mut output_settings = settings(400, 400);
    output_settings.image_x = 100.0 - output_settings.image_width / 2.0;
    let output = compose_screenshot(
      &solid_image(200, 100, [200, 100, 50, 255]),
      &output_settings,
    )
    .unwrap();
    let pixel = |x: u32, y: u32| {
      let start = ((y * output.width + x) * 4) as usize;
      &output.rgba[start..start + 4]
    };

    assert_colour_close(pixel(100, 200), [200, 100, 50, 255]);
    assert_colour_close(pixel(300, 200), [17, 34, 51, 255]);
  }

  #[test]
  fn accepts_short_hex_background_colours() {
    assert_eq!(parse_hex_colour("#12").unwrap(), [18, 18, 18, 255]);
    assert_eq!(parse_hex_colour("#123").unwrap(), [17, 34, 51, 255]);
  }

  #[test]
  fn renders_a_mesh_background_with_antibanding_grain() {
    let mut output_settings = settings(400, 400);
    output_settings.background_type = "mesh".to_owned();
    let output = compose_screenshot(
      &solid_image(200, 100, [200, 100, 50, 255]),
      &output_settings,
    )
    .unwrap();

    let corner = &output.rgba[..4];
    let opposite_corner = ((399 * output.width + 399) * 4) as usize;
    assert_ne!(corner, &output.rgba[opposite_corner..opposite_corner + 4]);
    assert!(
      (1..64).any(|x| output.rgba[(x * 4)..(x * 4 + 4)] != output.rgba[..4]),
      "the anti-banding tile should contain sub-pixel colour variation"
    );
  }

  #[test]
  fn rounds_the_custom_canvas_background() {
    let mut output_settings = settings(400, 400);
    output_settings.background_radius_percent = 10.0;
    let output = compose_screenshot(
      &solid_image(200, 100, [200, 100, 50, 255]),
      &output_settings,
    )
    .unwrap();

    assert_eq!(output.rgba[3], 0);
    let centre = ((200 * output.width + 200) * 4 + 3) as usize;
    assert_eq!(output.rgba[centre], 255);
  }

  #[test]
  fn adds_the_default_shadow_behind_the_placed_screenshot() {
    let mut output_settings = settings(400, 400);
    output_settings.drop_shadow = true;
    let output = compose_screenshot(
      &solid_image(200, 100, [200, 100, 50, 255]),
      &output_settings,
    )
    .unwrap();
    let start = ((286 * output.width + 200) * 4) as usize;

    assert_ne!(&output.rgba[start..start + 4], &[17, 34, 51, 255]);
  }
}
