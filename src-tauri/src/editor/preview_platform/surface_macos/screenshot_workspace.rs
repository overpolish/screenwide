// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Native screenshot workspace presentation.
//!
//! Screenshot annotations enter through this retained GPU workspace, flattened
//! from each layer's settings so the preview draws what the export will;
//! React remains semantic state and command/event transport.

use super::ffi::screenwide_preview_surface_present_screenshot_workspace;
use super::workspace_scene::{LayerPicture, StagedLayer};
use super::RecordingPreviewSurface;
use crate::screenshots::{CapturedImage, ScreenshotOutputSettings};

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
    let staged = layers
      .iter()
      .enumerate()
      .map(|(index, (source_token, source, settings))| StagedLayer {
        pane_index: 0,
        // Screenshot selection and gesture events address layers by their
        // workspace order. `source_token` remains the independent cache key;
        // using it as the layer identity prevents the crop magnifier from
        // resolving the selected retained source.
        layer_id: u32::try_from(index).unwrap_or(u32::MAX - 1),
        source_token: *source_token,
        source: LayerPicture::Image(source),
        redaction_picture: Some(source),
        settings,
        seconds: 0.0,
        cursor: None,
        keyboard: None,
        camera: None,
        overlay: None,
        camera_settings: None,
        foreground_only: index > 0,
        hover: hover
          .filter(|(layer, _, _)| layer == source_token)
          .map(|(_, index, width)| (index, width)),
      })
      .collect::<Vec<_>>();
    self.scene.stage(&staged, None)?;
    Ok(unsafe {
      screenwide_preview_surface_present_screenshot_workspace(
        self.handle,
        staged.len().try_into().unwrap_or(u32::MAX),
      ) != 0
    })
  }
}
