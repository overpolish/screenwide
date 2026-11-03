// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The overlays a native surface hands across as one C struct: the camera
//! bubble's placement and the cursor's size; and a still composed from a
//! frame and them.

use super::still::{compose_still, StillCamera, StillInputs};
use super::PictureLook;
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

impl StillOverlay {
  /// `camera` placed as this overlay places the camera bubble, drawn as
  /// `look` says the camera's effects left it; none where it has no frame.
  pub fn still_camera<'a>(
    &self,
    camera: &'a CapturedImage,
    look: PictureLook,
  ) -> Option<StillCamera<'a>> {
    (self.camera_frame_width > 0).then_some(StillCamera {
      image: camera,
      crop: [
        self.camera_crop_x,
        self.camera_crop_y,
        self.camera_crop_width,
        self.camera_crop_height,
      ],
      frame: (
        self.camera_frame_x,
        self.camera_frame_y,
        self.camera_frame_width,
        self.camera_frame_height,
      ),
      radius: self.camera_radius,
      drop_shadow: self.camera_drop_shadow != 0,
      on_top: self.camera_on_top != 0,
      look,
    })
  }
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
    (Some(camera), Some(overlay)) => overlay.still_camera(camera, PictureLook::default()),
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
      look: PictureLook::default(),
      transparent_background,
    },
  )
}
