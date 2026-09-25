// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! One live annotation edit. Workspaces own presentation and history; this
//! transaction owns only the annotations changed by a pointer gesture.

use super::gesture::{next_annotation_id, AnnotationDragOrigin, AnnotationGestureTarget};
use super::snap::{SnapModifiers, SnapRequest, SnapResult};
use super::{Annotation, AnnotationKind, AnnotationPoint, AnnotationStyle};

pub(crate) struct AnnotationEdit {
  before: Vec<Annotation>,
  index: usize,
  id: String,
  origin: AnnotationDragOrigin,
  target: AnnotationGestureTarget,
}

impl AnnotationEdit {
  /// Selection-only presses are handled by the workspace, without opening an
  /// edit. `kind` is the shape the tool in hand draws - absent where the tool
  /// draws nothing, which declines a press that would have made one - and
  /// `angle` where a fresh counter's tail points. Only a
  /// [`AnnotationGestureTarget::New`] press reads either.
  ///
  /// `source_per_output` is source pixels per output pixel for this
  /// gesture's pane, zero where it is unknown. A fresh text box needs it to
  /// centre itself on the press, and a text box's pointer to tell when its
  /// tip is inside the box.
  pub(crate) fn begin(
    annotations: &mut Vec<Annotation>,
    target: AnnotationGestureTarget,
    point: AnnotationPoint,
    defaults: Option<&AnnotationStyle>,
    kind: Option<AnnotationKind>,
    angle: Option<f64>,
    source_per_output: f64,
  ) -> Option<Self> {
    let index = match target {
      AnnotationGestureTarget::New => annotations.len(),
      AnnotationGestureTarget::Existing { index, .. } if index < annotations.len() => index,
      _ => return None,
    };
    let before = annotations.clone();
    if target == AnnotationGestureTarget::New {
      let id = next_annotation_id();
      annotations.push(kind?.new_annotation(
        id,
        point,
        defaults,
        angle,
        &before,
        source_per_output,
      ));
    }
    let annotation = &annotations[index];
    let mut origin = AnnotationDragOrigin::new(point, &annotation.shape);
    origin.source_per_output = source_per_output;
    Some(Self {
      before,
      index,
      id: annotation.id.clone(),
      origin,
      target,
    })
  }

  pub(crate) fn selected_id(&self) -> &str {
    &self.id
  }

  /// Source pixels per screen point for the sample about to be applied. Read
  /// each sample, since a pinch may zoom part way through a drag.
  pub(crate) fn set_source_per_point(&mut self, source_per_point: Option<f64>) {
    self.origin.source_per_point = source_per_point.unwrap_or(0.0);
  }

  /// Whether this gesture made a fresh annotation rather than moving one.
  pub(crate) fn is_new(&self) -> bool {
    self.target == AnnotationGestureTarget::New
  }

  /// Samples are applied against the original shape, so returning the pointer
  /// to its starting point also returns the annotation there without drift.
  ///
  /// `modifiers` is what was held for this sample and `field` the candidates
  /// the gesture began with. This is the one place that decides whether a
  /// sample snaps at all: with the positional modifier let go, or with no
  /// usable reach to convert, the candidates are simply not offered.
  pub(crate) fn update(
    &self,
    annotations: &mut [Annotation],
    point: AnnotationPoint,
    modifiers: SnapModifiers,
    field: Option<SnapRequest<'_>>,
  ) -> SnapResult {
    let Some(annotation) = annotations
      .get_mut(self.index)
      .filter(|item| item.id == self.id)
    else {
      return SnapResult::default();
    };
    let snap = field.filter(|request| {
      modifiers.position && request.threshold.is_finite() && request.threshold > 0.0
    });
    match self.target {
      // A fresh arrow is drawn out from where the press landed; a fresh
      // counter or text box was dropped whole there, so the same drag
      // carries it. Each snaps the way the same grip does on an annotation
      // already placed.
      AnnotationGestureTarget::New => {
        annotation.drag_new(point, &self.origin, modifiers.shift, snap)
      }
      AnnotationGestureTarget::Existing { handle, .. } => {
        annotation.drag_grip(handle, point, &self.origin, modifiers.shift, snap)
      }
      _ => SnapResult::default(),
    }
  }

  /// Drop an edit to accept the working list, or restore it on Escape.
  pub(crate) fn cancel(self, annotations: &mut Vec<Annotation>) {
    *annotations = self.before;
  }
}
