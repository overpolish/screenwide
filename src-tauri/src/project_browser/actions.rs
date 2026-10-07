// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the browser does to a project or a location.

use std::path::{Path, PathBuf};

use tauri::AppHandle;

use crate::app_windows::WindowLabel;
use crate::project::library;

/// Opens a project in the recording editor. Unlike a double-click, a failure
/// is the browser's to show, beside the project it was opened from.
#[tauri::command]
pub async fn open_project_file(app: AppHandle, file: PathBuf) -> Result<(), String> {
  tauri::async_runtime::spawn_blocking(move || crate::editor::open_project(&app, &file))
    .await
    .map_err(|error| error.to_string())?
}

/// Picks a project from anywhere on disk, for one the browser does not list.
#[tauri::command]
pub fn open_other_project(app: AppHandle) {
  crate::editor::choose_and_open_project(&app);
}

/// Shows the project's folder, selected, in Finder or Explorer.
#[tauri::command]
pub fn reveal_project(file: PathBuf) -> Result<(), String> {
  let root = project_root(&file)?;
  #[cfg(target_os = "macos")]
  let shown = std::process::Command::new("open")
    .arg("-R")
    .arg(root)
    .spawn();
  // Explorer parses its own command line: Rust would quote the whole
  // "/select,<path>" argument when the path has a space, which Explorer
  // rejects by opening its default folder. Only the path may be quoted.
  #[cfg(target_os = "windows")]
  let shown = {
    use std::os::windows::process::CommandExt;
    std::process::Command::new("explorer")
      .raw_arg(format!("/select,\"{}\"", root.display()))
      .spawn()
  };
  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  let shown = std::process::Command::new("xdg-open")
    .arg(root.parent().unwrap_or(root))
    .spawn();
  shown.map(|_| ()).map_err(|error| error.to_string())
}

/// Opens one of the browser's folders in Finder or Explorer.
#[tauri::command]
pub fn open_project_location(path: PathBuf) -> Result<(), String> {
  if !path.is_dir() {
    return Err(format!("{} is not available", path.display()));
  }
  #[cfg(target_os = "macos")]
  let opener = "open";
  #[cfg(target_os = "windows")]
  let opener = "explorer";
  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  let opener = "xdg-open";
  std::process::Command::new(opener)
    .arg(&path)
    .spawn()
    .map(|_| ())
    .map_err(|error| error.to_string())
}

/// Renames a project and returns its new manifest. One open in an editor is
/// opened again there under its new name.
#[tauri::command]
pub fn rename_project(app: AppHandle, file: PathBuf, title: String) -> Result<PathBuf, String> {
  crate::editor::rename_project(&app, &file, &title)
}

/// Moves a project's whole folder to the Trash, closing it first if it is
/// open: it is saved, so closing loses nothing.
#[tauri::command]
pub fn trash_project(app: AppHandle, file: PathBuf) -> Result<(), String> {
  if let Some(kind) = crate::editor::open_kind(&app, &file) {
    crate::editor::close(&app, kind);
  }
  library::trash(&app, &file)
}

/// Takes a project off the recent list without touching it, for one that is
/// no longer where the list says.
#[tauri::command]
pub fn forget_recent_project(app: AppHandle, file: PathBuf) -> Result<(), String> {
  library::forget(&app, &file)
}

/// Asks for a folder to list projects from. Returns the folder added, or None
/// when nothing was chosen.
#[tauri::command]
pub async fn add_project_location(app: AppHandle) -> Result<Option<PathBuf>, String> {
  let parent = tauri::Manager::get_webview_window(&app, WindowLabel::Projects.as_str());
  let picker = app.clone();
  let chosen = tauri::async_runtime::spawn_blocking(move || {
    use tauri_plugin_dialog::DialogExt;
    let mut dialog = picker.dialog().file().set_title("Add Location");
    if let Some(parent) = parent {
      dialog = dialog.set_parent(&parent);
    }
    dialog
      .blocking_pick_folder()
      .and_then(|path| path.into_path().ok())
  })
  .await
  .map_err(|error| error.to_string())?;
  if let Some(folder) = &chosen {
    library::add_location(&app, folder)?;
  }
  Ok(chosen)
}

#[tauri::command]
pub fn remove_project_location(app: AppHandle, path: PathBuf) -> Result<(), String> {
  library::remove_location(&app, &path)
}

fn project_root(file: &Path) -> Result<&Path, String> {
  file
    .parent()
    .ok_or_else(|| "The project has no folder".to_owned())
}
