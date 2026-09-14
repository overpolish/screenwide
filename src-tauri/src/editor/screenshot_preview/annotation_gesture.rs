// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The arrow tool's pointer gesture.
//!
//! The native interaction view reports where the pointer is, in the layer's
//! image-normalised space, and which grip it took hold of. Everything that
//! decides what an arrow *is* - the source-pixel geometry, the Bézier solve
//! behind the middle handle, the colour a fresh arrow is drawn in - happens
//! here, so the native side never carries a second copy of the model.
//!
//! The manager owns the picture for the length of the gesture, the way the
//! selection gesture does: it mutates a working copy of the layer's settings
//! and presents it natively on every sample, and only mouse-up hands the
//! finished list back to React to commit into the document and its history.

use super::super::preview_platform::SelectionGesturePhase;
use super::state::PreviewManager;
use crate::editor::annotations::edit::AnnotationEdit;
use crate::editor::annotations::gesture::AnnotationGestureTarget;
use crate::editor::annotations::handles::{annotation_handles, source_point};
use crate::editor::annotations::Annotation;

/// The list React is asked to commit when a gesture ends.
#[derive(Clone, Debug)]
pub(crate) struct AnnotationCommit {
  pub(crate) annotations: Vec<Annotation>,
  pub(crate) pane_index: u32,
  pub(crate) selected_annotation_id: Option<String>,
}

pub(super) struct AnnotationGestureOverride {
  pane_index: u32,
  edit: AnnotationEdit,
}

impl PreviewManager {
  /// The source's pixel dimensions for one pane, for the normalised points
  /// the native side reports.
  pub(super) fn annotation_source(&self, pane_index: u32) -> Option<(u32, u32)> {
    let output = self.output.as_ref()?;
    let item = output.items.get(pane_index as usize)?;
    let (_, source) = self.sources.iter().find(|(id, _)| *id == item.id)?;
    Some((source.width, source.height))
  }

  /// How wide this pane's picture is drawn, in output pixels: what turns a
  /// stroke in output pixels into a share of the picture.
  pub(super) fn annotation_image_width(&self, pane_index: u32) -> Option<f64> {
    Some(
      self
        .output
        .as_ref()?
        .items
        .get(pane_index as usize)?
        .output
        .image_width,
    )
  }

  pub(super) fn annotations_for(&self, pane_index: u32) -> Option<&Vec<Annotation>> {
    Some(
      &self
        .output
        .as_ref()?
        .items
        .get(pane_index as usize)?
        .output
        .annotations,
    )
  }

  /// Republishes the arrow grips and the live picture. Both are read from the
  /// manager's own working copy, so a gesture sample shows its own geometry
  /// rather than the React layout that is still catching up with it.
  fn present_annotation_gesture(&self, pane_index: u32, selected: Option<&str>) {
    #[cfg(target_os = "macos")]
    {
      if let (Some(surface), Some(source), Some(image_width), Some(annotations)) = (
        self.surface.as_ref(),
        self.annotation_source(pane_index),
        self.annotation_image_width(pane_index),
        self.annotations_for(pane_index),
      ) {
        let selected_index = selected
          .and_then(|id| annotations.iter().position(|item| item.id == id))
          .map_or(-1, |index| i32::try_from(index).unwrap_or(-1));
        surface.set_annotations(
          &annotation_handles(annotations, source, image_width),
          selected_index,
          self.annotation_mode,
        );
      }
    }
    #[cfg(not(target_os = "macos"))]
    let _ = (pane_index, selected);
    let _ = self.present_batch();
  }

  fn commit_for(&self, pane_index: u32, selected: Option<String>) -> Option<AnnotationCommit> {
    Some(AnnotationCommit {
      annotations: self.annotations_for(pane_index)?.clone(),
      pane_index,
      selected_annotation_id: selected,
    })
  }

  /// One pointer sample of the arrow tool. `x` and `y` are normalised over
  /// the whole source image. The returned commit, on the end of a gesture, is
  /// what React writes into the document.
  pub(crate) fn handle_annotation_gesture(
    &mut self,
    phase: SelectionGesturePhase,
    pane_index: u32,
    target: AnnotationGestureTarget,
    x: f64,
    y: f64,
  ) -> Option<AnnotationCommit> {
    if matches!(phase, SelectionGesturePhase::Cancel) {
      let gesture = self.annotation_gesture.take()?;
      if let Some(annotations) = self
        .output
        .as_mut()
        .and_then(|output| output.items.get_mut(gesture.pane_index as usize))
        .map(|item| &mut item.output.annotations)
      {
        gesture.edit.cancel(annotations);
      }
      self.present_annotation_gesture(gesture.pane_index, None);
      return None;
    }
    if matches!(phase, SelectionGesturePhase::Begin) {
      return self.begin_annotation_gesture(pane_index, target, x, y);
    }
    let gesture = self.annotation_gesture.as_ref()?;
    let pane_index = gesture.pane_index;
    let source = self.annotation_source(pane_index)?;
    let point = source_point(x, y, source);
    let annotations = &mut self
      .output
      .as_mut()?
      .items
      .get_mut(pane_index as usize)?
      .output
      .annotations;
    gesture.edit.update(annotations, point);
    let id = gesture.edit.selected_id().to_owned();
    if matches!(phase, SelectionGesturePhase::End) {
      self.annotation_gesture = None;
      self.present_annotation_gesture(pane_index, Some(id.as_str()));
      return self.commit_for(pane_index, Some(id));
    }
    self.present_annotation_gesture(pane_index, Some(id.as_str()));
    None
  }

  fn begin_annotation_gesture(
    &mut self,
    pane_index: u32,
    target: AnnotationGestureTarget,
    x: f64,
    y: f64,
  ) -> Option<AnnotationCommit> {
    // The OSC is derived from React's latest layout: rebase the native pixel
    // composition onto that same snapshot before accepting pointer input so
    // both Metal layers share one gesture origin.
    let base = self.react_output.clone().or_else(|| self.output.clone())?;
    self.output = Some(base);
    let source = self.annotation_source(pane_index)?;
    let point = source_point(x, y, source);
    if matches!(target, AnnotationGestureTarget::None) {
      // Nothing under the press: let the arrow go and leave the rest of the
      // gesture to the layer. The list is unchanged, so React reads this as a
      // selection rather than an edit.
      self.annotation_gesture = None;
      self.annotation_hover = None;
      self.present_annotation_gesture(pane_index, None);
      return self.commit_for(pane_index, None);
    }
    if let AnnotationGestureTarget::Select { index } = target {
      // A press on the shaft only chooses the arrow. It commits straight
      // away so the selection survives React's next layout; the move it may
      // turn into arrives as an `Existing` gesture of its own.
      let id = self.annotations_for(pane_index)?.get(index)?.id.clone();
      self.annotation_gesture = None;
      self.present_annotation_gesture(pane_index, Some(id.as_str()));
      return self.commit_for(pane_index, Some(id));
    }
    let defaults = self.annotation_defaults.clone();
    let annotations = &mut self
      .output
      .as_mut()?
      .items
      .get_mut(pane_index as usize)?
      .output
      .annotations;
    let edit = AnnotationEdit::begin(annotations, target, point, defaults.as_ref())?;
    let id = edit.selected_id().to_owned();
    self.annotation_gesture = Some(AnnotationGestureOverride { pane_index, edit });
    self.present_annotation_gesture(pane_index, Some(id.as_str()));
    None
  }
}
