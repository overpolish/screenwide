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
use super::super::ScreenshotWorkspaceOutputSettings;
use super::annotation::{annotation_handles, new_arrow, source_point};
use super::annotation_target::{
  drag_handle, next_annotation_id, AnnotationGestureTarget, AnnotationHandle,
};
use super::state::PreviewManager;
use crate::screenshots::{Annotation, AnnotationPoint, AnnotationShape};

/// The list React is asked to commit when a gesture ends.
#[derive(Clone, Debug)]
pub(crate) struct AnnotationCommit {
  pub(crate) annotations: Vec<Annotation>,
  pub(crate) pane_index: u32,
  pub(crate) selected_annotation_id: Option<String>,
}

pub(super) struct AnnotationGestureOverride {
  /// Where in the working copy's list the edited arrow lives.
  index: usize,
  pane_index: u32,
  snapshot: ScreenshotWorkspaceOutputSettings,
  target: AnnotationGestureTarget,
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
      if let (Some(surface), Some(source), Some(annotations)) = (
        self.surface.as_ref(),
        self.annotation_source(pane_index),
        self.annotations_for(pane_index),
      ) {
        let selected_index = selected
          .and_then(|id| annotations.iter().position(|item| item.id == id))
          .map_or(-1, |index| i32::try_from(index).unwrap_or(-1));
        surface.set_annotations(
          &annotation_handles(annotations, source),
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
      self.output = Some(gesture.snapshot);
      self.present_annotation_gesture(gesture.pane_index, None);
      return None;
    }
    if matches!(phase, SelectionGesturePhase::Begin) {
      return self.begin_annotation_gesture(pane_index, target, x, y);
    }
    let gesture = self.annotation_gesture.as_ref()?;
    let (pane_index, index, target) = (gesture.pane_index, gesture.index, gesture.target);
    let source = self.annotation_source(pane_index)?;
    let point = source_point(x, y, source);
    let annotation = self
      .output
      .as_mut()?
      .items
      .get_mut(pane_index as usize)?
      .output
      .annotations
      .get_mut(index)?;
    let id = annotation.id.clone();
    match target {
      AnnotationGestureTarget::NewArrow => {
        // A straight arrow while it is being drawn: the control point stays
        // on the midpoint so the bend is something you add afterwards.
        let AnnotationShape::Arrow {
          start,
          control,
          end,
        } = &mut annotation.shape;
        *end = point;
        *control = AnnotationPoint {
          x: (start.x + point.x) / 2.0,
          y: (start.y + point.y) / 2.0,
        };
      }
      AnnotationGestureTarget::Existing { handle, .. } => drag_handle(annotation, handle, point),
      // A press on nothing never opens a gesture, so nothing can update it.
      AnnotationGestureTarget::None => return None,
    }
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
    self.output = Some(base.clone());
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
    if let AnnotationGestureTarget::Existing { handle, index } = target {
      let annotations = self.annotations_for(pane_index)?;
      let id = annotations.get(index)?.id.clone();
      if handle == AnnotationHandle::Body {
        // A press on the shaft only chooses the arrow. It commits straight
        // away so the selection survives React's next layout.
        self.annotation_gesture = None;
        self.present_annotation_gesture(pane_index, Some(id.as_str()));
        return self.commit_for(pane_index, Some(id));
      }
      self.annotation_gesture = Some(AnnotationGestureOverride {
        index,
        pane_index,
        snapshot: base,
        target,
      });
      self.present_annotation_gesture(pane_index, Some(id.as_str()));
      return None;
    }
    let annotations = &mut self
      .output
      .as_mut()?
      .items
      .get_mut(pane_index as usize)?
      .output
      .annotations;
    if annotations.len() >= crate::screenshots::MAX_ANNOTATIONS {
      return None;
    }
    let arrow = new_arrow(next_annotation_id(), point, point);
    let id = arrow.id.clone();
    annotations.push(arrow);
    let index = annotations.len() - 1;
    self.annotation_gesture = Some(AnnotationGestureOverride {
      index,
      pane_index,
      snapshot: base,
      target,
    });
    self.present_annotation_gesture(pane_index, Some(id.as_str()));
    None
  }
}
