// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Copies made in place. A drag with the duplicate modifier held (Option on
//! macOS, Alt on Windows) carries copies of what is chosen and leaves the
//! originals where they were: each copy goes in just above its original, so
//! it draws over it, under an id of its own.

use crate::gesture::{next_annotation_id, AnnotationGestureTarget, AnnotationHandle};
use crate::timing::RecordingAnnotationClip;
use crate::Annotation;

/// An original and its copy, by their ids.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Copied {
  pub original: String,
  pub copy: String,
}

/// Puts a copy of each annotation `ids` names just above it in
/// `annotations`, and answers which copy is whose, in the list's order.
pub fn copy_in_place(annotations: &mut Vec<Annotation>, ids: &[String]) -> Vec<Copied> {
  let mut copied = Vec::new();
  let mut index = 0;
  while index < annotations.len() {
    if ids.contains(&annotations[index].id) {
      let copy = Annotation {
        id: next_annotation_id(),
        ..annotations[index].clone()
      };
      copied.push(Copied {
        original: annotations[index].id.clone(),
        copy: copy.id.clone(),
      });
      annotations.insert(index + 1, copy);
      index += 1;
    }
    index += 1;
  }
  copied
}

/// Puts the clip of `copied.copy` just above its original's in `clips`,
/// timed as the original and free of its pin, so it stays where it is put
/// rather than following the content. An original that follows its content
/// is copied where it is `shown`; any other as it was drawn, with its places
/// in the scenes. Nothing is added where the original has no clip.
pub fn copy_clip_in_place(
  clips: &mut Vec<RecordingAnnotationClip>,
  copied: &Copied,
  shown: &Annotation,
) {
  let Some(at) = clips
    .iter()
    .position(|clip| clip.annotation.id == copied.original)
  else {
    return;
  };
  let mut clip = clips[at].clone();
  let drawn = if clip.pin.take().is_some() {
    shown
  } else {
    &clip.annotation
  };
  clip.annotation = Annotation {
    id: copied.copy.clone(),
    ..drawn.clone()
  };
  clips.insert(at + 1, clip);
}

/// Puts a copy of the clip of each annotation `ids` names just above it, as
/// it was drawn, and answers which copy is whose, in the clips' order.
pub fn copy_clips_in_place(
  clips: &mut Vec<RecordingAnnotationClip>,
  ids: &[String],
) -> Vec<Copied> {
  let originals: Vec<Annotation> = clips
    .iter()
    .filter(|clip| ids.contains(&clip.annotation.id))
    .map(|clip| clip.annotation.clone())
    .collect();
  originals
    .iter()
    .map(|original| {
      let copied = Copied {
        original: original.id.clone(),
        copy: next_annotation_id(),
      };
      copy_clip_in_place(clips, &copied, original);
      copied
    })
    .collect()
}

/// The gesture a press on `annotations` becomes with the duplicate modifier
/// held: a carry of an annotation's body takes a copy of it, just above it,
/// and moves the copy instead. Any other gesture, or one without the
/// modifier, is left as it is.
pub fn carried_copy(
  annotations: &mut Vec<Annotation>,
  target: AnnotationGestureTarget,
  duplicate: bool,
) -> (AnnotationGestureTarget, Option<Copied>) {
  let AnnotationGestureTarget::Existing {
    index,
    handle: AnnotationHandle::Body,
  } = target
  else {
    return (target, None);
  };
  let Some(original) = annotations.get(index).filter(|_| duplicate) else {
    return (target, None);
  };
  let ids = [original.id.clone()];
  let copied = copy_in_place(annotations, &ids).pop();
  let carried = AnnotationGestureTarget::Existing {
    index: index + 1,
    handle: AnnotationHandle::Body,
  };
  (carried, copied)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::timing::AnnotationTrack;

  fn annotation(id: &str) -> Annotation {
    crate::gesture_tests::arrow(id)
  }

  #[test]
  fn each_copy_goes_just_above_its_original_under_its_own_id() {
    let mut annotations = vec![annotation("a"), annotation("b"), annotation("c")];
    let copied = copy_in_place(&mut annotations, &["a".to_owned(), "c".to_owned()]);
    let ids: Vec<&str> = annotations.iter().map(|one| one.id.as_str()).collect();
    assert_eq!(ids[0], "a");
    assert_eq!(ids[1], copied[0].copy);
    assert_eq!(ids[2], "b");
    assert_eq!(ids[3], "c");
    assert_eq!(ids[4], copied[1].copy);
    assert_ne!(copied[0].copy, copied[1].copy);
    assert_eq!(annotations[1].shape, annotations[0].shape);
  }

  #[test]
  fn only_a_body_carry_with_the_modifier_takes_a_copy() {
    let body = AnnotationGestureTarget::Existing {
      index: 0,
      handle: AnnotationHandle::Body,
    };
    let grip = AnnotationGestureTarget::Existing {
      index: 0,
      handle: AnnotationHandle::End,
    };
    let mut annotations = vec![annotation("a")];
    assert_eq!(carried_copy(&mut annotations, body, false), (body, None));
    assert_eq!(carried_copy(&mut annotations, grip, true), (grip, None));
    assert_eq!(annotations.len(), 1);
    let (carried, copied) = carried_copy(&mut annotations, body, true);
    assert_eq!(annotations.len(), 2);
    assert_eq!(
      carried,
      AnnotationGestureTarget::Existing {
        index: 1,
        handle: AnnotationHandle::Body,
      }
    );
    assert_eq!(
      copied.map(|copied| copied.copy),
      Some(annotations[1].id.clone())
    );
  }

  #[test]
  fn a_copied_clip_keeps_its_time_and_lets_go_of_the_pin() {
    let original = annotation("a");
    let mut shown = original.clone();
    shown.id = "shown".to_owned();
    let mut clips = vec![RecordingAnnotationClip {
      annotation: original,
      track_id: AnnotationTrack::Primary,
      start_ms: 1_000,
      end_ms: 4_000,
      pin: Some(crate::pin::AnnotationPin::default()),
      path_ms: None,
      scene_shapes: Vec::new(),
      recording_span: None,
    }];
    let copied = Copied {
      original: "a".to_owned(),
      copy: "copy".to_owned(),
    };
    copy_clip_in_place(&mut clips, &copied, &shown);
    assert_eq!(clips.len(), 2);
    assert_eq!(clips[1].annotation.id, "copy");
    assert_eq!((clips[1].start_ms, clips[1].end_ms), (1_000, 4_000));
    assert!(clips[1].pin.is_none());
    assert!(clips[0].pin.is_some());
  }
}
