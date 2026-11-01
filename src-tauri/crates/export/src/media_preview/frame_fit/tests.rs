// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

fn at(resolution_scale_percent: u16, source_scale_percent: u16) -> VideoExportOptions {
  VideoExportOptions {
    compression: 2,
    resolution_scale_percent,
    source_scale_percent,
  }
}

#[test]
fn keeps_one_scale_where_a_scaled_side_falls_on_a_half_pixel() {
  // Smaller: 2338 rows at 75% are 1753.5, which the frame rounds to the
  // nearest even side and centres the canvas in, a quarter pixel padded.
  let smaller = FrameFit::of(3_600, 2_338, at(75, 100));
  assert_eq!((smaller.width, smaller.height), (2_700, 1_754));
  assert_eq!(smaller.scale, 0.75);
  assert_eq!((smaller.x, smaller.y), (0.0, 0.25));
  assert_eq!(smaller.canvas(), [0.0, 0.25, 2_700.0, 1_753.5]);

  // Smallest: 1169 rows round down to 1168, half a pixel cropped.
  let smallest = FrameFit::of(3_600, 2_338, at(50, 100));
  assert_eq!((smallest.width, smallest.height), (1_800, 1_168));
  assert_eq!(smallest.scale, 0.5);
  assert_eq!((smallest.x, smallest.y), (0.0, -0.5));
}

#[test]
fn measures_a_retina_canvas_against_its_own_scale() {
  assert_eq!(
    FrameFit::of(3_600, 2_338, at(150, 200)),
    FrameFit::of(3_600, 2_338, at(75, 100))
  );
  let fit = FrameFit::of(3_024, 1_964, at(75, 200));
  assert_eq!((fit.width, fit.height), (1_134, 736));
  assert_eq!(fit.scale, 0.375);
  assert_eq!((fit.x, fit.y), (0.0, -0.25));
  let fit = FrameFit::of(2_560, 1_600, at(125, 200));
  assert_eq!((fit.width, fit.height), (1_600, 1_000));
  assert_eq!((fit.x, fit.y), (0.0, 0.0));
}

#[test]
fn rounds_an_odd_side_at_its_own_size_down_to_even() {
  let fit = FrameFit::of(1_919, 1_079, at(100, 100));
  assert_eq!((fit.width, fit.height), (1_918, 1_078));
  assert_eq!(fit.scale, 1.0);
  assert_eq!((fit.x, fit.y), (-0.5, -0.5));
  let fit = FrameFit::of(1_512, 982, at(50, 100));
  assert_eq!((fit.width, fit.height), (756, 490));
  assert_eq!(fit.y, -0.5);
}

#[test]
fn crops_or_pads_at_most_a_pixel_and_keeps_both_sides_even() {
  for (resolution, source) in [
    (25, 100),
    (50, 100),
    (75, 100),
    (75, 200),
    (150, 200),
    (60, 125),
  ] {
    for side in (64..2_400).step_by(7) {
      let fit = FrameFit::of(side, side + 3, at(resolution, source));
      assert_eq!(fit.width % 2, 0);
      assert_eq!(fit.height % 2, 0);
      assert!(
        fit.x.abs() <= 0.5,
        "{side} at {resolution}/{source}: {fit:?}"
      );
      assert!(
        fit.y.abs() <= 0.5,
        "{side} at {resolution}/{source}: {fit:?}"
      );
    }
  }
}

/// Where a canvas row `y` drawn into `exported` lands on the Original frame
/// once the compare carries the exported frame onto it, as the island does:
/// one frame's canvas rectangle laid over the other's.
fn compared_row(y: f64, original: FrameFit, exported: FrameFit) -> f64 {
  let [_, original_top, _, original_height] = original.canvas();
  let [_, exported_top, _, exported_height] = exported.canvas();
  original_top + (exported.y(y) - exported_top) * original_height / exported_height
}

#[test]
fn lines_up_the_top_and_bottom_rows_in_the_compare() {
  for (height, resolution) in [
    (2_338, 75),
    (2_338, 50),
    (1_079, 75),
    (1_964, 25),
    (982, 60),
  ] {
    let original = FrameFit::of(3_600, height, at(100, 100));
    let exported = FrameFit::of(3_600, height, at(resolution, 100));
    for row in [
      0.0,
      1.0,
      f64::from(height) / 2.0,
      f64::from(height) - 1.0,
      f64::from(height),
    ] {
      let shown = compared_row(row, original, exported);
      assert!(
        (shown - original.y(row)).abs() <= 0.5,
        "row {row} of {height} at {resolution}% shows at {shown}, not {}",
        original.y(row)
      );
    }
  }
}

#[test]
fn places_a_baked_camera_on_the_frame_at_the_same_scale() {
  let overlay = CameraOverlaySettings {
    camera_x: 1_800.0,
    camera_y: 2_000.0,
    camera_width: 800.0,
    frame_height: 400.0,
    frame_width: 600.0,
    frame_x: 1_500.0,
    frame_y: 1_800.0,
    radius_percent: 10.0,
    follow_face: false,
  };
  let fit = FrameFit::of(3_600, 2_338, at(75, 100));
  let placed = fit.overlay(overlay);
  assert_eq!((placed.camera_x, placed.camera_y), (1_350.0, 1_500.25));
  assert_eq!(placed.camera_width, 600.0);
  assert_eq!((placed.frame_x, placed.frame_y), (1_125.0, 1_350.25));
  assert_eq!((placed.frame_width, placed.frame_height), (450.0, 300.0));
  assert_eq!(placed.radius_percent, 10.0);
}
