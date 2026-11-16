// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Notes as the projects window shows them: read from their project's
//! Markdown file, edited there and saved as edited, and their file's path
//! copied to paste into an assistant. Opened from the island once saved, the
//! projects window comes out on them; a window made for this asks which
//! notes once its page has loaded, as an event sent before then is lost.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use screenwide_notes::clips;
use serde::Serialize;
use tauri::ipc::Response;
use tauri::{AppHandle, Emitter};
use tauri_plugin_clipboard_manager::ClipboardExt;
use ts_rs::TS;

use crate::app_windows::WindowLabel;

const OPEN_EVENT: &str = "notes://open";

/// The notes the projects window was last asked to show and has not yet.
static PENDING: Mutex<Option<PathBuf>> = Mutex::new(None);

/// A notes project's notes.
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Notes {
  /// The Markdown file.
  pub file: String,
  pub markdown: String,
  /// Still being taken, so not yet to be edited.
  pub is_taking: bool,
  /// The numbers of the speakers whose voice can still be heard: those not
  /// yet named.
  pub voices: Vec<u32>,
}

/// Shows the notes whose Markdown file is `notes` in the projects window.
pub(crate) fn open(app: &AppHandle, notes: &Path) -> Result<(), String> {
  show(app, &project_of(notes)?)
}

/// Shows the notes of the project whose manifest is `project` in the
/// projects window, which asks for them as it loads if it cannot yet hear.
pub(crate) fn show(app: &AppHandle, project: &Path) -> Result<(), String> {
  *PENDING.lock().unwrap_or_else(PoisonError::into_inner) = Some(project.to_owned());
  crate::project_browser::show(app);
  app
    .emit_to(WindowLabel::Projects.as_str(), OPEN_EVENT, project)
    .map_err(|error| error.to_string())
}

/// The manifest of the project the notes at `notes` are in.
fn project_of(notes: &Path) -> Result<PathBuf, String> {
  let root = notes
    .parent()
    .ok_or_else(|| "The notes have no folder".to_owned())?;
  let name = root
    .file_name()
    .and_then(|name| name.to_str())
    .ok_or_else(|| "The notes' folder has no name".to_owned())?;
  Ok(root.join(format!("{name}.{}", screenwide_project::EXTENSION)))
}

/// The Markdown file of the notes project whose manifest is `project`.
fn notes_file(project: &Path) -> Result<PathBuf, String> {
  let manifest = screenwide_project::read(project).map_err(|error| error.to_string())?;
  let notes = manifest
    .notes
    .ok_or_else(|| "This project holds no notes".to_owned())?;
  screenwide_project::resolve(project, &notes.file)
}

/// The notes the projects window was asked to show before it could hear.
#[tauri::command]
pub fn take_pending_notes() -> Option<PathBuf> {
  PENDING
    .lock()
    .unwrap_or_else(PoisonError::into_inner)
    .take()
}

#[tauri::command]
pub fn read_notes(project: PathBuf) -> Result<Notes, String> {
  let file = notes_file(&project)?;
  let markdown = std::fs::read_to_string(&file).map_err(|error| error.to_string())?;
  Ok(Notes {
    voices: clips::kept(root_of(&file)?),
    file: file.to_string_lossy().into_owned(),
    markdown,
    is_taking: super::is_taking_into(&project),
  })
}

/// Saves `markdown` as the notes of `project`, whole or not at all, unless
/// those notes are still being taken. Answers the numbers of the speakers
/// whose voice can still be heard: a speaker named since is heard no more.
#[tauri::command]
pub fn save_notes(project: PathBuf, markdown: String) -> Result<Vec<u32>, String> {
  if super::is_taking_into(&project) {
    return Err("These notes are still being taken".to_owned());
  }
  let file = notes_file(&project)?;
  let writing = file.with_extension("md.part");
  std::fs::write(&writing, &markdown).map_err(|error| error.to_string())?;
  std::fs::rename(writing, &file).map_err(|error| error.to_string())?;
  let root = root_of(&file)?;
  clips::forget_named(root, &markdown);
  Ok(clips::kept(root))
}

/// The few seconds of speaker `number`'s voice kept with the notes of
/// `project`, as a WAV file.
#[tauri::command]
pub fn read_voice_clip(project: PathBuf, number: u32) -> Result<Response, String> {
  let file = notes_file(&project)?;
  clips::read(root_of(&file)?, number)
    .map(Response::new)
    .map_err(|error| error.to_string())
}

fn root_of(notes: &Path) -> Result<&Path, String> {
  notes
    .parent()
    .ok_or_else(|| "The notes have no folder".to_owned())
}

/// Copies the path of the notes of `project`, to paste into an assistant.
#[tauri::command]
pub fn copy_notes_path(app: AppHandle, project: PathBuf) -> Result<(), String> {
  let file = notes_file(&project)?;
  app
    .clipboard()
    .write_text(file.to_string_lossy().into_owned())
    .map_err(|error| error.to_string())
}
