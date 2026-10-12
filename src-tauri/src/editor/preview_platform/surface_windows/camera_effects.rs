// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The camera's effects as the Windows preview draws them. Every pane keeps
//! its decoded frame, so a change to the effects is a redraw.

use super::*;

impl RecordingPreviewSurface {
  /// Draws the camera with `live` from now on, and draws every pane again
  /// from the frame it holds.
  pub(crate) fn set_camera_effects(
    &self,
    live: Option<std::sync::Arc<screenwide_export::camera_effects::live::LiveCameraEffects>>,
  ) -> Result<(), String> {
    self
      .inner
      .camera_effects
      .lock()
      .map_err(|_| "The camera's effects are poisoned".to_owned())?
      .set(live, self.inner.gpu.shared)?;
    if let Ok(mut state) = self.inner.state.lock() {
      redraw_composed_panes(&self.inner, &mut state);
    }
    Ok(())
  }
}
