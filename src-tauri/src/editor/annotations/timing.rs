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

/// How many clips one recording's timeline may carry. Live annotation and the
/// editor share the ceiling, so a mark that was recorded can always be shown.
pub(crate) const MAX_CLIPS: usize = 1_024;

pub(crate) fn validate_clips(clips: &[RecordingAnnotationClip]) -> Result<(), String> {
  let mut ids = std::collections::HashSet::new();
  if clips.len() > MAX_CLIPS {
    return Err("There are too many annotation clips".to_owned());
  }
  for clip in clips {
    let annotation = &clip.annotation;
    let placed = match annotation.shape {
      AnnotationShape::Arrow {
        start,
        control,
        end,
      } => [start, control, end]
        .iter()
        .all(|point| point.x.is_finite() && point.y.is_finite()),
      AnnotationShape::Counter { center, angle, .. } => {
        center.x.is_finite() && center.y.is_finite() && angle.is_finite()
      }
    };
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
/// half open, so a mark ends exactly where the next one may begin.
fn active_clips(
  clips: &[RecordingAnnotationClip],
  track: AnnotationTrack,
  source_ms: u64,
) -> impl Iterator<Item = &RecordingAnnotationClip> {
  clips
    .iter()
    .filter(move |clip| {
      clip.track_id == track && clip.start_ms <= source_ms && source_ms < clip.end_ms
    })
    // The same cap as `MAX_ANNOTATIONS`, written out because that constant is
    // macOS-gated while this module compiles everywhere a document is read.
    .take(32)
}

/// The marks a frame draws, each carrying the reveal window its own clip is
/// at. `frame_ms` is how much source time one drawn frame covers, and is read
/// only to measure the blur's lead: a paused preview passes zero.
///
/// Every mark follows its own reveal, paused or playing, so a scrub previews
/// the animation everywhere. A fresh mark stays visible because its clip is
/// placed a draw-in before the playhead rather than at it.
pub(crate) fn revealed_annotations(
  clips: &[RecordingAnnotationClip],
  track: AnnotationTrack,
  source_ms: u64,
  frame_ms: f32,
) -> Vec<Annotation> {
  active_clips(clips, track, source_ms)
    .map(|clip| {
      let mut annotation = clip.annotation.clone();
      let duration_ms = (clip.end_ms - clip.start_ms) as f32;
      let elapsed_ms = (source_ms - clip.start_ms) as f32;
      if annotation.animated {
        annotation.reveal = match annotation.shape {
          AnnotationShape::Arrow { .. } => {
            super::reveal::reveal_window(elapsed_ms, duration_ms, frame_ms)
          }
          AnnotationShape::Counter { .. } => {
            super::reveal::counter::counter_reveal_window(elapsed_ms, duration_ms, frame_ms)
          }
        };
      }
      annotation
    })
    .collect()
}

/// The marks a frame holds, whole: what the editor's handles sit on and what
/// a gesture edits, neither of which follows the reveal.
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

  /// A paused scrub shows whatever each mark's own reveal says, the chosen
  /// one included: holding a mark and dragging the playhead through its clip
  /// previews the draw-on, and by the end of the draw-in every mark is whole.
  #[test]
  fn a_paused_scrub_previews_every_mark_s_own_reveal() {
    let clips = vec![
      clip("chosen", AnnotationTrack::Primary, 1_000, 4_000),
      clip("other", AnnotationTrack::Primary, 1_000, 4_000),
    ];
    // One frame into three seconds: both clips are still drawing themselves
    // in, so an animated mark shows a window short of the whole path.
    let opening = revealed_annotations(&clips, AnnotationTrack::Primary, 1_050, 0.0);
    assert!(!opening[0].reveal.is_whole(), "{:?}", opening[0].reveal);
    assert!(!opening[1].reveal.is_whole(), "{:?}", opening[1].reveal);
    // A draw-in later - which is where a fresh clip puts the playhead - both
    // are whole.
    let drawn = revealed_annotations(
      &clips,
      AnnotationTrack::Primary,
      1_000 + super::super::reveal::REVEAL_DRAW_IN_MS as u64,
      0.0,
    );
    assert!(drawn[0].reveal.is_whole(), "{:?}", drawn[0].reveal);
    assert!(drawn[1].reveal.is_whole(), "{:?}", drawn[1].reveal);
  }

  /// A mark with the Animate switch off draws whole wherever it is asked
  /// from, chosen or not.
  #[test]
  fn a_mark_that_does_not_animate_draws_whole_throughout() {
    let mut unanimated = clip("still", AnnotationTrack::Primary, 1_000, 4_000);
    unanimated.annotation.animated = false;
    let marks = revealed_annotations(&[unanimated], AnnotationTrack::Primary, 1_050, 0.0);
    assert!(marks[0].reveal.is_whole());
  }

  #[test]
  fn invalid_clip_is_rejected() {
    let invalid = clip("bad", AnnotationTrack::Primary, 2_000, 2_000);
    assert!(validate_clips(&[invalid]).is_err());
  }
}
