// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A clip's start and end are times on the timeline: where it plays in the
//! edited video. A cut, a move or a speed change re-times the clips as it is
//! made, so playback never has to. Frames are decoded from the recording, so
//! each is placed on the timeline first and every clip is timed there. What
//! an annotation reads from the picture under it - a pin's path, a
//! redaction's fill - is read across its `recording_span`, the stretch of the
//! recording its clip plays.

use super::pin::AnnotationPin;
use super::Annotation;
use screenwide_timeline::{
  output_at_us, output_end_at_us, output_start_to_source_us, output_to_source_us, rate_at,
  TimelineRange,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

mod frame_time;
pub use frame_time::{covers, progress, reaches_video_ends, video_ms, FrameTime};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum AnnotationTrack {
  Primary,
  Camera,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RecordingAnnotationClip {
  pub annotation: Annotation,
  pub track_id: AnnotationTrack,
  /// Where the clip starts and ends on the timeline, in milliseconds of the
  /// edited video.
  pub start_ms: u64,
  pub end_ms: u64,
  /// Where the annotation follows the content it was placed on, when it
  /// does. Only the screen's clips are pinned: the camera is a face, not a
  /// page. Its keyframes are times in the recording, where the content is.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub pin: Option<AnnotationPin>,
  /// How long what the annotation draws along a path takes to draw in - the
  /// whole of an arrow, a highlight or a shape, a text box's pointer - paced
  /// by the editor from the path's length. It leaves over three quarters of
  /// that. Absent from a clip the editor has not paced, which takes its kind's
  /// own time, and from every counter and redaction, which grow into place.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub path_ms: Option<f32>,
  /// The stretch of the recording the clip plays, in milliseconds, for what
  /// follows the picture under it. Laid out by [`lay_on_recording`] from the
  /// timeline whatever draws the clips draws them on; never stored or sent.
  #[serde(skip)]
  #[ts(skip)]
  pub recording_span: Option<[u64; 2]>,
}

impl RecordingAnnotationClip {
  /// The stretch of the recording the clip plays: as laid out, or its own
  /// times where the timeline keeps the recording whole.
  pub fn recording_span(&self) -> [u64; 2] {
    self.recording_span.unwrap_or([self.start_ms, self.end_ms])
  }
}

/// Lays each clip's `recording_span` out from its times on the timeline
/// `ranges` lay out. With no ranges the recording plays whole, and the
/// stretch is the clip's own times.
pub fn lay_on_recording(clips: &mut [RecordingAnnotationClip], ranges: &[TimelineRange]) {
  for clip in clips {
    clip.recording_span = (!ranges.is_empty()).then(|| {
      let ms = |us: Option<u64>| us.unwrap_or(0).saturating_add(500) / 1_000;
      let start = ms(output_start_to_source_us(
        ranges,
        clip.start_ms.saturating_mul(1_000),
      ));
      let end = ms(output_to_source_us(
        ranges,
        clip.end_ms.saturating_mul(1_000),
      ));
      [start, end.max(start + 1)]
    });
  }
}

/// Whether a recording's clips can be drawn: each one spans time, has an
/// id of its own, a usable stroke, and a place on the picture.
pub fn validate_clips(clips: &[RecordingAnnotationClip]) -> Result<(), String> {
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

/// `clip`'s annotation as it is drawn over the picture `source_ms` into the
/// recording, and, for a pinned one, the stretch of the recording its
/// content is on the frame, which its arrival and leaving play over. `None`
/// while a pinned annotation's content is off the frame.
pub fn placed_annotation(
  clip: &RecordingAnnotationClip,
  source_ms: u64,
) -> Option<(Annotation, Option<[u64; 2]>)> {
  let Some(pin) = clip.pin.as_ref() else {
    return Some((clip.annotation.clone(), None));
  };
  let placement = super::pin::placement(clip, pin, source_ms);
  let shown = placement.shown?;
  Some((
    super::pin::displaced(&clip.annotation, &placement),
    Some(shown),
  ))
}

/// `span` of the recording, where its content is on the frame, as the
/// stretch of `clip` on the timeline it plays over.
fn shown_on_timeline(
  ranges: &[TimelineRange],
  clip: &RecordingAnnotationClip,
  [start, end]: [u64; 2],
) -> [u64; 2] {
  if ranges.is_empty() {
    return [start.max(clip.start_ms), end.min(clip.end_ms)];
  }
  let from = (output_at_us(ranges, start as f64 * 1_000.0) / 1_000.0).floor() as u64;
  let to = (output_end_at_us(ranges, end as f64 * 1_000.0) / 1_000.0).ceil() as u64;
  [from.max(clip.start_ms), to.min(clip.end_ms)]
}

/// The annotations a frame draws, each carrying the reveal window its own
/// clip is at. A pinned annotation arrives and leaves over each stretch its
/// content is on the frame rather than over its whole clip, and spotlights
/// butted up against each other hand their light on rather than fading.
/// `ranges` is the timeline the clips play on, empty when nothing is cut;
/// `frame_ms` is how much recording time the frame covers; `picture` is the
/// track's source size in pixels, which a spotlight's glide is paced against.
pub fn revealed_annotations(
  clips: &[RecordingAnnotationClip],
  track: AnnotationTrack,
  ranges: &[TimelineRange],
  source_ms: u64,
  frame_ms: f32,
  picture: (u32, u32),
) -> Vec<Annotation> {
  use super::spotlight::handoff::{drawn_until, joined, spotlight_links};
  let time = FrameTime::on(ranges, source_ms);
  let frame_ms = frame_ms / rate_at(ranges, source_ms as f64 * 1_000.0) as f32;
  let video_ms = video_ms(ranges);
  let links = spotlight_links(clips);
  clips
    .iter()
    .enumerate()
    .filter(|(index, clip)| {
      clip.track_id == track
        && covers(
          video_ms,
          [clip.start_ms, drawn_until(clips, &links, *index)],
          time.timeline_ms,
        )
    })
    .filter_map(|(index, clip)| {
      // A spotlight held on for the one it hands to is drawn where it ended.
      // One that holds to the video's end is still on its own clip there.
      let held = time.timeline_ms >= clip.end_ms
        && !reaches_video_ends(video_ms, [clip.start_ms, clip.end_ms]).1;
      let source_at = if held {
        clip.recording_span()[1].saturating_sub(1)
      } else {
        source_ms
      };
      let (mut annotation, shown) = placed_annotation(clip, source_at)?;
      let at = time.timeline_ms.min(clip.end_ms.saturating_sub(1));
      // A moving image plays, and a swaying one sways, from the start of its
      // clip, so a scrub lands on what playback showed.
      super::image::clocked(&mut annotation, at.saturating_sub(clip.start_ms) as f64);
      let mut shown = shown.map_or([clip.start_ms, clip.end_ms], |span| {
        shown_on_timeline(ranges, clip, span)
      });
      if shown[1] == clip.end_ms {
        shown[1] = drawn_until(clips, &links, index);
      }
      if annotation.animated {
        let timing = progress(shown, time.timeline_ms).and_then(|(elapsed_ms, duration_ms)| {
          // An annotation at either end of the video neither arrives nor
          // leaves there. Every kind caps each phase at a third of the length
          // it is given, so timing it over twice its length puts the phase it
          // skips wholly outside what is seen and still fits the other inside.
          match reaches_video_ends(video_ms, shown) {
            (true, true) => None,
            (true, false) => Some((elapsed_ms + duration_ms, duration_ms * 2.0)),
            (false, true) => Some((elapsed_ms, duration_ms * 2.0)),
            (false, false) => Some((elapsed_ms, duration_ms)),
          }
        });
        if let Some((elapsed_ms, duration_ms)) = timing {
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
            if let super::AnnotationShape::Text { clock, .. } = &mut annotation.shape {
              *clock = Some(super::text::model::TextClock {
                elapsed_ms,
                duration_ms,
                frame_ms,
              });
            }
          }
        }
      }
      Some(annotation)
    })
    .collect()
}

/// The annotations the frame `source_ms` into the recording holds whole,
/// for handles and gestures, where they are drawn; `ranges` is the timeline
/// the clips play on.
pub fn active_annotations(
  clips: &[RecordingAnnotationClip],
  track: AnnotationTrack,
  ranges: &[TimelineRange],
  source_ms: u64,
) -> Vec<Annotation> {
  let time = FrameTime::on(ranges, source_ms);
  let video_ms = video_ms(ranges);
  clips
    .iter()
    .filter(|clip| {
      clip.track_id == track && covers(video_ms, [clip.start_ms, clip.end_ms], time.timeline_ms)
    })
    .filter_map(|clip| placed_annotation(clip, source_ms).map(|(annotation, _)| annotation))
    .collect()
}

#[cfg(test)]
mod tests;
