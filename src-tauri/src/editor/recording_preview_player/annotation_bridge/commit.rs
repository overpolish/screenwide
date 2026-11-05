// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What React is told an edit on the picture did, with pinned annotations
//! handed back as they were drawn and an annotation moved in a scene handed
//! back with its place there.

use super::gesture::Commit;
use super::*;
use screenwide_annotations::pin::AnnotationPin;
use screenwide_annotations::scene_shape::SceneEdit;
use screenwide_annotations::SceneShape;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CommitPin {
  pub(super) annotation_id: String,
  pub(super) pin: AnnotationPin,
}

/// An annotation's shapes in the scenes once an edit in one of them lands.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CommitSceneShapes {
  pub(super) annotation_id: String,
  pub(super) scene_shapes: Vec<SceneShape>,
}

impl PreviewPlayerManager {
  /// Where an edit made on the frame `position` into the recording lands:
  /// in the scene the playhead is in, if any.
  pub(super) fn scene_edit(&self, position: u64) -> SceneEdit {
    let Some(sources) = self.sources.as_ref() else {
      return SceneEdit::at(Vec::new(), &[], position, [(0, 0); 2]);
    };
    let ranges = sources
      .annotation_ranges
      .read()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
      .clone();
    SceneEdit::at(
      self.scene_clips(),
      &ranges,
      position,
      sources.annotation_pictures(),
    )
  }

  /// What React is to commit for the annotations `shown` on `pane` at
  /// `position`. A pinned annotation is shown where its pin puts it, and is
  /// handed back as it was drawn, with its pin: an edit that moved it whole
  /// is a keyframe, not a new place to have drawn it. Inside a scene an
  /// annotation is handed back as it is outside it, with its shapes in the
  /// scenes, the one it was moved in among them.
  pub(super) fn commit(
    &self,
    pane: u32,
    position: u64,
    shown: Vec<Annotation>,
    selected_annotation_ids: Vec<String>,
    text_edit: Option<screenwide_annotations::text::edit::TextEditPhase>,
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
    let scenes = self.scene_edit(position);
    let mut pins = Vec::new();
    let mut scene_shapes = Vec::new();
    let annotations = shown
      .into_iter()
      .map(|annotation| {
        let Some(clip) = clips
          .iter()
          .find(|clip| clip.annotation.id == annotation.id)
          .filter(|clip| clip.pin.is_some() || scenes.is_in_scene())
        else {
          return annotation;
        };
        let mut clip = clip.clone();
        scenes.fold(&mut clip, &annotation, position);
        if let Some(pin) = clip.pin {
          pins.push(CommitPin {
            annotation_id: annotation.id.clone(),
            pin,
          });
        } else {
          scene_shapes.push(CommitSceneShapes {
            annotation_id: annotation.id.clone(),
            scene_shapes: clip.scene_shapes,
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
      selected_annotation_ids,
      text_edit,
      pins,
      scene_shapes,
    })
  }
}
