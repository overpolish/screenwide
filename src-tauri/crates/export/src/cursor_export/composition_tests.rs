// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use screenwide_output::{MeshGradientPoint, NormalizedSourceRect};

fn output_settings(width: u32, height: u32) -> ScreenshotOutputSettings {
  ScreenshotOutputSettings {
    annotations: Vec::new(),
    background_color: "#000000".to_owned(),
    background_image_path: None,
    background_type: "color".to_owned(),
    background_radius_percent: 0.0,
    capture_width_points: 0.0,
    capture_scale: 0.0,
    capture_corner_radius: 0.0,
    clear_area: None,
    crop_height: f64::from(height),
    crop_width: f64::from(width),
    crop_preview: None,
    crop_x: 0.0,
    crop_y: 0.0,
    drop_shadow: false,
    drop_shadow_strength: None,
    height,
    image_width: f64::from(width),
    image_x: 0.0,
    image_y: 0.0,
    inset: None,
    legacy_mode: None,
    scene_motion: None,
    scene_opacity: None,
    scene_camera_front: None,
    scene_window: None,
    background_turn: None,
    camera_shadow_strength: None,
    rotation: None,
    mesh_colors: Vec::new(),
    mesh_generator: String::new(),
    mesh_locked_colors: Vec::new(),
    mesh_points: Vec::<MeshGradientPoint>::new(),
    mesh_seed: 0,
    mesh_warp_percent: 0.0,
    radius: screenwide_scenes::Measure::default(),
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
fn recenter_inset_routes_video_through_composition() {
  let mut settings = output_settings(1_920, 1_080);
  assert!(!needs_composition(&settings, 1_920, 1_080));

  settings.recenter_inset_color = Some("#445566".to_owned());
  assert!(needs_composition(&settings, 1_920, 1_080));
}

#[test]
fn source_crop_routes_video_through_composition() {
  let mut settings = output_settings(1_920, 1_080);
  settings.source_crop = NormalizedSourceRect {
    height: 0.8,
    width: 0.75,
    x: 0.1,
    y: 0.1,
  };

  assert!(needs_composition(&settings, 1_920, 1_080));
}
