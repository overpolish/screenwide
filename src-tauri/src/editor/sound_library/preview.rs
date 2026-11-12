// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A stretch of a sound, decoded for the clip's island to preview: every
//! kind the library takes plays in the island's window this way, which its
//! web view could not promise of the files themselves.

use std::process::Stdio;

use screenwide_sound_track::store;
use tauri::ipc::Response;

use super::{open_project, project_sounds};

/// The longest stretch a preview plays, in ms.
const MOST_PREVIEW_MS: u64 = 30_000;

/// The sound `name`, as the recording open in the editor plays it, from
/// `from_ms` for `length_ms` or the longest a preview plays, whichever is
/// shorter, as a WAV.
#[tauri::command]
pub(super) async fn get_sound_preview(
  app: tauri::AppHandle,
  name: String,
  from_ms: u64,
  length_ms: u64,
) -> Result<Response, String> {
  let folder = open_project(&app).and_then(|project| project_sounds(&project));
  tauri::async_runtime::spawn_blocking(move || {
    let (path, _) = store::resolve(folder.as_deref(), &name)
      .ok_or_else(|| "There is no such sound".to_owned())?;
    let seconds = |ms: u64| format!("{:.3}", ms as f64 / 1_000.0);
    let output = screenwide_sidecar::ffmpeg_command()
      .args(["-hide_banner", "-loglevel", "error", "-nostdin", "-ss"])
      .arg(seconds(from_ms))
      .arg("-t")
      .arg(seconds(length_ms.min(MOST_PREVIEW_MS)))
      .arg("-i")
      .arg(&path)
      .args([
        "-vn",
        "-ac",
        "2",
        "-ar",
        "44100",
        "-c:a",
        "pcm_s16le",
        "-f",
        "wav",
        "pipe:1",
      ])
      .stdin(Stdio::null())
      .stderr(Stdio::null())
      .output()
      .map_err(|error| format!("Could not start FFmpeg: {error}"))?;
    if !output.status.success() {
      return Err("The sound could not be previewed".to_owned());
    }
    Ok(Response::new(output.stdout))
  })
  .await
  .map_err(|error| error.to_string())?
}
