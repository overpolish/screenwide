// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The frame under the editor's playhead as the export would write it, and
//! as it comes out at Original, for the export window to compare.

use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::export_inputs::ExportInputs;
use super::*;
use screenwide_export::media_preview::{ExportRunResult, PreviewPicture};

/// The app cache folder the frames are written to, which the webview reads
/// through the asset protocol.
const PREVIEW_FOLDER: &str = "export-previews";

static FRAMES_MADE: AtomicU64 = AtomicU64::new(0);

/// Which of the two compared frames to make.
#[derive(Clone, Copy, Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum ExportPreviewSide {
  /// At Original size and compression, what the export is set beside.
  Original,
  /// At the size and compression the export is set to.
  Export,
}

impl ExportPreviewSide {
  fn name(self) -> &'static str {
    match self {
      Self::Original => "original",
      Self::Export => "export",
    }
  }

  /// Where this side's running frame is held in `running_export_preview`.
  fn slot(self) -> usize {
    match self {
      Self::Original => 0,
      Self::Export => 1,
    }
  }

  /// The Original frame is drawn and never encoded; the exported one
  /// passes through the export's encoder.
  fn picture(self) -> PreviewPicture {
    match self {
      Self::Original => PreviewPicture::Drawn,
      Self::Export => PreviewPicture::Encoded,
    }
  }
}

/// A frame written for the export window, at its own size.
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ExportPreviewFrame {
  pub height: u32,
  pub path: String,
  pub width: u32,
}

/// Cancels the frame of `side` being made before this one, which nobody
/// waits for any more, and answers the flag that cancels this one in turn.
/// The other side's frame is made alongside and left running.
fn replace_running(state: &EditorState, side: ExportPreviewSide) -> Arc<AtomicBool> {
  let cancelled = Arc::new(AtomicBool::new(false));
  std::mem::replace(
    &mut state
      .running_export_preview
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner())[side.slot()],
    cancelled.clone(),
  )
  .store(true, Ordering::Release);
  cancelled
}

/// Where the next frame of `side` is written. Each frame has its own name, so
/// the window never shows a stale copy of the file; the ones before it on the
/// same side are removed, the window having moved on from them.
fn frame_path(directory: &Path, side: ExportPreviewSide) -> Result<std::path::PathBuf, String> {
  std::fs::create_dir_all(directory)
    .map_err(|error| format!("The export preview folder could not be made: {error}"))?;
  let prefix = format!("{}-", side.name());
  if let Ok(entries) = std::fs::read_dir(directory) {
    for entry in entries.flatten() {
      if entry.file_name().to_string_lossy().starts_with(&prefix) {
        let _ = std::fs::remove_file(entry.path());
      }
    }
  }
  let made = FRAMES_MADE.fetch_add(1, Ordering::Relaxed);
  Ok(directory.join(format!("{prefix}{made}.png")))
}

/// The frame at `position_ms` into the recording `artifact_id`, written as
/// the export with `options` writes it, or at Original. `None` where a newer
/// frame was asked for before this one was done.
#[tauri::command]
pub async fn preview_recording_export_frame(
  app: AppHandle,
  artifact_id: u64,
  options: RecordingExportOptions,
  position_ms: u64,
  side: ExportPreviewSide,
) -> Result<Option<ExportPreviewFrame>, String> {
  let directory = app
    .path()
    .app_cache_dir()
    .map_err(|error| format!("The export preview folder is not available: {error}"))?
    .join(PREVIEW_FOLDER);
  tauri::async_runtime::spawn_blocking(move || {
    let state = app.state::<EditorState>();
    let cancelled = replace_running(&state, side);
    let mut inputs = ExportInputs::read(&state, artifact_id, options)?;
    if let ExportPreviewSide::Original = side {
      inputs.compression = 0;
      inputs.resolution_scale_percent = inputs.original_scale_percent();
    }
    let source_us = position_ms.saturating_mul(1_000);
    let output_us = inputs
      .timeline
      .as_ref()
      .map_or(Some(source_us), |timeline| {
        timeline.source_to_output_us(source_us)
      })
      .ok_or_else(|| "The playhead is on a part of the recording the edit leaves out".to_owned())?;
    let picture = inputs
      .primary_picture()
      .ok_or_else(|| "The export has no video to preview".to_owned())?;
    let png = frame_path(&directory, side)?;
    if picture.preview_frame(&cancelled, output_us, side.picture(), &png)?
      == ExportRunResult::Cancelled
      || cancelled.load(Ordering::Acquire)
    {
      return Ok(None);
    }
    let (width, height) = image::image_dimensions(&png).map_err(|error| error.to_string())?;
    Ok(Some(ExportPreviewFrame {
      height,
      path: png.to_string_lossy().into_owned(),
      width,
    }))
  })
  .await
  .map_err(|error| error.to_string())?
}
