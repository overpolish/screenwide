// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

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
    annotations: Vec::new(),
    background_color: "#112233".to_owned(),
    background_image_path: None,
    background_type: "solid".to_owned(),
    background_radius_percent: 0.0,
    capture_width_points: 0.0,
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
