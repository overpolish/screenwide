// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The camera frame an exported One video frame draws.

use cidre::cv;

use crate::editor::preview_platform::compositor::{CameraComposition, Compositor, SourceTexture};
use crate::gpu::macos::bgra_buffer_texture;
use crate::screenshots::ScreenshotOutputSettings;

/// The decoded camera frame `pixels` as the canvas samples it. The
/// redactions and highlights drawn on the camera change its own pixels, so
/// where `composed` carries any - the camera's own settings holding them,
/// and its source size - they are drawn into the frame first.
pub(super) fn drawn_camera(
  compositor: &Compositor,
  pixels: &cv::PixelBuf,
  composed: Option<&(ScreenshotOutputSettings, (u32, u32))>,
) -> Result<SourceTexture, String> {
  let texture = bgra_buffer_texture(compositor.gpu(), pixels, "export camera")?;
  let raw = SourceTexture {
    size: (texture.width(), texture.height()),
    view: texture.create_view(&Default::default()),
    texture,
    picture: None,
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
