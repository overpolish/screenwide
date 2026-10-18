// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The camera's effects on the retained scene. The scene keeps each camera
//! frame as decoded and the matte read beside it as it is staged, so a
//! change to the effects is a redraw.

use std::sync::Arc;

use super::WorkspaceScene;
use screenwide_export::camera_effects::live::LiveCameraEffects;

impl WorkspaceScene {
  /// Draws the camera with `live` from now on, the matte of every camera
  /// frame the scene holds read again for it.
  pub fn set_camera_effects(&self, live: Option<Arc<LiveCameraEffects>>) -> Result<(), String> {
    let mut state = self.state()?;
    let gpu = state.compositor.gpu();
    state.effects.set(live, gpu)?;
    let state = &mut *state;
    for layer in &mut state.layers {
      layer.source_matte = layer
        .source_is_camera
        .then(|| state.effects.matte_at(layer.seconds))
        .flatten();
      if let Some(camera) = layer.camera.as_mut() {
        camera.matte = state.effects.matte_at(layer.seconds);
      }
    }
    Ok(())
  }
}
