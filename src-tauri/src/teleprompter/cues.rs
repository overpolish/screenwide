// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

//! Cues: pictures pinned to words of a script, shown while it is read so the
//! reader sees what they are about to talk about. Each is a still of a
//! recording at a moment, as its edit shows it, or a screenshot as edited,
//! kept as a picture of its own beside the script: a recording renamed,
//! moved or put away leaves its cues as they were, and one edited since is
//! drawn again as the script opens. The script's words stay plain text; the
//! cues are kept beside them, each by where its word starts, which the
//! teleprompter moves with every edit and saves here.

// Commands are registered by their own module's path, where their handler
// macros are.
pub(super) mod make;
mod picture;
mod refresh;

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use ts_rs::TS;

/// The cues of a script, in its project's folder.
const CUES_FILE: &str = "cues.json";
/// Their pictures, in its project's folder.
const PICTURES: &str = "cues";

/// A picture pinned to a word of a script.
#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "TeleprompterCue")]
pub struct Cue {
  pub id: String,
  /// Where its word starts in the script, in UTF-16 units, as the page
  /// counts them.
  pub at: u32,
  /// Its picture's file name, among the script's cue pictures.
  pub image: String,
  /// Its project's name when it was made.
  pub title: String,
  /// Where in what plays it was taken, in ms; none for a screenshot.
  pub played_ms: Option<u64>,
  /// Where in the recording it was taken, in ms, which it is drawn from
  /// again once the recording is edited; none for a screenshot, and for a
  /// cue made before cues kept it.
  #[serde(default)]
  #[ts(optional)]
  pub source_ms: Option<u64>,
  /// The project it was taken from, where it was then.
  pub source: PathBuf,
}

/// A cue with its picture's place, for the page to show.
#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "ShownTeleprompterCue")]
pub struct ShownCue {
  #[serde(flatten)]
  pub cue: Cue,
  pub path: PathBuf,
}

/// Where a recording's cue was taken: in the recording, and in what plays.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "TeleprompterCueTiming")]
pub struct CueTiming {
  pub played_ms: u64,
  pub source_ms: u64,
}

fn folder(script: &Path) -> Result<&Path, String> {
  script
    .parent()
    .ok_or_else(|| "The script has no folder".to_owned())
}

/// `cue` with its picture shown, let through the asset protocol.
fn shown(app: &AppHandle, script: &Path, cue: Cue) -> Result<ShownCue, String> {
  let path = folder(script)?.join(PICTURES).join(&cue.image);
  app
    .asset_protocol_scope()
    .allow_file(&path)
    .map_err(|error| error.to_string())?;
  Ok(ShownCue { cue, path })
}

/// The cues kept for the script `script`, none before its first.
fn read(script: &Path) -> Result<Vec<Cue>, String> {
  match std::fs::read(folder(script)?.join(CUES_FILE)) {
    Ok(bytes) => serde_json::from_slice(&bytes).map_err(|error| error.to_string()),
    Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
    Err(error) => Err(error.to_string()),
  }
}

/// Keeps `cues` as the cues of the script `script`, whole or not at all.
fn write(script: &Path, cues: &[Cue]) -> Result<(), String> {
  let file = folder(script)?.join(CUES_FILE);
  let json = serde_json::to_vec_pretty(cues).map_err(|error| error.to_string())?;
  let mut writing = file.clone().into_os_string();
  writing.push(".part");
  std::fs::write(&writing, json).map_err(|error| error.to_string())?;
  std::fs::rename(writing, &file).map_err(|error| error.to_string())
}

/// The cues of the script project whose manifest is `project`. Pictures no
/// cue holds any more, left by cues taken out or redrawn, are cleared away
/// as the script is opened, when nothing else is changing them; cues whose
/// source was edited since are then drawn again, and the teleprompter told.
#[tauri::command]
pub fn read_cues(app: AppHandle, project: PathBuf) -> Result<Vec<ShownCue>, String> {
  let cues = read(&project)?;
  clear_unheld(&project, &cues);
  refresh::redraw_stale(&app, project.clone(), cues.clone());
  cues
    .into_iter()
    .map(|cue| shown(&app, &project, cue))
    .collect()
}

fn clear_unheld(script: &Path, cues: &[Cue]) {
  let Ok(pictures) = folder(script).map(|folder| folder.join(PICTURES)) else {
    return;
  };
  let Ok(entries) = std::fs::read_dir(pictures) else {
    return;
  };
  for entry in entries.flatten() {
    let name = entry.file_name();
    if !cues.iter().any(|cue| name == cue.image.as_str()) {
      let _ = std::fs::remove_file(entry.path());
    }
  }
}

/// Keeps `cues` as the cues of the script project whose manifest is
/// `project`, whole or not at all.
#[tauri::command]
pub fn save_cues(project: PathBuf, cues: Vec<Cue>) -> Result<(), String> {
  write(&project, &cues)
}
