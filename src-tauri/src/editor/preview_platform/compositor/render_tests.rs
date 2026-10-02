// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::render_test_helpers::{
  compositor, draw, read_pixel, read_top_strip, target, target_size, white_source, OUTPUT,
};
use crate::screenshots::test_output_settings;

#[test]
#[ignore = "requires a GPU adapter"]
fn aurora_background_survives_moved_foreground() {
  let compositor = compositor();
  let source = white_source(&compositor, (64, 64));
  let mut first = test_output_settings(OUTPUT.0, OUTPUT.1);
  first.background_type = "mesh".to_owned();
  first.mesh_generator = "aurora".to_owned();
  first.mesh_seed = 59_335;
  first.mesh_colors = vec!["#8FE3FF".into(), "#FF9BE3".into(), "#C9B8FF".into()];
  first.mesh_points.clear();
  first.background_radius_percent = 15.9;
  first.drop_shadow = false;
  first.image_width = 64.0;
  first.crop_width = 64.0;
  first.crop_height = 64.0;
  first.image_x = 400.0;
  first.image_y = 300.0;
  first.crop_x = 400.0;
  first.crop_y = 300.0;
  let mut second = first.clone();
  second.image_x = 1_200.0;
  second.crop_x = 1_200.0;
  let first_target = target();
  draw(&compositor, &first_target, &source, &first);
  let first_strip = read_top_strip(&first_target);
  // Row 32: the canvas's top row is its rounded background's edge, which is
  // smoothed over a pixel and so never fully opaque.
  let row = 32 * OUTPUT.0 as usize;
  let samples = [
    OUTPUT.0 as usize / 4,
    OUTPUT.0 as usize / 2,
    OUTPUT.0 as usize * 3 / 4,
  ]
  .map(|x| &first_strip[(row + x) * 4..(row + x) * 4 + 4]);
  assert!(samples.iter().all(|pixel| pixel[3] == u8::MAX));
  assert!(samples
    .iter()
    .all(|pixel| pixel[..3].iter().any(|channel| *channel != 0)));
  assert!(samples.windows(2).any(|pair| pair[0][..3] != pair[1][..3]));
  let second_target = target();
  draw(&compositor, &second_target, &source, &second);
  let second_strip = read_top_strip(&second_target);
  assert_eq!(
    first_strip, second_strip,
    "Aurora background changed when only foreground position moved"
  );
}

#[test]
#[ignore = "requires a GPU adapter"]
fn aurora_drag_trace_keeps_clear_background_stable() {
  let compositor = compositor();
  let source = white_source(&compositor, (653, 367));
  let mut settings = test_output_settings(1920, 1080);
  settings.background_type = "mesh".to_owned();
  settings.mesh_generator = "aurora".to_owned();
  settings.mesh_seed = 50_628;
  settings.mesh_colors = vec!["#8FE3FF".into(), "#FF9BE3".into(), "#C9B8FF".into()];
  settings.mesh_points.clear();
  settings.mesh_warp_percent = 13.26;
  settings.background_radius_percent = 15.9;
  settings.drop_shadow = true;
  settings.image_width = 653.15;
  settings.crop_width = 653.15;
  settings.crop_height = 367.4;
  let positions = [
    (352.35, 126.32),
    (11.78, 0.53),
    (304.79, 203.03),
    (1097.92, 428.54),
  ];
  let mut samples = Vec::new();
  for (image_x, image_y) in positions {
    settings.image_x = image_x;
    settings.image_y = image_y;
    settings.crop_x = image_x;
    settings.crop_y = image_y;
    let target = target();
    draw(&compositor, &target, &source, &settings);
    let strip = read_top_strip(&target);
    let offset = (32 * 1920 + 1440) * 4;
    let pixel = &strip[offset..offset + 4];
    assert_eq!(pixel[3], u8::MAX);
    samples.push([pixel[0], pixel[1], pixel[2]]);
  }
  assert!(
    samples.windows(2).all(|pair| pair[0] == pair[1]),
    "clear Aurora background changed across drag positions: {samples:?}"
  );
}

#[test]
#[ignore = "requires a GPU adapter"]
fn aurora_background_survives_long_live_draw_sequence() {
  let compositor = compositor();
  let source = white_source(&compositor, (1920, 1080));
  let mut settings = test_output_settings(1920, 1080);
  settings.background_type = "mesh".to_owned();
  settings.mesh_generator = "aurora".to_owned();
  settings.mesh_colors = vec!["#8FE3FF".into(), "#FF9BE3".into(), "#C9B8FF".into()];
  settings.mesh_points.clear();
  settings.mesh_warp_percent = 0.025;
  settings.background_radius_percent = 15.9;
  settings.drop_shadow = true;
  settings.mesh_seed = 34_644;
  settings.image_width = 1_074.86;
  settings.crop_width = 1_074.86;
  settings.crop_height = 604.61;
  settings.image_x = 0.0;
  settings.image_y = 0.0;
  settings.crop_x = settings.image_x;
  settings.crop_y = settings.image_y;
  let target = target_size((2048, 1280));
  let positions = [(0.0, 0.0), (1074.86, 0.0), (0.0, 475.39)];
  let mut baseline = None;
  for frame in 0..500 {
    settings.mesh_seed = 34_644;
    let (image_x, image_y) = positions[frame % positions.len()];
    settings.image_x = image_x;
    settings.image_y = image_y;
    settings.crop_x = image_x;
    settings.crop_y = image_y;
    draw(&compositor, &target, &source, &settings);
    if frame % 10 == 0 {
      let pixel = read_pixel(&target, 1500, 700);
      assert_eq!(pixel[3], u8::MAX);
      if let Some(previous) = baseline {
        assert_eq!(
          previous, pixel,
          "Aurora changed during repeated live draws at draw {frame}"
        );
      } else {
        baseline = Some(pixel);
      }
    }
  }
}
