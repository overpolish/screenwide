// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Output settings and pictures for tests, here and in the app.

use screenwide_imaging::CapturedImage;

use crate::mesh_generator::default_generator;
use crate::{MeshGradientPoint, NormalizedSourceRect, ScreenshotOutputSettings};

pub fn solid_image(width: u32, height: u32, colour: [u8; 4]) -> CapturedImage {
  CapturedImage {
    height,
    rgba: colour.repeat((width * height) as usize),
    width,
  }
}

pub fn assert_colour_close(actual: &[u8], expected: [u8; 4]) {
  assert!(actual
    .iter()
    .zip(expected)
    .all(|(actual, expected)| actual.abs_diff(expected) <= 1));
}

pub fn test_output_settings(width: u32, height: u32) -> ScreenshotOutputSettings {
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
    capture_scale: 0.0,
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
    scene_motion: None,
    scene_opacity: None,
    scene_camera_front: None,
    scene_rotation: None,
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
