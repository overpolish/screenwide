// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The picture the Export island compares: a recording's frame under the
//! editor's playhead as the export would write it and as it comes out at
//! Original, or a screenshot as its JPEG and as drawn.

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

/// A frame written for the Export island, at its own size.
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ExportPreviewFrame {
  /// Where the look's canvas lies in the frame, in its pixels: an export
  /// drawn smaller fits it at one scale, rounding the frame's sides to even,
  /// so the island lays one frame's canvas over the other's to line them up.
  pub canvas: ExportPreviewCanvas,
  pub height: u32,
  pub path: String,
  pub width: u32,
}

/// A rectangle in a frame's pixels.
#[derive(Clone, Copy, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ExportPreviewCanvas {
  pub height: f64,
  pub width: f64,
  pub x: f64,
  pub y: f64,
}

impl ExportPreviewCanvas {
  /// The whole of a `width` by `height` frame.
  fn whole(width: u32, height: u32) -> Self {
    Self::from([0.0, 0.0, f64::from(width), f64::from(height)])
  }
}

impl From<[f64; 4]> for ExportPreviewCanvas {
  fn from([x, y, width, height]: [f64; 4]) -> Self {
    Self {
      height,
      width,
      x,
      y,
    }
  }
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

/// Where the next frame of `side` is written, as an `extension` file. Each
/// frame has its own name, so the island never shows a stale copy of the
/// file; the ones before it on the same side are removed, the island having
/// moved on from them.
fn frame_path(
  directory: &Path,
  side: ExportPreviewSide,
  extension: &str,
) -> Result<std::path::PathBuf, String> {
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
  Ok(directory.join(format!("{prefix}{made}.{extension}")))
}

/// The app cache folder the frames are written to.
fn preview_directory(app: &AppHandle) -> Result<std::path::PathBuf, String> {
  Ok(
    app
      .path()
      .app_cache_dir()
      .map_err(|error| format!("The export preview folder is not available: {error}"))?
      .join(PREVIEW_FOLDER),
  )
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
  let directory = preview_directory(&app)?;
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
    let png = frame_path(&directory, side, "png")?;
    // Read before the picture is spent drawing the frame.
    let canvas = picture.canvas();
    if picture.preview_frame(&cancelled, output_us, side.picture(), &png)?
      == ExportRunResult::Cancelled
      || cancelled.load(Ordering::Acquire)
    {
      return Ok(None);
    }
    let (width, height) = image::image_dimensions(&png).map_err(|error| error.to_string())?;
    Ok(Some(ExportPreviewFrame {
      canvas: canvas.map_or_else(
        || ExportPreviewCanvas::whole(width, height),
        ExportPreviewCanvas::from,
      ),
      height,
      path: png.to_string_lossy().into_owned(),
      width,
    }))
  })
  .await
  .map_err(|error| error.to_string())?
}

/// A screenshot compared before it is saved in a lossy format: as drawn, and
/// as the JPEG or WebP the export writes.
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ScreenshotExportPreview {
  pub exported: ExportPreviewFrame,
  pub original: ExportPreviewFrame,
  /// The picture has see-through pixels the format fills with white, as
  /// only JPEG does.
  pub see_through: bool,
}

/// The open screenshot composed with `screenshot_output`, written as drawn
/// and as the export writes it in `format`. `None` where a newer one was
/// asked for before this one was done.
#[tauri::command]
pub async fn preview_screenshot_export(
  app: AppHandle,
  window: tauri::WebviewWindow,
  format: ScreenshotFormat,
  screenshot_output: ScreenshotWorkspaceOutputSettings,
) -> Result<Option<ScreenshotExportPreview>, String> {
  let kind = kind_of_window(&window)?;
  let directory = preview_directory(&app)?;
  tauri::async_runtime::spawn_blocking(move || {
    let state = app.state::<EditorState>();
    let cancelled = replace_running(&state, ExportPreviewSide::Export);
    let composed = {
      let artifact = state
        .slot(kind)
        .artifact
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
      let Some(EditorArtifact::Screenshot { items, .. }) = artifact.as_ref() else {
        return Err("There is no screenshot to preview".to_owned());
      };
      compose_screenshot_workspace(items, &screenshot_output)?
    };
    if cancelled.load(Ordering::Acquire) {
      return Ok(None);
    }
    let frame = |path: &Path| ExportPreviewFrame {
      canvas: ExportPreviewCanvas::whole(composed.width, composed.height),
      height: composed.height,
      path: path.to_string_lossy().into_owned(),
      width: composed.width,
    };
    // Drawn, written losslessly but quickly: it is only looked at.
    let original = frame_path(&directory, ExportPreviewSide::Original, "png")?;
    let file = std::fs::File::create(&original).map_err(|error| error.to_string())?;
    image::ImageEncoder::write_image(
      image::codecs::png::PngEncoder::new_with_quality(
        std::io::BufWriter::new(file),
        image::codecs::png::CompressionType::Fast,
        image::codecs::png::FilterType::NoFilter,
      ),
      &composed.rgba,
      composed.width,
      composed.height,
      image::ExtendedColorType::Rgba8,
    )
    .map_err(|error| error.to_string())?;
    let exported = frame_path(&directory, ExportPreviewSide::Export, format.extension())?;
    std::fs::write(&exported, format.encode(&composed)?).map_err(|error| error.to_string())?;
    if cancelled.load(Ordering::Acquire) {
      return Ok(None);
    }
    Ok(Some(ScreenshotExportPreview {
      exported: frame(&exported),
      original: frame(&original),
      see_through: format == ScreenshotFormat::Jpeg
        && !screenwide_output::is_opaque(&composed.rgba),
    }))
  })
  .await
  .map_err(|error| error.to_string())?
}
