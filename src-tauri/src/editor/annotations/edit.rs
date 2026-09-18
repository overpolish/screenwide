// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! One live annotation edit. Workspaces own presentation and history; this
//! transaction owns only the annotations changed by a pointer gesture.

use super::counter::{new_counter, next_counter_value};
use super::gesture::{
  drag_handle, next_annotation_id, AnnotationDragOrigin, AnnotationGestureTarget, NewAnnotationKind,
};
use super::model::new_arrow;
use super::{Annotation, AnnotationPoint, AnnotationShape, AnnotationStyle, MAX_ANNOTATIONS};

pub(crate) struct AnnotationEdit {
  before: Vec<Annotation>,
  index: usize,
  id: String,
  origin: AnnotationDragOrigin,
  target: AnnotationGestureTarget,
}

impl AnnotationEdit {
  /// Selection-only presses are handled by the workspace, without opening an
  /// edit. `kind` is the shape the tool in hand draws and `angle` where a
  /// fresh counter's tail points, both of which only a
  /// [`AnnotationGestureTarget::New`] press reads.
  pub(crate) fn begin(
    annotations: &mut Vec<Annotation>,
    target: AnnotationGestureTarget,
    point: AnnotationPoint,
    defaults: Option<&AnnotationStyle>,
    kind: NewAnnotationKind,
    angle: Option<f64>,
  ) -> Option<Self> {
    let index = match target {
      AnnotationGestureTarget::New if annotations.len() < MAX_ANNOTATIONS => annotations.len(),
      AnnotationGestureTarget::Existing { index, .. } if index < annotations.len() => index,
      _ => return None,
    };
    let before = annotations.clone();
    if target == AnnotationGestureTarget::New {
      let id = next_annotation_id();
      annotations.push(match kind {
        NewAnnotationKind::Arrow => new_arrow(id, point, point, defaults),
        NewAnnotationKind::Counter => {
          new_counter(id, point, next_counter_value(&before), defaults, angle)
        }
      });
    }
    let annotation = &annotations[index];
    Some(Self {
      before,
      index,
      id: annotation.id.clone(),
      origin: AnnotationDragOrigin::new(point, &annotation.shape),
      target,
    })
  }

  pub(crate) fn selected_id(&self) -> &str {
    &self.id
  }

  /// Samples are applied against the original shape, so returning the pointer
  /// to its starting point also returns the annotation there without drift.
  /// `snap` is whether this sample was taken with Shift held.
  pub(crate) fn update(&self, annotations: &mut [Annotation], point: AnnotationPoint, snap: bool) {
    let Some(annotation) = annotations
      .get_mut(self.index)
      .filter(|item| item.id == self.id)
    else {
      return;
    };
    match self.target {
      // A fresh arrow is drawn out from where the press landed; a fresh
      // counter was dropped whole there, so the same drag carries it.
      AnnotationGestureTarget::New => match &mut annotation.shape {
        AnnotationShape::Arrow {
          start,
          control,
          end,
        } => {
          *end = point;
          *control = AnnotationPoint {
            x: (start.x + point.x) / 2.0,
            y: (start.y + point.y) / 2.0,
          };
        }
        AnnotationShape::Counter { center, .. } => *center = point,
      },
      AnnotationGestureTarget::Existing { handle, .. } => {
        drag_handle(annotation, handle, point, &self.origin, snap)
      }
      _ => {}
    }
  }

  /// Drop an edit to accept the working list, or restore it on Escape.
  pub(crate) fn cancel(self, annotations: &mut Vec<Annotation>) {
    *annotations = self.before;
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::editor::annotations::gesture::AnnotationHandle;

  fn point(x: f64, y: f64) -> AnnotationPoint {
    AnnotationPoint { x, y }
  }

  #[test]
  fn cancelling_a_new_arrow_restores_the_original_list() {
    let original = vec![new_arrow(
      "existing".to_owned(),
      point(0.0, 0.0),
      point(100.0, 0.0),
      None,
    )];
    let mut annotations = original.clone();
    let edit = AnnotationEdit::begin(
      &mut annotations,
      AnnotationGestureTarget::New,
      point(20.0, 30.0),
      None,
      NewAnnotationKind::Arrow,
      None,
    )
    .unwrap();
    edit.update(&mut annotations, point(80.0, 60.0), false);
    assert_eq!(annotations.len(), 2);
    assert_eq!(annotations[0], original[0]);
    edit.cancel(&mut annotations);
    assert_eq!(annotations, original);
  }

  #[test]
  fn completing_or_cancelling_an_existing_drag_only_changes_its_annotation() {
    let original = vec![
      new_arrow("a".to_owned(), point(0.0, 0.0), point(100.0, 0.0), None),
      new_arrow("b".to_owned(), point(0.0, 50.0), point(100.0, 50.0), None),
    ];
    let mut annotations = original.clone();
    let edit = AnnotationEdit::begin(
      &mut annotations,
      AnnotationGestureTarget::Existing {
        index: 0,
        handle: AnnotationHandle::Body,
      },
      point(50.0, 0.0),
      None,
      NewAnnotationKind::Arrow,
      None,
    )
    .unwrap();
    edit.update(&mut annotations, point(80.0, 20.0), false);
    edit.update(&mut annotations, point(60.0, 10.0), false);
    assert_eq!(annotations[1], original[1]);
    assert_eq!(
      annotations[0].shape,
      AnnotationShape::Arrow {
        start: point(10.0, 10.0),
        control: point(60.0, 10.0),
        end: point(110.0, 10.0)
      }
    );
    edit.cancel(&mut annotations);
    assert_eq!(annotations, original);
    let edit = AnnotationEdit::begin(
      &mut annotations,
      AnnotationGestureTarget::Existing {
        index: 0,
        handle: AnnotationHandle::Body,
      },
      point(50.0, 0.0),
      None,
      NewAnnotationKind::Arrow,
      None,
    )
    .unwrap();
    edit.update(&mut annotations, point(60.0, 10.0), false);
    drop(edit);
    assert_ne!(annotations[0], original[0]);
    assert_eq!(annotations[1], original[1]);
  }

  #[test]
  fn selection_and_capacity_do_not_open_an_edit() {
    let mut annotations =
      vec![new_arrow("a".to_owned(), point(0.0, 0.0), point(100.0, 0.0), None); MAX_ANNOTATIONS];
    let before = annotations.clone();
    for target in [
      AnnotationGestureTarget::Select { index: 0 },
      AnnotationGestureTarget::None,
      AnnotationGestureTarget::New,
      AnnotationGestureTarget::Existing {
        index: MAX_ANNOTATIONS,
        handle: AnnotationHandle::End,
      },
    ] {
      assert!(AnnotationEdit::begin(
        &mut annotations,
        target,
        point(0.0, 0.0),
        None,
        NewAnnotationKind::Arrow,
        None
      )
      .is_none());
      assert_eq!(annotations, before);
    }
  }
}
