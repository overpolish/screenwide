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
use crate::editor::annotations::gesture::{drawing_kind, AnnotationGestureTarget};
use crate::editor::annotations::handles::{annotation_handles, annotation_snap, source_point};
use crate::editor::annotations::snap::{
  detect_anchors, request_anchors, threshold_source_px, AnchorBoxes, SnapField, SnapModifiers,
  SnapRequest, SnapResult,
};
use crate::editor::annotations::Annotation;
use std::sync::Arc;

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
  /// What this gesture can snap to, built from the pane as it was when the
  /// press landed.
  field: SnapField,
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
    #[cfg(any(target_os = "macos", target_os = "windows"))]
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
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
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

  /// The elements detected in this pane's image, for an arrow's tip to land
  /// on. The first call starts the detection and answers `None`; the pointer
  /// is never held for it, and every later sample reads the cached result.
  fn annotation_anchors(&self, pane_index: u32, source: (u32, u32)) -> Option<Arc<AnchorBoxes>> {
    let key = (self.session_id?, pane_index, 0);
    if let Some(anchors) = self.annotation_anchor_cache.anchors(key) {
      return anchors.matches(source).then_some(anchors);
    }
    let item_id = self.output.as_ref()?.items.get(pane_index as usize)?.id;
    let image = self
      .sources
      .iter()
      .find(|(id, _)| *id == item_id)
      .map(|(_, image)| Arc::clone(image))?;
    request_anchors(&self.annotation_anchor_cache, key, move || {
      detect_anchors(&image.rgba, image.width, image.height)
    });
    None
  }

  /// Publishes what the sample on screen snapped to. It goes out before the
  /// grips, because on Windows publishing those is what redraws the chrome.
  fn publish_annotation_snap(&self, pane_index: u32, result: &SnapResult) {
    if let (Some(surface), Some(source)) =
      (self.surface.as_ref(), self.annotation_source(pane_index))
    {
      surface.set_annotation_snap_guides(annotation_snap(result, source));
    }
  }

  /// One pointer sample of the arrow tool. `x` and `y` are normalised over
  /// the whole source image. The returned commit, on the end of a gesture, is
  /// what React writes into the document.
  #[allow(clippy::too_many_arguments)]
  pub(crate) fn handle_annotation_gesture(
    &mut self,
    phase: SelectionGesturePhase,
    pane_index: u32,
    target: AnnotationGestureTarget,
    x: f64,
    y: f64,
    snap: u32,
    image_points: f64,
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
      self.publish_annotation_snap(gesture.pane_index, &SnapResult::default());
      self.present_annotation_gesture(gesture.pane_index, None);
      return None;
    }
    let modifiers = SnapModifiers::from_bits(snap);
    if matches!(phase, SelectionGesturePhase::Begin) {
      return self.begin_annotation_gesture(pane_index, target, x, y, modifiers);
    }
    let pane_index = self.annotation_gesture.as_ref()?.pane_index;
    let source = self.annotation_source(pane_index)?;
    let point = source_point(x, y, source);
    let threshold = threshold_source_px(source.0, image_points);
    let anchors = modifiers
      .position
      .then(|| self.annotation_anchors(pane_index, source))
      .flatten();
    let gesture = self.annotation_gesture.as_mut()?;
    gesture.field.anchors = anchors;
    let annotations = &mut self
      .output
      .as_mut()?
      .items
      .get_mut(pane_index as usize)?
      .output
      .annotations;
    let gesture = self.annotation_gesture.as_ref()?;
    let request = threshold.map(|threshold| SnapRequest {
      field: &gesture.field,
      threshold,
    });
    let result = gesture.edit.update(annotations, point, modifiers, request);
    let id = gesture.edit.selected_id().to_owned();
    let ended = matches!(phase, SelectionGesturePhase::End);
    // A gesture that has ended leaves nothing on screen to explain.
    let shown = if ended { SnapResult::default() } else { result };
    self.publish_annotation_snap(pane_index, &shown);
    if ended {
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
    modifiers: SnapModifiers,
  ) -> Option<AnnotationCommit> {
    // The OSC is derived from React's latest layout: rebase the native pixel
    // composition onto that same snapshot before accepting pointer input so
    // both Metal layers share one gesture origin.
    let base = self.react_output.clone().or_else(|| self.output.clone())?;
    self.output = Some(base);
    let source = self.annotation_source(pane_index)?;
    let point = source_point(x, y, source);
    self.publish_annotation_snap(pane_index, &SnapResult::default());
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
    let mode = self.annotation_mode;
    let angle = self.annotation_counter_angle;
    let image_width = self.annotation_image_width(pane_index).unwrap_or_default();
    let annotations = &mut self
      .output
      .as_mut()?
      .items
      .get_mut(pane_index as usize)?
      .output
      .annotations;
    let edit = AnnotationEdit::begin(
      annotations,
      target,
      point,
      defaults.as_ref(),
      drawing_kind(mode),
      angle,
    )?;
    // The field excludes the annotation the gesture holds, so a counter can
    // never snap back to the place it started from.
    let field = SnapField::new(source, annotations, edit.selected_id(), image_width);
    let id = edit.selected_id().to_owned();
    self.annotation_gesture = Some(AnnotationGestureOverride {
      pane_index,
      edit,
      field,
    });
    // The detector costs far too much to run between two pointer samples, so
    // the press starts it while the hand is still holding steady.
    if modifiers.position {
      let _ = self.annotation_anchors(pane_index, source);
    }
    self.present_annotation_gesture(pane_index, Some(id.as_str()));
    None
  }
}
