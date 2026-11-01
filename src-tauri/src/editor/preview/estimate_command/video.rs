// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The bytes each exported video file's picture takes at each look, found
//! the way the export makes that file: drawn by the compositor, re-encoded
//! by FFmpeg, or copied as recorded. Every look is measured from one pass
//! over the picture: drawn or decoded once, encoded at each look at once.

use std::path::Path;
use std::sync::atomic::AtomicBool;

use super::estimate_helpers;
use crate::editor::export_picture::{ExportPicture, Run};
use crate::editor::EditorState;
use screenwide_export::cursor_export;
use screenwide_export::media_preview::{self, VideoExportOptions};
use screenwide_export::recording_model::{RecordingAudioTrack, RecordingCamera};
use screenwide_export::track_selection::{AudioLayout, TrackSelection};
use screenwide_timeline_edit::TimelinePlan;

/// The cache key's mark for an estimate of the camera's own file.
const CAMERA_FILE: u8 = 1;
/// The cache key's mark for an estimate that follows a timeline, which
/// re-encodes even at Original.
const FOLLOWS_TIMELINE: u8 = 16;

/// One estimate of a recording's export, made for the edit `timeline`.
pub(super) struct Estimate<'a> {
  pub artifact_id: u64,
  pub cancelled: &'a AtomicBool,
  pub duration_ms: u64,
  pub state: &'a EditorState,
  pub timeline: Option<&'a TimelinePlan>,
}

impl Estimate<'_> {
  /// The screen's file at each of `videos`, `source` being the recording
  /// with `tracks`.
  pub(super) fn screen(
    &self,
    picture: ExportPicture<'_>,
    source: &Path,
    tracks: &[RecordingAudioTrack],
    videos: &[VideoExportOptions],
  ) -> Result<Vec<u64>, String> {
    if picture.is_composed() {
      return self.composed(picture, videos);
    }
    // The recording's own video, copied: its file less its audio.
    let copied = || {
      let all = tracks
        .iter()
        .map(|track| track.stream_index)
        .collect::<Vec<_>>();
      let audio = TrackSelection::new(tracks, &all).estimated_audio_bytes(
        tracks,
        AudioLayout::SeparateTracks,
        self.duration_ms,
      );
      let recorded = std::fs::metadata(source).map_or(0, |metadata| metadata.len());
      recorded.saturating_sub(audio)
    };
    self.each(videos, copied, |reencoded| {
      self.reencoded(0, source, self.duration_ms, reencoded)
    })
  }

  /// The camera's own file at each of `videos`.
  pub(super) fn camera(
    &self,
    picture: ExportPicture<'_>,
    camera: &RecordingCamera,
    videos: &[VideoExportOptions],
  ) -> Result<Vec<u64>, String> {
    if picture.is_composed() {
      return self.composed(picture, videos);
    }
    self.each(
      videos,
      || camera.original_size_bytes,
      |reencoded| self.reencoded(CAMERA_FILE, &camera.path, camera.duration_ms, reencoded),
    )
  }

  /// A picture the compositor draws, measured by the platform's encoder at
  /// each of `videos`.
  pub(super) fn composed(
    &self,
    picture: ExportPicture<'_>,
    videos: &[VideoExportOptions],
  ) -> Result<Vec<u64>, String> {
    cursor_export::estimate_video_bytes(
      picture.request(Run {
        cancelled: self.cancelled,
        destination: Path::new(""),
        on_progress: &mut |_| {},
      }),
      videos,
    )
  }

  /// `bytes` of the whole recording carried onto the edited video's length.
  pub(super) fn to_output(&self, bytes: u64) -> u64 {
    self.timeline.map_or(bytes, |timeline| {
      bytes.saturating_mul(timeline.duration_ms()) / self.duration_ms.max(1)
    })
  }

  /// The file at each of `videos`: `copied` where it is the recording's own
  /// video, copied as it is, and the rest from `reencode`, which re-encodes
  /// them together.
  fn each(
    &self,
    videos: &[VideoExportOptions],
    copied: impl Fn() -> u64,
    reencode: impl FnOnce(&[VideoExportOptions]) -> Result<Vec<u64>, String>,
  ) -> Result<Vec<u64>, String> {
    let is_copied = |video: &VideoExportOptions| {
      self.timeline.is_none()
        && video.compression == 0
        && video.resolution_scale_percent >= video.source_scale_percent
    };
    let reencoded = videos
      .iter()
      .filter(|video| !is_copied(video))
      .copied()
      .collect::<Vec<_>>();
    let mut reencoded = reencode(&reencoded)?.into_iter();
    videos
      .iter()
      .map(|video| {
        if is_copied(video) {
          Ok(copied())
        } else {
          reencoded
            .next()
            .ok_or_else(|| "The estimate measured too few looks".to_owned())
        }
      })
      .collect()
  }

  /// `source` re-encoded by FFmpeg at each of `videos`, over its whole
  /// length, each kept for the next estimate that asks the same.
  fn reencoded(
    &self,
    file: u8,
    source: &Path,
    duration_ms: u64,
    videos: &[VideoExportOptions],
  ) -> Result<Vec<u64>, String> {
    let follows_timeline = self.timeline.is_some();
    let key = |video: &VideoExportOptions| {
      (
        self.artifact_id,
        file
          | if follows_timeline {
            FOLLOWS_TIMELINE
          } else {
            0
          },
        video.compression,
        video.resolution_scale_percent,
      )
    };
    let kept = videos
      .iter()
      .map(|video| estimate_helpers::kept_estimate(self.state, &key(video)))
      .collect::<Vec<_>>();
    let missing = videos
      .iter()
      .zip(&kept)
      .filter(|(_, kept)| kept.is_none())
      .map(|(video, _)| *video)
      .collect::<Vec<_>>();
    let mut measured = media_preview::estimate_compressed_video_bytes(
      source,
      duration_ms,
      &missing,
      follows_timeline,
    )?
    .into_iter()
    .zip(&missing)
    .map(|(bytes, video)| estimate_helpers::keep_estimate(self.state, key(video), bytes));
    kept
      .into_iter()
      .map(|kept| {
        kept
          .or_else(|| measured.next())
          .map(|bytes| self.to_output(bytes))
          .ok_or_else(|| "The estimate measured too few looks".to_owned())
      })
      .collect()
  }
}
