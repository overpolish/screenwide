// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Native screenshot workspace presentation.
//!
//! Screenshot annotations enter through this retained GPU workspace, flattened
//! from each layer's settings so the preview draws what the export will;
//! React remains semantic state and command/event transport.

use super::ffi::screenwide_preview_surface_present_screenshot_workspace;
use super::RecordingPreviewSurface;
use screenwide_imaging::CapturedImage;
use screenwide_output::ScreenshotOutputSettings;

impl RecordingPreviewSurface {
  /// Composes every screenshot item into one workspace drawable. The scene
  /// retains each picture by its token, so a later pan/zoom redraw never
  /// requires React or another upload.
  #[allow(clippy::type_complexity)]
  pub(crate) fn present_screenshot_workspace(
    &self,
    layers: &[(u64, &CapturedImage, ScreenshotOutputSettings)],
    // The hovered arrow's halo: its layer, its place in that layer's list,
    // and the halo's width in canvas pixels. It never reaches the export.
    hover: Option<(u64, usize, f32)>,
  ) -> Result<bool, String> {
    self.scene.stage_screenshot(layers, hover)?;
    Ok(unsafe {
      screenwide_preview_surface_present_screenshot_workspace(
        self.handle,
        layers.len().try_into().unwrap_or(u32::MAX),
      ) != 0
    })
  }
}
