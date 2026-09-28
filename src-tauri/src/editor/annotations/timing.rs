// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Clips follow source time, so a cut or a speed change never moves where an
//! annotation sits in the recording. Its arrival and leaving play in output
//! time instead: they start at the first moment of its clip the timeline
//! keeps, end at the last, and take the same time at any speed. A cut inside
//! a clip does not create a new entrance or exit.

use super::pin::AnnotationPin;
use super::Annotation;
use crate::editor::timeline_edit::{output_at_us, rate_at, TimelineRange};
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

/// How far into the stretch `[start_ms, end_ms)` the timeline is at
/// `source_ms`, and how long the stretch plays for, both in output
/// milliseconds. The stretch is measured over what the timeline keeps of it,
/// so one that loses its start to a cut arrives where it is first seen.
/// `None` where the timeline keeps none of it.
pub(crate) fn output_progress(
  ranges: &[TimelineRange],
  [start_ms, end_ms]: [u64; 2],
  source_ms: u64,
) -> Option<(f32, f32)> {
  let at = |ms: u64| output_at_us(ranges, ms as f64 * 1_000.0) / 1_000.0;
  let from = at(start_ms);
  let duration = at(end_ms) - from;
  (duration > 0.0).then(|| ((at(source_ms) - from).max(0.0) as f32, duration as f32))
}

/// The annotations a frame draws, each carrying the reveal window its own
/// clip is at. A pinned annotation arrives and leaves over each stretch its
/// content is on the frame rather than over its whole clip. `ranges` is the
/// timeline the reveal is timed on, empty when nothing is cut; `frame_ms` is
/// how much source time the frame covers.
pub(crate) fn revealed_annotations(
  clips: &[RecordingAnnotationClip],
  track: AnnotationTrack,
  ranges: &[TimelineRange],
  source_ms: u64,
  frame_ms: f32,
) -> Vec<Annotation> {
  let frame_ms = frame_ms / rate_at(ranges, source_ms as f64 * 1_000.0) as f32;
  active_clips(clips, track, source_ms)
    .filter_map(|clip| {
      let (mut annotation, shown) = placed_annotation(clip, source_ms)?;
      if annotation.animated {
        // A stretch the timeline cut away entirely is never seen, so it is
        // left whole for the handles and gestures that still reach it.
        if let Some((elapsed_ms, duration_ms)) = output_progress(ranges, shown, source_ms) {
          annotation.reveal =
            annotation
              .shape
              .kind()
              .reveal_window(elapsed_ms, duration_ms, frame_ms, clip.path_ms);
        }
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
      revealed_annotations(&[paced], AnnotationTrack::Primary, &[], at, 0.0)[0]
        .reveal
        .high
    };
    assert_eq!(drawn(None, 1_000), 1.0);
    assert!(drawn(Some(2_000.0), 1_000) < 1.0);
    assert_eq!(drawn(Some(2_000.0), 2_000), 1.0);
    assert_eq!(drawn(Some(600.0), 600), 1.0);
  }

  fn kept(source_start_ms: u64, source_end_ms: u64, playback_rate: f64) -> TimelineRange {
    TimelineRange {
      output_start_us: 0,
      source_end_us: source_end_ms * 1_000,
      source_start_us: source_start_ms * 1_000,
      playback_rate,
    }
  }

  fn reveal_at(
    ranges: &[TimelineRange],
    at: u64,
  ) -> crate::editor::annotations::reveal::AnnotationReveal {
    let clip = clip("a", AnnotationTrack::Primary, 1_000, 10_000);
    revealed_annotations(&[clip], AnnotationTrack::Primary, ranges, at, 0.0)[0].reveal
  }

  /// A clip whose start the timeline trimmed away arrives from the first
  /// frame it is seen, rather than already part way in.
  #[test]
  fn a_trimmed_start_arrives_where_the_clip_is_first_seen() {
    let ranges = [kept(3_000, 20_000, 1.0)];
    assert_eq!(reveal_at(&ranges, 3_000).high, 0.0);
    assert!(reveal_at(&ranges, 3_500).high < 1.0);
    assert_eq!(reveal_at(&ranges, 4_000).high, 1.0);
  }

  /// A clip whose end the timeline trimmed away has left by the last frame
  /// that is kept, rather than being cut off whole.
  #[test]
  fn a_trimmed_end_leaves_before_the_cut() {
    let ranges = [kept(0, 6_000, 1.0)];
    assert!(reveal_at(&ranges, 5_999).low > 0.99);
    assert!(reveal_at(&[], 5_999).is_whole());
  }

  /// Arriving takes the same output time at any speed.
  #[test]
  fn a_faster_timeline_does_not_hurry_the_arrival() {
    let ranges = [kept(0, 20_000, 2.0)];
    assert!(reveal_at(&ranges, 2_000).high < 1.0);
    assert_eq!(reveal_at(&ranges, 3_000).high, 1.0);
  }
}
