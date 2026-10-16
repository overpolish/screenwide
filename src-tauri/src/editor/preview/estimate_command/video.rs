// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The bytes each exported video file's picture takes, found the way the
//! export makes that file: drawn by the compositor, re-encoded by FFmpeg,
//! or copied as recorded.

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
  /// The screen's file, `source` being the recording with `tracks` and
  /// `video` the scale and compression it is exported at.
  pub(super) fn screen(
    &self,
    picture: ExportPicture<'_>,
    source: &Path,
    tracks: &[RecordingAudioTrack],
    video: VideoExportOptions,
  ) -> Result<u64, String> {
    if picture.is_composed() {
      return self.composed(picture);
    }
    if self.timeline.is_none()
      && video.compression == 0
      && video.resolution_scale_percent >= video.source_scale_percent
    {
      // The recording's own video, copied: its file less its audio.
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
      return Ok(recorded.saturating_sub(audio));
    }
    let bytes = self.reencoded(
      0,
      source,
      self.duration_ms,
      video.compression,
      (video.source_scale_percent, video.resolution_scale_percent),
    )?;
    Ok(self.to_output(bytes))
  }

  /// The camera's own file.
  pub(super) fn camera(
    &self,
    picture: ExportPicture<'_>,
    camera: &RecordingCamera,
    compression: u8,
    resolution_scale_percent: u16,
  ) -> Result<u64, String> {
    if picture.is_composed() {
      return self.composed(picture);
    }
    if self.timeline.is_none() && compression == 0 && resolution_scale_percent == 100 {
      return Ok(camera.original_size_bytes);
    }
    let bytes = self.reencoded(
      CAMERA_FILE,
      &camera.path,
      camera.duration_ms,
      compression,
      (100, resolution_scale_percent),
    )?;
    Ok(self.to_output(bytes))
  }

  /// A picture the compositor draws, measured by the platform's encoder.
  pub(super) fn composed(&self, picture: ExportPicture<'_>) -> Result<u64, String> {
    cursor_export::estimate_video_bytes(picture.request(Run {
      cancelled: self.cancelled,
      destination: Path::new(""),
      on_progress: &mut |_| {},
    }))
  }

  /// `bytes` of the whole recording carried onto the edited video's length.
  pub(super) fn to_output(&self, bytes: u64) -> u64 {
    self.timeline.map_or(bytes, |timeline| {
      bytes.saturating_mul(timeline.duration_ms()) / self.duration_ms.max(1)
    })
  }

  /// `source` re-encoded by FFmpeg, over its whole length, kept for the
  /// next estimate that asks the same.
  fn reencoded(
    &self,
    file: u8,
    source: &Path,
    duration_ms: u64,
    compression: u8,
    (source_scale_percent, resolution_scale_percent): (u16, u16),
  ) -> Result<u64, String> {
    let follows_timeline = self.timeline.is_some();
    let key = (
      self.artifact_id,
      file
        | if follows_timeline {
          FOLLOWS_TIMELINE
        } else {
          0
        },
      compression,
      resolution_scale_percent,
    );
    if let Some(bytes) = estimate_helpers::kept_estimate(self.state, &key) {
      return Ok(bytes);
    }
    let bytes = media_preview::estimate_compressed_video_bytes(
      source,
      duration_ms,
      compression,
      source_scale_percent,
      resolution_scale_percent,
      follows_timeline,
    )?;
    Ok(estimate_helpers::keep_estimate(self.state, key, bytes))
  }
}
