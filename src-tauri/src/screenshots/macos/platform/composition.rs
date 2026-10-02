// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// `image` composed on the canvas `settings` describe, through the shared
/// compositor. `overlay` carries the camera bubble's placement; the cursor is
/// drawn from its artwork.
#[allow(clippy::too_many_arguments)]
pub(crate) fn compose_output_layers(
  image: &CapturedImage,
  settings: &ScreenshotOutputSettings,
  seconds: f64,
  transparent_background: bool,
  cursor: Option<(&GpuCursor, &[GpuArtwork])>,
  camera: Option<&CapturedImage>,
  overlay: Option<&StillOverlay>,
  keyboard: Option<&KeyboardOverlay>,
  clip_cursor_at_video_edge: bool,
  foreground_only: bool,
) -> Result<CapturedImage, String> {
  super::super::super::validate_output_settings(image.width, image.height, settings)?;
  let camera = match (camera, overlay) {
    (Some(camera), Some(overlay)) if overlay.camera_frame_width > 0 => Some(StillCamera {
      image: camera,
      crop: [
        overlay.camera_crop_x,
        overlay.camera_crop_y,
        overlay.camera_crop_width,
        overlay.camera_crop_height,
      ],
      frame: (
        overlay.camera_frame_x,
        overlay.camera_frame_y,
        overlay.camera_frame_width,
        overlay.camera_frame_height,
      ),
      radius: overlay.camera_radius,
      drop_shadow: overlay.camera_drop_shadow != 0,
      on_top: overlay.camera_on_top != 0,
    }),
    _ => None,
  };
  let artworks = cursor
    .map_or(&[][..], |(_, artworks)| artworks)
    .iter()
    .map(CursorArtwork::from)
    .collect::<Vec<_>>();
  compose_still(
    image,
    settings,
    StillInputs {
      seconds,
      cursor: cursor.map(|(cursor, _)| {
        (
          GpuCursor {
            clip_at_video_edge: clip_cursor_at_video_edge,
            ..*cursor
          },
          artworks.as_slice(),
        )
      }),
      camera,
      keyboard: keyboard.copied(),
      foreground_only,
      transparent_background,
    },
  )
}

pub(crate) fn alpha_composite(
  base: &CapturedImage,
  overlay: &CapturedImage,
) -> Result<CapturedImage, String> {
  crate::editor::preview_platform::compositor::still::alpha_composite(base, overlay)
}
