// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where the overlays drawn over the picture show, for the press that opens
//! their tools: the shortcut strip and the caption, each as x, y, width and
//! height shares of its canvas.

use super::WorkspaceScene;
use screenwide_compositor::annotation_gpu::caption_visible_bounds;

impl WorkspaceScene {
  pub(super) fn keyboard_bounds(&self, pane: u32) -> Option<[f64; 4]> {
    let state = self.state().ok()?;
    let layer = state.layers.iter().find(|layer| {
      layer.pane_index == pane
        && layer
          .keyboard
          .is_some_and(|keyboard| keyboard.key_count > 0)
        && layer.geometry.size.0 > 0
        && layer.geometry.size.1 > 0
    })?;
    state
      .compositor
      .keyboard_visible_bounds(layer.keyboard.as_ref()?, layer.geometry.size)
      .ok()
      .flatten()
  }

  pub(super) fn caption_bounds(&self, pane: u32) -> Option<[f64; 4]> {
    let state = self.state().ok()?;
    state
      .layers
      .iter()
      .filter(|layer| layer.pane_index == pane)
      .find_map(|layer| {
        let (track, moment) = layer.captions.as_ref()?;
        caption_visible_bounds(track, *moment, layer.geometry.size)
      })
  }
}
