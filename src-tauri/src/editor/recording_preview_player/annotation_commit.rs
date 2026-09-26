// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What React is told an edit on the picture did, with pinned annotations
//! handed back as they were drawn.

use super::gesture::Commit;
use super::*;
use crate::editor::annotations::pin::{fold, AnnotationPin};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CommitPin {
  pub(super) annotation_id: String,
  pub(super) pin: AnnotationPin,
}

impl PreviewPlayerManager {
  /// What React is to commit for the annotations `shown` on `pane` at
  /// `position`. A pinned annotation is shown where its pin puts it, and is
  /// handed back as it was drawn, with its pin: an edit that moved it whole
  /// is a keyframe, not a new place to have drawn it.
  pub(super) fn commit(
    &self,
    pane: u32,
    position: u64,
    shown: Vec<Annotation>,
    selected_annotation_id: Option<String>,
    text_edit: Option<crate::editor::annotations::text::edit::TextEditPhase>,
  ) -> Option<Commit> {
    let session_id = self.session_id?;
    let clips = self
      .sources
      .as_ref()
      .and_then(|sources| {
        sources
          .annotation_clips
          .read()
          .ok()
          .map(|clips| clips.clone())
      })
      .unwrap_or_default();
    let mut pins = Vec::new();
    let annotations = shown
      .into_iter()
      .map(|annotation| {
        let Some(clip) = clips
          .iter()
          .find(|clip| clip.annotation.id == annotation.id && clip.pin.is_some())
        else {
          return annotation;
        };
        let mut clip = clip.clone();
        fold(&mut clip, &annotation, position);
        if let Some(pin) = clip.pin {
          pins.push(CommitPin {
            annotation_id: annotation.id.clone(),
            pin,
          });
        }
        clip.annotation
      })
      .collect();
    Some(Commit {
      session_id,
      pane_index: pane,
      source_position_ms: position,
      annotations,
      selected_annotation_id,
      text_edit,
      pins,
    })
  }
}
