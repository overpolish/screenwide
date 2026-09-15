// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Clips follow source time; timeline cuts and rates map their boundaries.
//! A cut inside a clip does not create a new entrance or exit.

use super::{Annotation, AnnotationShape};
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

pub(crate) fn validate_clips(clips: &[RecordingAnnotationClip]) -> Result<(), String> {
  let mut ids = std::collections::HashSet::new();
  if clips.len() > 1024 {
    return Err("There are too many annotation clips".to_owned());
  }
  for clip in clips {
    let annotation = &clip.annotation;
    let AnnotationShape::Arrow {
      start,
      control,
      end,
    } = annotation.shape;
    if clip.end_ms <= clip.start_ms
      || annotation.id.is_empty()
      || !ids.insert(&annotation.id)
      || !annotation.style.width.is_finite()
      || annotation.style.width <= 0.0
      || [start, control, end]
        .iter()
        .any(|p| !p.x.is_finite() || !p.y.is_finite())
    {
      return Err("The annotation clip is invalid".to_owned());
    }
  }
  Ok(())
}

pub(crate) fn active_annotations(
  clips: &[RecordingAnnotationClip],
  track: AnnotationTrack,
  source_ms: u64,
) -> Vec<Annotation> {
  clips
    .iter()
    .filter(|clip| clip.track_id == track && clip.start_ms <= source_ms && source_ms < clip.end_ms)
    .take(32)
    .map(|clip| clip.annotation.clone())
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  fn clip(
    id: &str,
    track_id: AnnotationTrack,
    start_ms: u64,
    end_ms: u64,
  ) -> RecordingAnnotationClip {
    RecordingAnnotationClip {
      annotation: Annotation {
        above_camera: false,
        id: id.to_owned(),
        shape: AnnotationShape::Arrow {
          start: Default::default(),
          control: Default::default(),
          end: Default::default(),
        },
        style: crate::editor::annotations::AnnotationStyle {
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
  fn active_annotations_respects_half_open_intervals_and_tracks() {
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
  fn invalid_clip_is_rejected() {
    let invalid = clip("bad", AnnotationTrack::Primary, 2_000, 2_000);
    assert!(validate_clips(&[invalid]).is_err());
  }
}
