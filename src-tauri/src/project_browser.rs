// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The project browser window: the projects it lists, and where from.
//!
//! It lists the recent projects, wherever they are, and every project one
//! level inside the projects folder and the folders the user added. Nothing
//! else on disk is searched: a wider scan would be slow on large drives and
//! would ask for access to folders the user never pointed it at.

pub(crate) mod actions;
pub(crate) mod copying;
mod drives;
pub(crate) mod recently_deleted;
pub(crate) mod scrub;

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use ts_rs::TS;

use crate::app_windows::{self, WindowLabel};
use crate::project::ProjectSummary;

/// A folder the browser lists projects from.
#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ProjectLocation {
  /// False for a folder that is not there, such as one on a drive that is not
  /// connected.
  pub available: bool,
  /// The projects folder new recordings go into, as opposed to one the user
  /// added.
  pub is_default: bool,
  pub name: String,
  /// On a drive other than the one holding the user's home folder, such as
  /// an external disk. A folder that is not there counts as one: a missing
  /// location is almost always a drive that is not plugged in.
  pub on_other_drive: bool,
  pub path: PathBuf,
}

/// Where the browser lists projects from: the recent ones, a folder, or
/// Recently Deleted.
#[derive(Debug, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum ProjectSource {
  Recent,
  Folder { path: PathBuf },
  Deleted,
}

/// The drive a folder is on, as something two folders can be compared by.
/// None when the folder is not there.
fn drive_of(path: &Path) -> Option<String> {
  #[cfg(unix)]
  {
    use std::os::unix::fs::MetadataExt;
    std::fs::metadata(path)
      .ok()
      .map(|metadata| metadata.dev().to_string())
  }
  #[cfg(windows)]
  {
    // The volume a path is on is its prefix once links are followed: a
    // drive letter or a share, compared without regard to case.
    let resolved = std::fs::canonicalize(path).ok()?;
    match resolved.components().next()? {
      std::path::Component::Prefix(prefix) => {
        Some(prefix.as_os_str().to_string_lossy().to_lowercase())
      }
      _ => None,
    }
  }
}

fn location(path: PathBuf, is_default: bool, home_drive: Option<&str>) -> ProjectLocation {
  let drive = drive_of(&path);
  ProjectLocation {
    available: path.is_dir(),
    is_default,
    name: path.file_name().map_or_else(
      || path.display().to_string(),
      |name| name.to_string_lossy().into_owned(),
    ),
    on_other_drive: match (drive.as_deref(), home_drive) {
      (None, _) => true,
      (Some(drive), Some(home)) => drive != home,
      (Some(_), None) => false,
    },
    path,
  }
}

pub fn show(app: &AppHandle) -> tauri::Result<()> {
  crate::capture_overlays::dismiss_all(app);
  let window = app
    .get_webview_window(WindowLabel::Projects.as_str())
    .ok_or(tauri::Error::WindowNotFound)?;
  drives::watch(app);
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

/// The projects folder first, then the folders the user added. The default
/// projects folder is made again if it has gone; one chosen in Settings that
/// is not there is still listed, unavailable, so the browser can say so.
#[tauri::command]
pub fn get_project_locations(app: AppHandle) -> Vec<ProjectLocation> {
  crate::project::ensure_default_directory(&app);
  let home_drive = app.path().home_dir().ok().and_then(|home| drive_of(&home));
  let home_drive = home_drive.as_deref();
  let default = crate::settings::current(&app)
    .project_directory
    .or_else(|| crate::project::projects_directory(&app).ok());
  default
    .map(|path| location(path, true, home_drive))
    .into_iter()
    .chain(
      crate::project::library::load(&app)
        .locations
        .into_iter()
        .map(|path| location(path, false, home_drive)),
    )
    .collect()
}

/// The projects of `source`, each with its length as edited: the card says
/// how long the take plays, and its scrub strip runs along the edit.
#[tauri::command]
pub async fn list_projects(
  app: AppHandle,
  source: ProjectSource,
) -> Result<Vec<ProjectSummary>, String> {
  tauri::async_runtime::spawn_blocking(move || {
    let mut projects: Vec<ProjectSummary> = match source {
      ProjectSource::Recent => crate::project::library::load(&app)
        .recents
        .iter()
        .map(|file| crate::project::summarize(file))
        .collect(),
      ProjectSource::Folder { path } => crate::project::in_folder(&path),
      ProjectSource::Deleted => {
        // What has waited long enough goes to the Trash before the rest are
        // listed, so the list never shows a project past its time.
        crate::project::expire(&app);
        crate::project::recently_deleted(&app)
          .into_iter()
          .map(|deleted| ProjectSummary {
            expires_ms: Some(deleted.deleted_ms.saturating_add(crate::project::KEEP_MS)),
            ..crate::project::summarize(&deleted.file)
          })
          .collect()
      }
    };
    for project in &mut projects {
      if let Some(duration_ms) = project.duration_ms {
        project.duration_ms = Some(crate::editor::edited_duration_ms(
          &project.file,
          duration_ms,
        ));
      }
    }
    projects
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
