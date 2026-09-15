// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl PreviewPlayerManager {
  pub(super) fn annotation_targets(&self) -> Vec<(u32, Annotation)> {
    let Some(sources) = &self.sources else {
      return Vec::new();
    };
    (0..sources.playback_layout.panes.len().min(2) as u32)
      .flat_map(|pane| {
        self
          .annotation_marks(pane)
          .into_iter()
          .map(move |mark| (pane, mark))
      })
      .collect()
  }

  pub(in crate::editor::recording_preview_player) fn publish_annotation_handles(&self) {
    let Some(sources) = &self.sources else {
      return;
    };
    let Some(surface) = &sources.preview_surface else {
      return;
    };
    let Some(composition) = self.selection_composition() else {
      return;
    };
    let mut handles = Vec::new();
    let mut selected = if self.annotation.selected.is_some() {
      -2
    } else {
      -1
    };
    for pane in 0..sources.playback_layout.panes.len().min(2) {
      let source = &sources.playback_layout.panes[pane];
      let output = if pane == 1 {
        &composition.recording_output.camera
      } else {
        &composition.recording_output.primary
      };
      let marks = self.annotation_marks(pane as u32);
      if let Some(index) = marks
        .iter()
        .position(|mark| Some(&mark.id) == self.annotation.selected.as_ref())
      {
        selected = (handles.len() + index) as i32;
      }
      handles.extend(
        annotation_handles(
          &marks,
          (source.source_width, source.source_height),
          output.image_width,
        )
        .into_iter()
        .map(|mut handle| {
          handle.layer_id = pane as i32;
          handle
        }),
      );
    }
    surface.set_annotation_layer(
      &handles,
      selected,
      if self.is_playing {
        0
      } else {
        self.annotation.mode
      },
      self.annotation.pane.map_or(-1, |pane| pane as i32),
    );
  }
}
