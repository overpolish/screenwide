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
) -> Result<Option<crate::editor::commands::recenter::RecenterAnalysis>, String> {
  source_crop.validate()?;
  let sources = headless_sources(&app, artifact_id)?;
  if sources.primary_kind != PrimaryRecordingKind::Screen {
    return Err("Recenter is only available for screen recordings".to_owned());
  }
  tauri::async_runtime::spawn_blocking(move || {
    let position_ms = position_ms.min(sources.duration_ms.saturating_sub(1));
    let frame =
      platform::source_frame_image(&sources.screen_path, position_ms, sources.duration_ms)?;
    Ok(crate::editor::commands::recenter::analyse(
      &frame.rgba,
      frame.width,
      frame.height,
      source_crop,
      24,
    ))
  })
  .await
  .map_err(|error| error.to_string())?
}

#[cfg(all(test, target_os = "macos"))]
#[path = "recenter_tests.rs"]
mod tests;
