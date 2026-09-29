// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! One live annotation edit. Workspaces own presentation and history; this
//! transaction owns only the annotations changed by a pointer gesture.

use std::time::Instant;

use super::freehand::hold::StrokeHold;
use super::gesture::{next_annotation_id, AnnotationDragOrigin, AnnotationGestureTarget};
use super::snap::{SnapModifiers, SnapRequest, SnapResult};
use super::{Annotation, AnnotationKind, AnnotationPoint, AnnotationStyle};

pub(crate) struct AnnotationEdit {
  before: Vec<Annotation>,
  index: usize,
  id: String,
  origin: AnnotationDragOrigin,
  target: AnnotationGestureTarget,
  /// A stroke fresh from the pen: its rest at the end, and what it was taken
  /// for. Such a stroke is never left chosen.
  stroke: Option<StrokeHold>,
}

impl AnnotationEdit {
  /// Selection-only presses are handled by the workspace, without opening an
  /// edit. `kind` is the shape the tool in hand draws - absent where the tool
  /// draws nothing, which declines a press that would have made one - and
  /// `angle` where a fresh counter's tail points. Only a
  /// [`AnnotationGestureTarget::New`] press reads either.
  ///
  /// `source_per_size` is source pixels per point of a style's size for this
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
    source_per_size: f64,
  ) -> Option<Self> {
    let index = match target {
      AnnotationGestureTarget::New => annotations.len(),
      AnnotationGestureTarget::Existing { index, .. } if index < annotations.len() => index,
      _ => return None,
    };
    let before = annotations.clone();
    if target == AnnotationGestureTarget::New {
      let id = next_annotation_id();
      annotations.push(kind?.new_annotation(id, point, defaults, angle, &before, source_per_size));
    }
    let annotation = &annotations[index];
    let mut origin = AnnotationDragOrigin::new(point, &annotation.shape);
    origin.source_per_size = source_per_size;
    Some(Self {
      before,
      index,
      id: annotation.id.clone(),
      origin,
      target,
      stroke: (target == AnnotationGestureTarget::New
        && annotation.shape.kind() == AnnotationKind::Draw)
        .then(|| StrokeHold::new(point, Instant::now())),
    })
  }

  pub(crate) fn selected_id(&self) -> &str {
    &self.id
  }

  /// The annotation this gesture leaves chosen. A stroke fresh from the pen is
  /// let go, so the next press draws another rather than picking it up and no
  /// box stands over the drawing; every other annotation stays in hand for its
  /// panel to dress.
  pub(crate) fn chosen_id(&self) -> Option<&str> {
    self.stroke.is_none().then_some(self.id.as_str())
  }

  /// Whether this gesture is drawing a fresh stroke, which a clock outside
  /// it asks [`Self::hold`] about until it ends.
  pub(crate) fn holds_stroke(&self) -> bool {
    self.stroke.is_some()
  }

  /// Source pixels per screen point for the sample about to be applied. Read
  /// each sample, since a pinch may zoom part way through a drag.
  pub(crate) fn set_source_per_point(&mut self, source_per_point: Option<f64>) {
    self.origin.source_per_point = source_per_point.unwrap_or(0.0);
  }

  /// The pixels a highlight selects from, for the samples still to come.
  /// Set as soon as the workspace has them, which for a recording may be
  /// part way through the drag; a picture once given is kept.
  pub(crate) fn set_picture(
    &mut self,
    picture: Option<std::sync::Arc<super::highlight::picture::HighlightPicture>>,
  ) {
    if picture.is_some() {
      self.origin.picture = picture;
    }
  }

  /// Whether the annotation this gesture holds reads a picture: only a
  /// highlight does, so no other gesture has one decoded for it.
  pub(crate) fn wants_picture(&self, annotations: &[Annotation]) -> bool {
    annotations
      .get(self.index)
      .filter(|annotation| annotation.id == self.id)
      .is_some_and(|annotation| annotation.shape.kind() == AnnotationKind::Highlight)
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
    &mut self,
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
        // A stroke still resting where it was taken for something cleaner
        // keeps that; moving on gives the stroke back to carry on.
        let unit = self.origin.source_per_point;
        if let Some(stroke) = &mut self.stroke {
          if !stroke.sample(annotation, point, unit, Instant::now()) {
            return SnapResult::default();
          }
        }
        annotation.drag_new(point, &self.origin, modifiers.shift, snap)
      }
      AnnotationGestureTarget::Existing { handle, .. } => {
        annotation.drag_grip(handle, point, &self.origin, modifiers.shift, snap)
      }
      _ => SnapResult::default(),
    }
  }

  /// Reads a fresh stroke the hand has rested on long enough, and puts what
  /// it was taken for in its place. Answers whether the annotation changed,
  /// and so wants presenting.
  pub(crate) fn hold(&mut self, annotations: &mut [Annotation], now: Instant) -> bool {
    let unit = self.origin.source_per_point;
    let (Some(stroke), Some(annotation)) = (
      &mut self.stroke,
      annotations
        .get_mut(self.index)
        .filter(|item| item.id == self.id),
    ) else {
      return false;
    };
    stroke.hold(annotation, unit, now)
  }

  /// Drop an edit to accept the working list, or restore it on Escape.
  pub(crate) fn cancel(self, annotations: &mut Vec<Annotation>) {
    *annotations = self.before;
  }
}
