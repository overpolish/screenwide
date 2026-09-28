// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Clips follow source time; timeline cuts and rates map their boundaries.
//! A cut inside a clip does not create a new entrance or exit.

use super::pin::AnnotationPin;
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
  /// Where the annotation follows the content it was placed on, when it
  /// does. Only the screen's clips are pinned: the camera is a face, not a
  /// page.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub pin: Option<AnnotationPin>,
  /// How long what the annotation draws along a path takes to draw in - the
  /// whole of an arrow, a highlight or a shape, a text box's pointer - paced
  /// by the editor from the path's length. It leaves over three quarters of
  /// that. Absent from a clip the editor has not paced, which takes its kind's
  /// own time, and from every counter and redaction, which grow into place.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub path_ms: Option<f32>,
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
      || clip.pin.as_ref().is_some_and(|pin| !pin.is_valid())
      || clip.path_ms.is_some_and(|ms| !ms.is_finite() || ms <= 0.0)
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

/// `clip`'s annotation as it is drawn `source_ms` into the recording, and
/// the stretch its arrival and leaving play over; `None` while a pinned
/// annotation's content is off the frame.
pub(crate) fn placed_annotation(
  clip: &RecordingAnnotationClip,
  source_ms: u64,
) -> Option<(Annotation, [u64; 2])> {
  let Some(pin) = clip.pin.as_ref() else {
    return Some((clip.annotation.clone(), [clip.start_ms, clip.end_ms]));
  };
  let placement = super::pin::placement(clip, pin, source_ms);
  let shown = placement.shown?;
  Some((super::pin::displaced(&clip.annotation, &placement), shown))
}

/// The annotations a frame draws, each carrying the reveal window its own
/// clip is at. A pinned annotation arrives and leaves over each stretch its
/// content is on the frame rather than over its whole clip.
pub(crate) fn revealed_annotations(
  clips: &[RecordingAnnotationClip],
  track: AnnotationTrack,
  source_ms: u64,
  frame_ms: f32,
) -> Vec<Annotation> {
  active_clips(clips, track, source_ms)
    .filter_map(|clip| {
      let (mut annotation, [start_ms, end_ms]) = placed_annotation(clip, source_ms)?;
      if annotation.animated {
        annotation.reveal = annotation.shape.kind().reveal_window(
          source_ms.saturating_sub(start_ms) as f32,
          end_ms.saturating_sub(start_ms) as f32,
          frame_ms,
          clip.path_ms,
        );
      }
      Some(annotation)
    })
    .collect()
}

/// The annotations a frame holds whole, for handles and gestures, where they
/// are drawn.
pub(crate) fn active_annotations(
  clips: &[RecordingAnnotationClip],
  track: AnnotationTrack,
  source_ms: u64,
) -> Vec<Annotation> {
  active_clips(clips, track, source_ms)
    .filter_map(|clip| placed_annotation(clip, source_ms).map(|(annotation, _)| annotation))
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
      path_ms: None,
      annotation: Annotation {
        above_camera: false,
        animated: true,
        id: id.to_owned(),
        held: None,
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
          hand_drawn: false,
          manual: false,
          radius: 0.0,
          redaction: Default::default(),
          strength: 0.0,
          width: 8.0,
        },
      },
      track_id,
      start_ms,
      end_ms,
      pin: None,
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

  /// A clip draws in over its own pace, and one the editor has not paced
  /// over its kind's second.
  #[test]
  fn a_clip_draws_in_at_its_own_pace() {
    let drawn = |path_ms: Option<f32>, at: u64| {
      let mut paced = clip("a", AnnotationTrack::Primary, 0, 10_000);
      paced.path_ms = path_ms;
      revealed_annotations(&[paced], AnnotationTrack::Primary, at, 0.0)[0]
        .reveal
        .high
    };
    assert_eq!(drawn(None, 1_000), 1.0);
    assert!(drawn(Some(2_000.0), 1_000) < 1.0);
    assert_eq!(drawn(Some(2_000.0), 2_000), 1.0);
    assert_eq!(drawn(Some(600.0), 600), 1.0);
  }
}
