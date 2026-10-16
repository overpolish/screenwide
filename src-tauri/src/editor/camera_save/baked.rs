// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::editor::export_picture::{ExportPicture, Run};
use screenwide_export::cursor_export;
use screenwide_export::media_preview;

/// Exports the screen with the camera baked into it, as
/// [`ExportPicture::baked`] draws them, lasting `progress_duration_ms`.
pub(in crate::editor) fn save_baked_recording(
  picture: ExportPicture<'_>,
  progress_duration_ms: u64,
  directory: &Path,
  stem: &str,
  artifact_id: u64,
  progress_app: &AppHandle,
  cancelled: &AtomicBool,
) -> Result<Option<PathBuf>, String> {
  let mut on_progress = |processed_ms| {
    emit_progress(
      progress_app,
      artifact_id,
      "recording",
      processed_ms,
      progress_duration_ms,
      0.0,
      99.0,
    );
  };
  let path = unique_path(directory, stem, RECORDING_EXTENSION, &|candidate| {
    candidate.exists()
  });
  let result = cursor_export::export(picture.request(Run {
    cancelled,
    destination: &path,
    on_progress: &mut on_progress,
  }))?;
  match result {
    media_preview::ExportRunResult::Completed => {
      emit_progress(
        progress_app,
        artifact_id,
        "finalizing",
        progress_duration_ms,
        progress_duration_ms,
        0.0,
        99.0,
      );
      if !path.is_file() {
        let _ = std::fs::remove_file(&path);
        return Err("The exported recording did not finish publishing".to_owned());
      }
      Ok(Some(path))
    }
    media_preview::ExportRunResult::Cancelled => Ok(None),
  }
}
