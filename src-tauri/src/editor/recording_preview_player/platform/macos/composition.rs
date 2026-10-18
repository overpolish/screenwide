// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use screenwide_compositor::compositor::cursor_placement::placed_cursor;
use screenwide_compositor::compositor::still_overlay::StillOverlay;
use screenwide_cursor_effects::{CursorCompositor, CursorEffectSettings, GpuCursor};
use screenwide_imaging::CapturedImage;
use screenwide_output::ScreenshotOutputSettings;
use screenwide_scenes::CameraOverlaySettings;
use screenwide_workspace::scene::baked_camera_overlay;

/// The cursor `cursor` evaluates at its moment, placed on the canvas `output`
/// lays the decoded `screen` on, and the baked camera's place there where
/// `camera_overlay` asks for one.
pub(super) fn gpu_still_overlay(
  screen: &CapturedImage,
  output: &ScreenshotOutputSettings,
  cursor: Option<(&CursorCompositor, u64, CursorEffectSettings)>,
  camera: Option<&CapturedImage>,
  camera_overlay: Option<CameraOverlaySettings>,
  camera_drop_shadow: bool,
  camera_on_top: bool,
) -> Result<(Option<GpuCursor>, Option<StillOverlay>), String> {
  let cursor = match cursor.filter(|(_, _, settings)| settings.bake) {
    Some((cursor, position_ms, settings)) => placed_cursor(
      cursor,
      position_ms,
      settings,
      (screen.width, screen.height),
      output,
    )?,
    None => None,
  };
  let overlay = camera_overlay
    .map(|settings| {
      let camera = camera.ok_or_else(|| "Camera pixels are missing from the preview".to_owned())?;
      baked_camera_overlay(
        (camera.width, camera.height),
        output,
        settings,
        camera_drop_shadow,
        camera_on_top,
      )
    })
    .transpose()?;
  Ok((cursor, overlay))
}

pub(super) fn encoded_jpeg(image: &CapturedImage) -> Result<Vec<u8>, String> {
  let rgba = image::RgbaImage::from_raw(image.width, image.height, image.rgba.clone())
    .ok_or_else(|| "The native preview compositor returned invalid pixels".to_owned())?;
  let mut bytes = Vec::new();
  image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, 85)
    .encode_image(&rgba)
    .map_err(|error| error.to_string())?;
  Ok(bytes)
}
