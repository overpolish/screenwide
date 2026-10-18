// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The overlays a native surface hands across as one C struct: the camera
//! bubble's placement and the cursor's size; and a still composed from a
//! frame and them.

use super::still::{compose_still, StillCamera, StillInputs};
use screenwide_cursor_effects::GpuCursor;
use screenwide_imaging::CapturedImage;
use screenwide_keyboard_effects::KeyboardOverlay;
use screenwide_output::ScreenshotOutputSettings;

#[repr(C)]
#[derive(Default)]
pub struct StillOverlay {
  pub cursor_x: i32,
  pub cursor_y: i32,
  pub cursor_width: u32,
  pub cursor_height: u32,
  pub cursor_source_width: u32,
  pub cursor_source_height: u32,
  pub camera_crop_x: u32,
  pub camera_crop_y: u32,
  pub camera_crop_width: u32,
  pub camera_crop_height: u32,
  pub camera_frame_x: i32,
  pub camera_frame_y: i32,
  pub camera_frame_width: u32,
  pub camera_frame_height: u32,
  pub camera_radius: u32,
  pub camera_source_width: u32,
  pub camera_source_height: u32,
  pub camera_drop_shadow: u32,
  pub camera_on_top: u32,
}

/// `image` composed on the canvas `settings` describe, through the shared
/// compositor. `overlay` carries the camera bubble's placement; `cursor` is
/// placed on the canvas.
#[allow(clippy::too_many_arguments)]
pub fn compose_output_layers(
  image: &CapturedImage,
  settings: &ScreenshotOutputSettings,
  seconds: f64,
  transparent_background: bool,
  cursor: Option<&GpuCursor>,
  camera: Option<&CapturedImage>,
  overlay: Option<&StillOverlay>,
  keyboard: Option<&KeyboardOverlay>,
  clip_cursor_at_video_edge: bool,
  foreground_only: bool,
) -> Result<CapturedImage, String> {
  screenwide_output::validate_output_settings(image.width, image.height, settings)?;
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
  compose_still(
    image,
    settings,
    StillInputs {
      seconds,
      cursor: cursor.map(|cursor| GpuCursor {
        clip_at_video_edge: clip_cursor_at_video_edge,
        ..*cursor
      }),
      camera,
      keyboard: keyboard.copied(),
      foreground_only,
      transparent_background,
    },
  )
}
