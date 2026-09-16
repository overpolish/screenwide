// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::render_test_helpers::{device, read_pixel, target, OUTPUT};
use super::*;
use crate::editor::preview_platform::ComposedFrame;
use crate::screenshots::{test_output_settings, CapturedImage, CropPreviewRect};

/// What the crop tool sends: the whole 800px-square source placed in the
/// canvas as a ghost, rounded by half of its shorter side and shadowed.
fn crop_tool_settings() -> ScreenshotOutputSettings {
  let mut settings = test_output_settings(OUTPUT.0, OUTPUT.1);
  settings.drop_shadow = true;
  settings.radius_percent = 50.0;
  settings.image_width = 800.0;
  settings.image_x = 400.0;
  settings.image_y = 200.0;
  settings.crop_width = 800.0;
  settings.crop_height = 800.0;
  settings.crop_x = 400.0;
  settings.crop_y = 200.0;
  settings
}

fn brightness(pixel: [u8; 4]) -> i32 {
  pixel[..3].iter().map(|channel| i32::from(*channel)).sum()
}

#[test]
#[ignore = "requires a Windows D3D11 hardware adapter"]
fn a_crop_preview_rounds_and_shadows_the_crop_rather_than_the_ghost() {
  let (device, context) = device();
  let compositor = Compositor::new(&device).expect("D3D11 preview compositor");
  let source = compositor
    .screenshot_source(
      &device,
      &CapturedImage {
        width: 64,
        height: 64,
        rgba: vec![255; 64 * 64 * 4],
      },
    )
    .expect("source texture");
  let ghost = crop_tool_settings();
  let mut cropped = ghost.clone();
  cropped.crop_preview = Some(CropPreviewRect {
    height: 400.0,
    width: 400.0,
    x: 600.0,
    y: 400.0,
  });
  let draw = |settings: &ScreenshotOutputSettings| {
    let texture = target(&device);
    compositor
      .draw_with_camera(
        &context,
        &texture,
        &source,
        settings,
        ComposedFrame {
          cursor: None,
          keyboard: None,
          foreground_only: false,
          seconds: 0.0,
        },
        None,
        None,
        &Default::default(),
      )
      .expect("preview draw");
    texture
  };
  let without_preview = draw(&ghost);
  let with_preview = draw(&cropped);

  // Six pixels in from the ghost's top left corner: well outside a 400px
  // rounding of the whole ghost, and inside the flat ghost that a crop
  // preview leaves behind because the rounding moved onto the crop.
  let background = brightness(read_pixel(&device, &context, &with_preview, 100, 100));
  let rounded_ghost = brightness(read_pixel(&device, &context, &without_preview, 406, 206));
  let flat_ghost = brightness(read_pixel(&device, &context, &with_preview, 406, 206));
  assert!(
    rounded_ghost < background + 16,
    "the ghost's corner should be background without a crop preview, was {rounded_ghost}"
  );
  assert!(
    flat_ghost > 750,
    "a crop preview should leave the ghost's corner opaque source, was {flat_ghost}"
  );

  // Ten pixels outside the ghost's left edge: the shadow travels with the
  // rounding, so the crop's own shadow never reaches out this far.
  let shadowed = brightness(read_pixel(&device, &context, &without_preview, 390, 600));
  let unshadowed = brightness(read_pixel(&device, &context, &with_preview, 390, 600));
  assert!(
    shadowed + 8 < background,
    "the ghost should cast the shadow without a crop preview, was {shadowed}"
  );
  assert!(
    (unshadowed - background).abs() <= 6,
    "a crop preview should leave the canvas beside the ghost unshadowed, was {unshadowed}"
  );
}
