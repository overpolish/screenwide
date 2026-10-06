// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The project browser window: the projects it lists, and where from.
//!
//! It lists the recent projects, wherever they are, and every project one
//! level inside the projects folder and the folders the user added. Nothing
//! else on disk is searched: a wider scan would be slow on large drives and
//! would ask for access to folders the user never pointed it at.

pub(crate) mod actions;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::app_windows::{self, WindowLabel};
use crate::project::ProjectSummary;

/// A folder the browser lists projects from.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectLocation {
  /// False for a folder that is not there, such as one on a drive that is not
  /// connected.
  pub available: bool,
  /// The projects folder new recordings go into, as opposed to one the user
  /// added.
  pub is_default: bool,
  pub name: String,
  pub path: PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ProjectSource {
  Recent,
  Folder { path: PathBuf },
}

fn location(path: PathBuf, is_default: bool) -> ProjectLocation {
  ProjectLocation {
    available: path.is_dir(),
    is_default,
    name: path.file_name().map_or_else(
      || path.display().to_string(),
      |name| name.to_string_lossy().into_owned(),
    ),
    path,
  }
}

pub fn show(app: &AppHandle) -> tauri::Result<()> {
  crate::capture_overlays::dismiss_all(app);
  let window = app
    .get_webview_window(WindowLabel::Projects.as_str())
    .ok_or(tauri::Error::WindowNotFound)?;
  #[cfg(target_os = "macos")]
  app.set_dock_visibility(true)?;
  app_windows::show(&window, true)?;
  app_windows::recover_window_position(app, &window)
}

#[tauri::command]
pub fn show_project_browser(app: AppHandle) -> tauri::Result<()> {
  show(&app)
}

#[tauri::command]
pub fn hide_project_browser(app: AppHandle) -> tauri::Result<()> {
  if let Some(window) = app.get_webview_window(WindowLabel::Projects.as_str()) {
    app_windows::hide_without_focus_transfer(&window)?;
  }
  Ok(())
}

/// The projects folder first, then the folders the user added. A projects
/// folder chosen in Settings that is not there is still listed, unavailable,
/// so the browser can say so.
#[tauri::command]
pub fn get_project_locations(app: AppHandle) -> Vec<ProjectLocation> {
  let default = crate::settings::current(&app)
    .project_directory
    .or_else(|| crate::project::projects_directory(&app).ok());
  default
    .map(|path| location(path, true))
    .into_iter()
    .chain(
      crate::project::library::load(&app)
        .locations
        .into_iter()
        .map(|path| location(path, false)),
    )
    .collect()
}

#[tauri::command]
pub async fn list_projects(
  app: AppHandle,
  source: ProjectSource,
) -> Result<Vec<ProjectSummary>, String> {
  tauri::async_runtime::spawn_blocking(move || match source {
    ProjectSource::Recent => crate::project::library::load(&app)
      .recents
      .iter()
      .map(|file| crate::project::summarize(file))
      .collect(),
    ProjectSource::Folder { path } => crate::project::in_folder(&path),
  })
  .await
  .map_err(|error| error.to_string())
}

/// A project's preview for its card, or None for one without a picture. Made
/// on first ask for a project that has none, so listing never waits on
/// FFmpeg.
#[tauri::command]
pub async fn get_project_thumbnail(app: AppHandle, file: PathBuf) -> Option<PathBuf> {
  let preview = tauri::async_runtime::spawn_blocking(move || {
    crate::editor::project_thumbnail(&file)
      .inspect_err(|error| eprintln!("Could not make a still for {}: {error}", file.display()))
      .ok()
      .flatten()
  })
  .await
  .ok()
  .flatten()?;
  // A project can be in any folder, so rather than open the asset protocol
  // to all of them, each preview is let through as the browser asks for it.
  app
    .asset_protocol_scope()
    .allow_file(&preview)
    .inspect_err(|error| eprintln!("Could not show {}: {error}", preview.display()))
    .ok()?;
  Some(preview)
}
