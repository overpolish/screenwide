// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where the overlays drawn over the picture show, for the press that opens
//! their tools: the shortcut strip and the caption, each as x, y, width and
//! height shares of its canvas.

use super::WorkspaceScene;
use screenwide_compositor::annotation_gpu::{
  caption_editing_block, caption_measure, caption_visible_bounds, CaptionFaces,
};
use screenwide_text::captions::CaptionFrame;

impl WorkspaceScene {
  pub(super) fn keyboard_bounds(&self, pane: u32) -> Option<[f64; 4]> {
    let state = self.read().ok()?;
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
    let state = self.read().ok()?;
    state
      .layers
      .iter()
      .filter(|layer| layer.pane_index == pane)
      .find_map(|layer| {
        let (track, moment) = layer.captions.as_ref()?;
        caption_visible_bounds(track, *moment, layer.geometry.size)
      })
  }

  /// Where the caption being typed into on `pane` sets its words: its block
  /// as shares of its canvas, its type size as a share of the block's
  /// height, and how its lines line up.
  pub(super) fn caption_editing_block(&self, pane: u32) -> Option<([f64; 4], f64, f64)> {
    let state = self.read().ok()?;
    state
      .layers
      .iter()
      .filter(|layer| layer.pane_index == pane)
      .find_map(|layer| {
        let (track, _) = layer.captions.as_ref()?;
        caption_editing_block(track, layer.geometry.size)
      })
  }

  /// Which word of the caption showing on `pane` is under the point `at`,
  /// as shares of its canvas, counted across the caption's lines.
  pub(super) fn caption_word_at(&self, pane: u32, at: [f64; 2]) -> Option<usize> {
    let state = self.read().ok()?;
    state
      .layers
      .iter()
      .filter(|layer| layer.pane_index == pane)
      .find_map(|layer| {
        let (track, moment) = layer.captions.as_ref()?;
        let (width, height) = layer.geometry.size;
        let frame = CaptionFrame { width, height };
        let faces = CaptionFaces::of(&track.recording.style);
        let point = [at[0] * f64::from(width), at[1] * f64::from(height)];
        let mut measure = caption_measure(&faces);
        track.word_at(frame, *moment, point, &mut measure)
      })
  }
}
