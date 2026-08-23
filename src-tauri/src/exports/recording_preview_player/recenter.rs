// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::AppHandle;

use super::{platform, sources::headless_sources};
use crate::recording::PrimaryRecordingKind;
use crate::screenshots::NormalizedSourceRect;

#[tauri::command]
pub async fn get_recording_content_bounds(
  app: AppHandle,
  artifact_id: u64,
  position_ms: u64,
  source_crop: NormalizedSourceRect,
) -> Result<Option<crate::exports::commands::recenter::RecenterAnalysis>, String> {
  source_crop.validate()?;
  let sources = headless_sources(&app, artifact_id)?;
  if sources.primary_kind != PrimaryRecordingKind::Screen {
    return Err("Recenter is only available for screen recordings".to_owned());
  }
  tauri::async_runtime::spawn_blocking(move || {
    let position_ms = position_ms.min(sources.duration_ms.saturating_sub(1));
    let encoded =
      platform::source_frame_jpeg(&sources.screen_path, position_ms, sources.duration_ms)?;
    let rgba = image::load_from_memory(&encoded)
      .map_err(|error| error.to_string())?
      .into_rgba8();
    let (width, height) = rgba.dimensions();
    Ok(crate::exports::commands::recenter::analyse(
      rgba.as_raw(),
      width,
      height,
      source_crop,
      24,
    ))
  })
  .await
  .map_err(|error| error.to_string())?
}
