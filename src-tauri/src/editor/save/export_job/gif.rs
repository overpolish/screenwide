// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A GIF of the stretch drawn on the timeline. The recording is exported as
//! its video would be, with its captions, scenes and annotations, into a
//! scratch folder, without sound or a separate camera, which a GIF cannot
//! hold; the stretch of that video is then made into the GIF.

use super::*;
use crate::editor::recording_model::RecordingFormat;
use screenwide_export::media_preview::{self, ExportRunResult};

pub(super) fn save_gif(
  artifact: &EditorArtifact,
  writing: &Path,
  stem: &str,
  progress_app: &AppHandle,
  job_cancellation: &AtomicBool,
  mut options: RecordingExportOptions,
) -> Result<Option<PathBuf>, String> {
  let EditorArtifact::Recording {
    id, primary_kind, ..
  } = artifact
  else {
    unreachable!("a GIF is made of a recording");
  };
  if *primary_kind == PrimaryRecordingKind::Audio
    || !(options.include_primary_video || options.include_camera)
  {
    return Err("A GIF is made of the recording's picture".to_owned());
  }
  let range = options
    .gif_range
    .ok_or_else(|| "Draw the stretch the GIF is made of on the timeline".to_owned())?;
  let length_ms = range.end_ms.saturating_sub(range.start_ms);
  if length_ms == 0 || length_ms > media_preview::GIF_LONGEST_MS {
    return Err(format!(
      "A GIF is made of up to {} seconds",
      media_preview::GIF_LONGEST_MS / 1_000
    ));
  }

  options.recording_format = RecordingFormat::Mp4;
  options.enabled_stream_indices.clear();
  // A camera in the picture stays; one saved as a file of its own has no
  // place in a GIF.
  if options.include_primary_video && !options.bake_camera {
    options.include_camera = false;
  }
  let scratch = std::env::temp_dir().join(format!("screenwide-gif-{}-{id}", std::process::id()));
  let _ = std::fs::remove_dir_all(&scratch);
  std::fs::create_dir_all(&scratch)
    .map_err(|error| format!("There is no room to make the GIF in: {error}"))?;
  let result = save_recording_media(
    artifact,
    &scratch,
    stem,
    progress_app,
    job_cancellation,
    options,
  )
  .and_then(|video| {
    let Some(video) = video else {
      return Ok(None);
    };
    let _ = progress_app.emit(
      EXPORT_PROGRESS_EVENT,
      ExportProgress {
        artifact_id: *id,
        phase: "finalizing",
        progress_percent: 99.0,
      },
    );
    let destination = unique_path(writing, stem, "gif", &|candidate| candidate.exists());
    match media_preview::encode_gif(
      &video,
      &destination,
      [range.start_ms, length_ms],
      job_cancellation,
      &mut |_| {},
    )? {
      ExportRunResult::Completed => Ok(Some(destination)),
      ExportRunResult::Cancelled => Ok(None),
    }
  });
  let _ = std::fs::remove_dir_all(&scratch);
  result
}
