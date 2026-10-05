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

/// Whether the stretch `[start_ms, end_ms)` reaches the first and the last
/// moment the timeline keeps, to within a millisecond of rounding: there is
/// nothing before it to arrive from, or after it to leave for. With no
/// ranges it reaches neither.
pub(crate) fn reaches_video_ends(
  ranges: &[TimelineRange],
  [start_ms, end_ms]: [u64; 2],
) -> (bool, bool) {
  let starts = ranges
    .first()
    .is_some_and(|first| start_ms.saturating_mul(1_000) <= first.source_start_us + 1_000);
  let ends = ranges
    .last()
    .is_some_and(|last| (end_ms + 1).saturating_mul(1_000) >= last.source_end_us);
  (starts, ends)
}

/// Whether a clip over `[start_ms, end_ms)` shows `source_ms` into the
/// recording. Its bounds are half open, but one that reaches the end of the
/// video holds to it: rounding can leave its end a millisecond short of the
/// last frame, which would otherwise fall just past it.
pub(crate) fn covers(
  ranges: &[TimelineRange],
  [start_ms, end_ms]: [u64; 2],
  source_ms: u64,
) -> bool {
  start_ms <= source_ms && (source_ms < end_ms || reaches_video_ends(ranges, [start_ms, end_ms]).1)
}

/// The annotations a frame draws, each carrying the reveal window its own
/// clip is at. A pinned annotation arrives and leaves over each stretch its
/// content is on the frame rather than over its whole clip, and spotlights
/// butted up against each other hand their light on rather than fading.
/// `ranges` is the timeline the reveal is timed on, empty when nothing is
/// cut; `frame_ms` is how much source time the frame covers; `picture` is the
/// track's source size in pixels, which a spotlight's glide is paced against.
pub(crate) fn revealed_annotations(
  clips: &[RecordingAnnotationClip],
  track: AnnotationTrack,
  ranges: &[TimelineRange],
  source_ms: u64,
  frame_ms: f32,
  picture: (u32, u32),
) -> Vec<Annotation> {
  use super::spotlight::handoff::{drawn_until, joined, spotlight_links};
  let frame_ms = frame_ms / rate_at(ranges, source_ms as f64 * 1_000.0) as f32;
  let links = spotlight_links(clips);
  clips
    .iter()
    .enumerate()
    .filter(|(index, clip)| {
      clip.track_id == track
        && covers(
          ranges,
          [clip.start_ms, drawn_until(clips, &links, *index)],
          source_ms,
        )
    })
    .filter_map(|(index, clip)| {
      // A spotlight held on for the one it hands to is drawn where it ended.
      let at = source_ms.min(clip.end_ms.saturating_sub(1));
      let (mut annotation, mut shown) = placed_annotation(clip, at)?;
      // A moving image plays from the start of its clip, in recording time,
      // so a scrub lands on the frame playback showed there.
      super::image::play::clocked(&mut annotation, at.saturating_sub(clip.start_ms) as f64);
      if shown[1] == clip.end_ms {
        shown[1] = drawn_until(clips, &links, index);
      }
      if annotation.animated {
        // A stretch the timeline cut away entirely is never seen, so it is
        // left whole for the handles and gestures that still reach it.
        let progress =
          output_progress(ranges, shown, source_ms).and_then(|(elapsed_ms, duration_ms)| {
            // An annotation at either end of the video neither arrives nor
            // leaves there. Every kind caps each phase at a third of the length
            // it is given, so timing it over twice its length puts the phase it
            // skips wholly outside what is seen and still fits the other inside.
            match reaches_video_ends(ranges, shown) {
              (true, true) => None,
              (true, false) => Some((elapsed_ms + duration_ms, duration_ms * 2.0)),
              (false, true) => Some((elapsed_ms, duration_ms * 2.0)),
              (false, false) => Some((elapsed_ms, duration_ms)),
            }
          });
        if let Some((elapsed_ms, duration_ms)) = progress {
          let kind = annotation.shape.kind();
          if kind == super::AnnotationKind::Spotlight {
            annotation = joined(
              clips,
              &links,
              index,
              picture,
              annotation,
              elapsed_ms,
              duration_ms,
            )
            .0;
          } else {
            annotation.reveal = kind.reveal_window(elapsed_ms, duration_ms, frame_ms, clip.path_ms);
          }
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
mod tests;
