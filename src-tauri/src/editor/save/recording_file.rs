// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::editor::export_picture::{ExportPicture, Run, Screen};
use screenwide_export::cursor_export;
use screenwide_export::media_preview;
use screenwide_export::track_selection;

pub(in crate::editor) struct PrimaryRecordingSaveRequest<'a> {
  pub app: &'a AppHandle,
  pub artifact_id: u64,
  pub audio_tracks: &'a [RecordingAudioTrack],
  pub cancelled: &'a AtomicBool,
  pub directory: &'a Path,
  /// The screen's picture, with the camera's effects only where the
  /// recording is of the camera alone.
  pub picture: Screen<'a>,
  pub progress_share: f64,
  pub stem: &'a str,
}

pub(in crate::editor) fn save_primary_recording(
  request: PrimaryRecordingSaveRequest<'_>,
) -> Result<Option<PathBuf>, String> {
  let Screen {
    duration_ms,
    layout,
    screen,
    selection,
    timeline,
    video,
    ..
  } = request.picture;
  let progress_duration_ms = timeline.map_or(
    duration_ms,
    screenwide_timeline_edit::TimelinePlan::duration_ms,
  );
  let mut on_progress = |processed_ms| {
    camera_save::emit_progress(
      request.app,
      request.artifact_id,
      "recording",
      processed_ms,
      progress_duration_ms,
      0.0,
      request.progress_share,
    );
  };
  let picture = ExportPicture::screen(request.picture);
  if picture.is_composed() {
    let path = unique_path(
      request.directory,
      request.stem,
      RECORDING_EXTENSION,
      &|candidate| candidate.exists(),
    );
    return match cursor_export::export(picture.request(Run {
      cancelled: request.cancelled,
      destination: &path,
      on_progress: &mut on_progress,
    }))? {
      media_preview::ExportRunResult::Completed => Ok(Some(path)),
      media_preview::ExportRunResult::Cancelled => Ok(None),
    };
  }
  if timeline.is_none()
    && video.compression == 0
    && video.resolution_scale_percent >= video.source_scale_percent
    && !selection.needs_processing(request.audio_tracks, layout)
  {
    return save_recording_copy(
      screen,
      request.directory,
      request.stem,
      media_preview::remuxer(),
    )
    .map(Some);
  }
  save_selected_recording_copy(
    screen,
    request.directory,
    request.stem,
    selection,
    layout,
    media_preview::ExportRunOptions {
      cancelled: request.cancelled,
      on_progress: &mut on_progress,
      timeline,
      video,
    },
    media_preview::selected_recording_exporter(),
  )
}

/// The extension the working recording will actually be saved under.
///
/// `.mp4` whenever it can be remuxed into one, because that is the file
/// everything opens and nobody should have to know what the app records to.
/// Without FFmpeg the QuickTime movie is handed over as it is, under its own
/// name: renaming a `.mov` to `.mp4` would produce a file that lies about
/// itself, and some players trust the name over the bytes.
///
/// Falls back to the working file's own extension rather than a constant, so
/// a project recorded on Windows - which really is an `.mp4` - is described
/// correctly wherever it is opened.
pub(in crate::editor) fn delivered_extension(working: &Path, can_remux: bool) -> &str {
  if can_remux {
    return RECORDING_EXTENSION;
  }

  working
    .extension()
    .and_then(|extension| extension.to_str())
    .unwrap_or(WORKING_RECORDING_EXTENSION)
}

/// Puts a copy of a recording where the user asked for it, as an .mp4 if
/// that is possible and honestly as what it is if it is not. The recording
/// itself stays in its project.
///
/// The remux is attempted first and its failure is not an error: FFmpeg being
/// absent, or refusing the file, is no reason to lose a recording the user
/// just asked to keep. The path returned is the one that was written - never
/// a name the caller assumed.
pub(in crate::editor) fn save_recording_copy(
  working: &Path,
  directory: &Path,
  stem: &str,
  remux: Option<media_preview::Remux>,
) -> Result<PathBuf, String> {
  let taken = |candidate: &Path| candidate.exists();
  if let Some(remux) = remux {
    let path = unique_path(directory, stem, RECORDING_EXTENSION, &taken);
    if remux(working, &path).is_ok() {
      return Ok(path);
    }
  }

  let path = unique_path(directory, stem, delivered_extension(working, false), &taken);
  std::fs::copy(working, &path).map_err(|error| error.to_string())?;
  Ok(path)
}

#[allow(clippy::too_many_arguments)]
pub(in crate::editor) fn save_selected_recording_copy(
  working: &Path,
  directory: &Path,
  stem: &str,
  selection: &track_selection::TrackSelection,
  layout: track_selection::AudioLayout,
  run: media_preview::ExportRunOptions<'_>,
  exporter: Option<media_preview::SelectedRecordingExport>,
) -> Result<Option<PathBuf>, String> {
  let exporter = exporter.ok_or_else(|| {
    "FFmpeg is required to compress or change which audio tracks are exported".to_owned()
  })?;
  let path = unique_path(directory, stem, RECORDING_EXTENSION, &|candidate| {
    candidate.exists()
  });
  match exporter(working, &path, selection, layout, run)? {
    media_preview::ExportRunResult::Completed => Ok(Some(path)),
    media_preview::ExportRunResult::Cancelled => Ok(None),
  }
}

pub(in crate::editor) fn scale_percent(scale_factor: f32) -> u16 {
  (scale_factor.max(1.0) * 100.0).round().clamp(100.0, 400.0) as u16
}
