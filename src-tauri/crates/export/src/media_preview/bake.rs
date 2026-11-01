// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#![cfg_attr(not(target_os = "macos"), allow(dead_code))]

use super::*;

fn even(value: f64) -> u32 {
  ((value.round().max(2.0) as u32) & !1).max(2)
}

pub fn bake_geometry(options: BakedVideoExportOptions) -> Result<BakeGeometry, String> {
  // The overlay is placed in the screen output's own pixels. A bake may render
  // at a scaled size, so carry the placement across as the screen is.
  let fit = FrameFit::of(options.screen_width, options.screen_height, options.video);
  let (output_width, output_height) = (fit.width, fit.height);
  let overlay = fit.overlay(options.overlay);
  let frame_x = overlay.frame_x;
  let frame_y = overlay.frame_y;
  let frame_width = overlay.frame_width;
  let frame_height = overlay.frame_height;
  let camera_width = overlay.camera_width;
  let camera_height =
    camera_width * f64::from(options.camera_height) / f64::from(options.camera_width.max(1));
  let camera_x = overlay.camera_x - camera_width / 2.0;
  let camera_y = overlay.camera_y - camera_height / 2.0;
  // The frame is gently clamped into the camera image instead of rejected:
  // aspect-dependent defaults (and a crop-tool reset) can land the frame
  // slightly outside the camera, and failing every composition over that
  // leaves the preview stuck on an error.
  let frame_width = frame_width.min(camera_width);
  let frame_height = frame_height.min(camera_height);
  let frame_x = frame_x.clamp(
    camera_x,
    (camera_x + camera_width - frame_width).max(camera_x),
  );
  let frame_y = frame_y.clamp(
    camera_y,
    (camera_y + camera_height - frame_height).max(camera_y),
  );

  let source_scale = f64::from(options.camera_width.max(1)) / camera_width.max(1.0);
  let source_x = (frame_x - camera_x) * source_scale;
  let source_y = (frame_y - camera_y) * source_scale;
  let source_width = frame_width * source_scale;
  let source_height = frame_height * source_scale;
  let crop_x = (source_x.round().max(0.0) as u32) & !1;
  let crop_y = (source_y.round().max(0.0) as u32) & !1;
  let crop_width = even(source_width)
    .min(options.camera_width.saturating_sub(crop_x) & !1)
    .max(2);
  let crop_height = even(source_height)
    .min(options.camera_height.saturating_sub(crop_y) & !1)
    .max(2);
  let frame_width = even(frame_width);
  let frame_height = even(frame_height);
  Ok(BakeGeometry {
    crop_height,
    crop_width,
    crop_x,
    crop_y,
    frame_height,
    frame_width,
    frame_x: frame_x.round() as i32,
    frame_y: frame_y.round() as i32,
    output_height,
    output_width,
    radius: (f64::from(frame_width.min(frame_height)) * options.overlay.radius_percent / 100.0)
      .round() as u32,
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use screenwide_scenes::CameraOverlaySettings;

  #[test]
  fn accepts_a_crop_fitted_in_a_rounded_region_preview() {
    let screen = (2_700.0, 924.0);
    let camera_preview = (954.0, 720.0);
    let frame_width = screen.0 * 0.25;
    let frame_height = frame_width * camera_preview.1 / camera_preview.0;

    let geometry = bake_geometry(BakedVideoExportOptions {
      camera_drop_shadow: false,
      camera_height: 1_328,
      camera_width: 1_760,
      overlay: CameraOverlaySettings {
        camera_width: frame_width,
        camera_x: screen.0 / 2.0,
        camera_y: screen.1 / 2.0,
        frame_height,
        frame_width,
        frame_x: (screen.0 - frame_width) / 2.0,
        frame_y: (screen.1 - frame_height) / 2.0,
        radius_percent: 8.0,
        follow_face: false,
      },
      screen_height: 924,
      screen_width: 2_700,
      video: VideoExportOptions {
        compression: 2,
        resolution_scale_percent: 200,
        source_scale_percent: 200,
      },
    })
    .unwrap();

    assert_eq!(
      (geometry.output_width, geometry.output_height),
      (2_700, 924)
    );
    assert!(geometry.crop_width <= 1_760);
    assert!(geometry.crop_height <= 1_328);
  }

  #[test]
  fn preserves_a_camera_frame_partly_outside_the_output() {
    let geometry = bake_geometry(BakedVideoExportOptions {
      camera_drop_shadow: true,
      camera_height: 1_080,
      camera_width: 1_920,
      overlay: CameraOverlaySettings {
        camera_width: 768.0,
        camera_x: 0.0,
        camera_y: 540.0,
        frame_height: 324.0,
        frame_width: 768.0,
        frame_x: -384.0,
        frame_y: 378.0,
        radius_percent: 10.0,
        follow_face: false,
      },
      screen_height: 1_080,
      screen_width: 1_920,
      video: VideoExportOptions {
        compression: 2,
        resolution_scale_percent: 100,
        source_scale_percent: 100,
      },
    })
    .unwrap();

    assert_eq!(geometry.frame_x, -384);
    assert_eq!(geometry.frame_y, 378);
  }
}
