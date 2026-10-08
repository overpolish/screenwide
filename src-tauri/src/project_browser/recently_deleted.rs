// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the browser does in Recently Deleted: put projects back, or send
//! them on to the system Trash before their time.

use std::path::{Path, PathBuf};

use tauri::AppHandle;

use crate::project::library;

/// Runs `each` over `files`, trying every one, and reports the failures
/// together.
fn each(
  app: &AppHandle,
  files: &[PathBuf],
  each: impl Fn(&AppHandle, &Path) -> Result<(), String>,
) -> Result<(), String> {
  let failures: Vec<String> = files
    .iter()
    .filter_map(|file| each(app, file).err())
    .collect();
  if failures.is_empty() {
    Ok(())
  } else {
    Err(failures.join("\n"))
  }
}

/// Puts deleted projects back where they came from, and on Recent.
#[tauri::command]
pub fn restore_projects(app: AppHandle, files: Vec<PathBuf>) -> Result<(), String> {
  each(&app, &files, |app, file| {
    crate::project::restore(app, file).map(|_| ())
  })
}

/// Sends deleted projects on to the system Trash now.
#[tauri::command]
pub fn trash_projects(app: AppHandle, files: Vec<PathBuf>) -> Result<(), String> {
  each(&app, &files, library::trash)
}

/// Sends everything in Recently Deleted on to the system Trash.
#[tauri::command]
pub fn empty_recently_deleted(app: AppHandle) -> Result<(), String> {
  let files: Vec<PathBuf> = crate::project::recently_deleted(&app)
    .into_iter()
    .map(|deleted| deleted.file)
    .collect();
  each(&app, &files, library::trash)
}
