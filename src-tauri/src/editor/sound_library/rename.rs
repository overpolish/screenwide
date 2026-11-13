// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A sound in the library given another title by whoever uses it.

use screenwide_sound_track::store;
use tauri::Emitter;

use super::LIBRARY_EVENT;

/// Gives the sound `name` the title `title` in the library.
#[tauri::command]
pub async fn rename_sound(
  app: tauri::AppHandle,
  name: String,
  title: String,
) -> Result<(), String> {
  tauri::async_runtime::spawn_blocking(move || store::rename(&name, &title))
    .await
    .map_err(|error| error.to_string())??;
  let _ = app.emit(LIBRARY_EVENT, ());
  Ok(())
}
