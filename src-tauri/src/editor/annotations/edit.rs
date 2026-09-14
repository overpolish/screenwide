// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! One live annotation edit. Workspaces own presentation and history; this
//! transaction owns only the marks changed by a pointer gesture.

use super::gesture::{
  drag_handle, next_annotation_id, AnnotationDragOrigin, AnnotationGestureTarget,
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
  /// Selection-only presses are handled by the workspace, without opening an edit.
  pub(crate) fn begin(
    annotations: &mut Vec<Annotation>,
    target: AnnotationGestureTarget,
    point: AnnotationPoint,
    defaults: Option<&AnnotationStyle>,
  ) -> Option<Self> {
    let index = match target {
      AnnotationGestureTarget::NewArrow if annotations.len() < MAX_ANNOTATIONS => annotations.len(),
      AnnotationGestureTarget::Existing { index, .. } if index < annotations.len() => index,
      _ => return None,
    };
    let before = annotations.clone();
    if target == AnnotationGestureTarget::NewArrow {
      annotations.push(new_arrow(next_annotation_id(), point, point, defaults));
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
  pub(crate) fn update(&self, annotations: &mut [Annotation], point: AnnotationPoint) {
    let Some(annotation) = annotations
      .get_mut(self.index)
      .filter(|item| item.id == self.id)
    else {
      return;
    };
    match self.target {
      AnnotationGestureTarget::NewArrow => {
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
      AnnotationGestureTarget::Existing { handle, .. } => {
        drag_handle(annotation, handle, point, &self.origin)
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
      AnnotationGestureTarget::NewArrow,
      point(20.0, 30.0),
      None,
    )
    .unwrap();
    edit.update(&mut annotations, point(80.0, 60.0));
    assert_eq!(annotations.len(), 2);
    assert_eq!(annotations[0], original[0]);
    edit.cancel(&mut annotations);
    assert_eq!(annotations, original);
  }

  #[test]
  fn completing_or_cancelling_an_existing_drag_only_changes_its_mark() {
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
    )
    .unwrap();
    edit.update(&mut annotations, point(80.0, 20.0));
    edit.update(&mut annotations, point(60.0, 10.0));
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
    )
    .unwrap();
    edit.update(&mut annotations, point(60.0, 10.0));
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
      AnnotationGestureTarget::NewArrow,
      AnnotationGestureTarget::Existing {
        index: MAX_ANNOTATIONS,
        handle: AnnotationHandle::End,
      },
    ] {
      assert!(AnnotationEdit::begin(&mut annotations, target, point(0.0, 0.0), None).is_none());
      assert_eq!(annotations, before);
    }
  }
}
