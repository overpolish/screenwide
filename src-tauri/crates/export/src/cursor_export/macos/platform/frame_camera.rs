// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The camera frame an exported One video frame draws.

use cidre::cv;

use crate::cursor_export::camera_effects::FrameEffects;
use screenwide_compositor::compositor::{CameraComposition, Compositor, SourceTexture};
use screenwide_gpu::macos::bgra_buffer_texture;
use screenwide_output::ScreenshotOutputSettings;

/// The decoded camera frame `pixels`, at `ms` into the camera, as the canvas
/// samples it. The camera's effects are drawn on it first where it has any.
/// The redactions and highlights drawn on the camera change its own pixels,
/// so where `composed` carries any - the camera's own settings holding them,
/// and its source size - they are drawn into the frame after.
pub(super) fn drawn_camera(
  compositor: &Compositor,
  pixels: &cv::PixelBuf,
  composed: Option<&(ScreenshotOutputSettings, (u32, u32))>,
  (effects, ms): (Option<&mut FrameEffects>, u64),
) -> Result<SourceTexture, String> {
  let texture = bgra_buffer_texture(compositor.gpu(), pixels, "export camera")?;
  let raw = SourceTexture {
    size: (texture.width(), texture.height()),
    view: texture.create_view(&Default::default()),
    texture,
    picture: None,
    enhanced: false,
    cut_out: false,
  };
  let raw = match effects {
    Some(effects) => effects.drawn(compositor, &raw, ms)?,
    None => raw,
  };
  let composition = composed
    .map(|(camera, source)| CameraComposition::new(camera, *source, raw.size))
    .transpose()?
    .flatten();
  match composition {
    Some(composition) => compositor.composed_camera(&raw, &composition, None, 0, false),
    None => Ok(raw),
  }
}
