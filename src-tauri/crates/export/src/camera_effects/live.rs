// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The camera's effects as a drawing applies them: where the background is
//! taken away and what is behind the person there, the grade, and the matte
//! to read beside the camera. The app loads them from the project whenever
//! they change; the preview, the export, the thumbnails and the stills read
//! them alike.
//!
//! The background is taken away only inside its ranges, which are timed on
//! the edited video. A range fades the room out over its first transition
//! window and back in over its last, as long as a scene takes to arrive. One
//! that starts or ends the video holds there, and two ranges that butt hand
//! over at once.

use std::path::PathBuf;
use std::sync::Arc;

use screenwide_annotations::timing::{covers, progress, reaches_video_ends, FrameTime};
use screenwide_timeline::easing::ease_in_out_cubic;
use screenwide_timeline::TimelineRange;

use super::grade::Grade;
use super::CameraBackground;

/// How long the room takes to go or come back, in output milliseconds, as
/// long as a scene takes to arrive. A range too short for two of them gives
/// each half of its length.
const TRANSITION_MS: f32 = 600.0;

/// A matte file, the camera time it holds the matte of, and when each of
/// its frames' camera frames shows. The camera's matte is one file, or the
/// chunks of it made so far.
#[derive(Clone, Debug, PartialEq)]
pub struct MatteFile {
  pub path: PathBuf,
  pub timings: Arc<screenwide_transcriber::MatteTimings>,
  /// The camera time it holds, `[from_ms, to_ms)`.
  pub covers: (u64, u64),
}

impl MatteFile {
  /// How long the matte file runs, in its own milliseconds.
  pub(crate) fn duration_ms(&self) -> u64 {
    (self.timings.frames_ms.len() as u64 * 1_000)
      .div_ceil(u64::from(self.timings.frames_per_second))
  }

  /// Where in the matte file the matte of the camera frame showing at
  /// camera time `ms` is: the start of that frame's slot, in its own
  /// milliseconds.
  pub(crate) fn matte_ms(&self, ms: u64) -> u64 {
    // A frame shows from its own time; the decoders hand over a frame
    // stamped up to 2 ms after the moment asked for, so the matte does too.
    let shown = self
      .timings
      .frames_ms
      .partition_point(|&frame_ms| frame_ms <= ms as f64 + 2.0)
      .saturating_sub(1);
    (shown as u64 * 1_000).div_ceil(u64::from(self.timings.frames_per_second))
  }
}

/// One stretch of the edited video, `[start_ms, end_ms)`, where the
/// background is taken away.
#[derive(Clone, Debug)]
pub struct LiveBackground {
  pub start_ms: u64,
  pub end_ms: u64,
  /// What is behind the person: never [`CameraBackground::Kept`], which is
  /// what the camera is outside every range.
  pub background: CameraBackground,
  /// The range's picture, for an [`CameraBackground::Image`] background.
  pub picture: Option<Arc<::image::RgbaImage>>,
}

/// What is behind the person on one frame.
#[derive(Clone, Copy, Debug)]
pub struct BackgroundAt<'a> {
  pub background: CameraBackground,
  pub picture: Option<&'a Arc<::image::RgbaImage>>,
  /// Which range it is, so what is made for its picture is used again.
  pub index: usize,
  /// How far the room has gone: 0 as recorded, 1 taken away entirely.
  pub strength: f32,
}

/// The effects one drawing applies to the camera.
pub struct LiveCameraEffects {
  /// Where the background is taken away, in order on the edited video.
  pub backgrounds: Vec<LiveBackground>,
  /// How the recording plays onto the edited video the ranges are timed
  /// on; empty where it plays whole.
  pub timeline: Vec<TimelineRange>,
  /// How long the edited video plays.
  pub video_ms: u64,
  pub grade: Option<Grade>,
  /// The matte, as the files it is read from: none until any is made.
  pub matte: Vec<MatteFile>,
  /// Sharpened, and enlarged along its edges where drawn larger.
  pub enhance: bool,
  /// Different for every load, so what is cached from one set of effects,
  /// such as a picture on the GPU, is not drawn with the next.
  pub generation: u64,
}

impl LiveCameraEffects {
  /// What is behind the person on the frame `source_ms` into the recording,
  /// or nothing outside every range, where the camera keeps its room.
  pub fn background_at(&self, source_ms: u64) -> Option<BackgroundAt<'_>> {
    let timeline_ms = FrameTime::on(&self.timeline, source_ms).timeline_ms;
    let video_ms = Some(self.video_ms);
    let index = self
      .backgrounds
      .iter()
      .position(|range| covers(video_ms, [range.start_ms, range.end_ms], timeline_ms))?;
    let range = &self.backgrounds[index];
    let span = [range.start_ms, range.end_ms];
    let (elapsed, length) = progress(span, timeline_ms)?;
    let (starts_video, ends_video) = reaches_video_ends(video_ms, span);
    let window = TRANSITION_MS.min(if starts_video || ends_video {
      length
    } else {
      length / 2.0
    });
    let joined_before = index
      .checked_sub(1)
      .is_some_and(|previous| self.backgrounds[previous].end_ms == range.start_ms);
    let joined_after = self
      .backgrounds
      .get(index + 1)
      .is_some_and(|next| next.start_ms == range.end_ms);
    let arrived = if starts_video || joined_before {
      1.0
    } else {
      elapsed / window
    };
    let staying = if ends_video || joined_after {
      1.0
    } else {
      (length - elapsed) / window
    };
    Some(BackgroundAt {
      background: range.background,
      picture: range.picture.as_ref(),
      index,
      strength: ease_in_out_cubic(arrived.min(staying).clamp(0.0, 1.0)),
    })
  }
}
