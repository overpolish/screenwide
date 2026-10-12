// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where new projects go: the folder chosen in Settings, or the default one.

use std::path::PathBuf;

use screenwide_project::NewProject;
use tauri::{AppHandle, Manager};

/// The folder new projects go in when Settings names none, inside the
/// platform's Movies or Videos folder.
const DEFAULT_FOLDER: &str = "Screenwide";

/// The folder new projects are made in. A folder chosen in Settings that is
/// not there, such as one on a drive that is not plugged in, is an error: the
/// recording must not quietly land somewhere the user did not choose.
pub(crate) fn projects_directory(app: &AppHandle) -> Result<PathBuf, String> {
  if let Some(directory) = crate::settings::current(app).project_directory {
    return if directory.is_dir() {
      Ok(directory)
    } else {
      Err(format!(
        "The projects folder {} is not available. Connect its drive or choose another folder in Settings.",
        directory.display()
      ))
    };
  }
  app
    .path()
    .video_dir()
    .map(|directory| directory.join(DEFAULT_FOLDER))
    .map_err(|error| error.to_string())
}

/// Makes the default projects folder again if it has gone, such as one
/// deleted in Finder, so it is always there to list and to move into. A
/// folder chosen in Settings is never made: one that is missing may be on a
/// drive that is not plugged in, and is shown that way instead.
pub(crate) fn ensure_default_directory(app: &AppHandle) {
  if crate::settings::current(app).project_directory.is_some() {
    return;
  }
  if let Ok(directory) = projects_directory(app) {
    if let Err(error) = std::fs::create_dir_all(&directory) {
      eprintln!(
        "Could not make the projects folder {}: {error}",
        directory.display()
      );
    }
  }
}

/// Makes a new, empty project called `title` in the projects folder.
pub(crate) fn create(app: &AppHandle, title: &str) -> Result<NewProject, String> {
  screenwide_project::create_in(&projects_directory(app)?, title)
}

/// Finishes deleting projects whose recording was discarded just before the
/// app last quit.
pub(crate) fn sweep_cancelled(app: &AppHandle) {
  if let Ok(directory) = projects_directory(app) {
    screenwide_project::sweep_cancelled(&directory);
  }
}
