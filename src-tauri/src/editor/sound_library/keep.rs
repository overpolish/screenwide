// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A sound the open recording plays from its own copy, the library having
//! none (it was taken out, or the project came from another computer),
//! taken back into the library from the clip's island.

use screenwide_sound_track::store::{self, Sound};
use screenwide_sound_track::AudioClipKind;
use tauri::Emitter;

use super::{open_project, project_sounds, LIBRARY_EVENT};

/// Takes the sound `name` the open recording carries into the library,
/// listed as `kind`, and says what it is there.
#[tauri::command]
pub async fn keep_project_sound(
  app: tauri::AppHandle,
  name: String,
  kind: AudioClipKind,
) -> Result<Sound, String> {
  let folder = open_project(&app)
    .and_then(|project| project_sounds(&project))
    .ok_or("There is no recording open")?;
  let sound =
    tauri::async_runtime::spawn_blocking(move || store::keep_from_project(&folder, &name, kind))
      .await
      .map_err(|error| error.to_string())??;
  let _ = app.emit(LIBRARY_EVENT, ());
  Ok(sound)
}
