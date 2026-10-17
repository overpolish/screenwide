// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Native screenshot workspace presentation, through the shared scene as on
//! macOS. Each item keeps a pane of its own for selection and hit-testing.

use super::*;

impl RecordingPreviewSurface {
  /// Composes every screenshot item into the workspace drawable. The scene
  /// retains each picture by its token, so a later pan/zoom redraw never
  /// requires React or another upload. `Ok(false)` means layout has not made
  /// the items' panes yet.
  #[allow(clippy::type_complexity)]
  pub(crate) fn present_screenshot_workspace(
    &self,
    layers: &[(u64, &CapturedImage, ScreenshotOutputSettings)],
    // The hovered arrow's halo: its layer, its place in that layer's list,
    // and the halo's width in canvas pixels. It never reaches the export.
    hover: Option<(u64, usize, f32)>,
  ) -> Result<bool, String> {
    let Ok(mut state) = self.inner.state.lock() else {
      return Ok(false);
    };
    if (0..layers.len()).any(|index| state.panes.get(index).is_none_or(Option::is_none)) {
      return Ok(false);
    }
    self
      .inner
      .gpu
      .workspace
      .scene
      .stage_screenshot(layers, hover)?;
    for (pane, (source_token, _, settings)) in state.panes.iter_mut().flatten().zip(layers) {
      let canvas = (settings.width, settings.height);
      if pane.content_size != canvas {
        pane.content_size = canvas;
        pane.selection_stale = true;
      }
      pane.settings = Some(settings.clone());
      pane.source_token = Some(*source_token);
      pane.captions = None;
    }
    state.workspace.panes = (0..layers.len()).collect();
    state.workspace.recording = false;
    present_workspace(&self.inner, &mut state);
    redraw_stale_selection(&self.inner, &mut state);
    Ok(true)
  }
}
