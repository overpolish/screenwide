// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Clips follow source time; timeline cuts and rates map their boundaries.
//! A cut inside a clip does not create a new entrance or exit.

use super::Annotation;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AnnotationTrack {
  Primary,
  Camera,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingAnnotationClip {
  pub annotation: Annotation,
  pub track_id: AnnotationTrack,
  pub start_ms: u64,
  pub end_ms: u64,
}

/// Whether a recording's clips can be drawn: each one spans time, has an
/// id of its own, a usable stroke, and a place on the picture.
pub(crate) fn validate_clips(clips: &[RecordingAnnotationClip]) -> Result<(), String> {
  let mut ids = std::collections::HashSet::new();
  for clip in clips {
    let annotation = &clip.annotation;
    let placed = annotation.shape.placed();
    if clip.end_ms <= clip.start_ms
      || annotation.id.is_empty()
      || !ids.insert(&annotation.id)
      || !annotation.style.width.is_finite()
      || annotation.style.width <= 0.0
      || !placed
    {
      return Err("The annotation clip is invalid".to_owned());
    }
  }
  Ok(())
}

/// The clips a source time falls inside, on one track. A clip's bounds are
/// half open, so an annotation ends exactly where the next one may begin.
fn active_clips(
  clips: &[RecordingAnnotationClip],
  track: AnnotationTrack,
  source_ms: u64,
) -> impl Iterator<Item = &RecordingAnnotationClip> {
  clips.iter().filter(move |clip| {
    clip.track_id == track && clip.start_ms <= source_ms && source_ms < clip.end_ms
  })
}

/// The annotations a frame draws, each carrying the reveal window its own clip is at.
pub(crate) fn revealed_annotations(
  clips: &[RecordingAnnotationClip],
  track: AnnotationTrack,
  source_ms: u64,
  frame_ms: f32,
) -> Vec<Annotation> {
  active_clips(clips, track, source_ms)
    .map(|clip| {
      let mut annotation = clip.annotation.clone();
      if annotation.animated {
        annotation.reveal = annotation.shape.kind().reveal_window(
          (source_ms - clip.start_ms) as f32,
          (clip.end_ms - clip.start_ms) as f32,
          frame_ms,
        );
      }
      annotation
    })
    .collect()
}

/// The annotations a frame holds whole, for handles and gestures.
pub(crate) fn active_annotations(
  clips: &[RecordingAnnotationClip],
  track: AnnotationTrack,
  source_ms: u64,
) -> Vec<Annotation> {
  active_clips(clips, track, source_ms)
    .map(|clip| clip.annotation.clone())
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::editor::annotations::{AnnotationShape, AnnotationStyle};

  fn clip(
    id: &str,
    track_id: AnnotationTrack,
    start_ms: u64,
    end_ms: u64,
  ) -> RecordingAnnotationClip {
    RecordingAnnotationClip {
      annotation: Annotation {
        above_camera: false,
        animated: true,
        id: id.to_owned(),
        reveal: Default::default(),
        shape: AnnotationShape::Arrow {
          start: Default::default(),
          control: Default::default(),
          end: Default::default(),
        },
        style: AnnotationStyle {
          align: Default::default(),
          color: "#ff0000".to_owned(),
          head: Default::default(),
          width: 8.0,
        },
      },
      track_id,
      start_ms,
      end_ms,
    }
  }

  #[test]
  fn active_annotations_respects_intervals_and_tracks() {
    let clips = vec![
      clip("a", AnnotationTrack::Primary, 1_000, 2_000),
      clip("b", AnnotationTrack::Camera, 1_000, 2_000),
    ];
    assert_eq!(
      active_annotations(&clips, AnnotationTrack::Primary, 999).len(),
      0
    );
    assert_eq!(
      active_annotations(&clips, AnnotationTrack::Primary, 1_000)[0].id,
      "a"
    );
    assert_eq!(
      active_annotations(&clips, AnnotationTrack::Primary, 2_000).len(),
      0
    );
    assert_eq!(
      active_annotations(&clips, AnnotationTrack::Camera, 1_500)[0].id,
      "b"
    );
  }

  #[test]
  fn active_annotations_no_longer_truncates_at_thirty_two() {
    let clips: Vec<_> = (0..40)
      .map(|index| clip(&index.to_string(), AnnotationTrack::Primary, 0, 1_000))
      .collect();
    assert_eq!(
      active_annotations(&clips, AnnotationTrack::Primary, 500).len(),
      40
    );
  }

  #[test]
  fn invalid_clip_is_rejected() {
    assert!(validate_clips(&[clip("bad", AnnotationTrack::Primary, 2_000, 2_000)]).is_err());
  }
}
