// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What Settings asks of transcription. Every change also goes out as a
//! change event, which is how windows learn of it; a reply carries only
//! whether the request failed.

use tauri::AppHandle;

use super::models::TranscriptionState;
use super::{catalogue, download, language, models, notes};

#[tauri::command]
pub fn get_transcription_state(app: AppHandle) -> TranscriptionState {
  models::state(&app)
}

/// Settles when the model is in place, the download is cancelled, or it
/// fails, so a failure reaches whoever asked.
#[tauri::command]
pub async fn download_transcription_model(app: AppHandle, model: String) -> Result<(), String> {
  download::run(&app, catalogue::find(&model)?).await
}

#[tauri::command]
pub fn cancel_transcription_download(model: String) -> Result<(), String> {
  download::cancel(catalogue::find(&model)?.id);
  Ok(())
}

#[tauri::command]
pub fn remove_transcription_model(app: AppHandle, model: String) -> Result<(), String> {
  let result = models::remove(&app, catalogue::find(&model)?);
  models::changed(&app);
  notes::changed(&app);
  result
}

#[tauri::command]
pub fn set_transcription_language(app: AppHandle, language: String) -> Result<(), String> {
  language::set(&app, &language)?;
  models::changed(&app);
  Ok(())
}
