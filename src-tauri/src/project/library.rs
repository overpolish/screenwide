// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The projects the browser knows beyond the projects folder: the ones opened
//! or made recently, wherever they are, and the folders the user added.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

const LIBRARY_FILE: &str = "project-library.json";
const MAX_RECENTS: usize = 30;
/// Sent whenever what the browser shows may have changed.
pub(crate) const CHANGED_EVENT: &str = "projects://changed";
/// Sent with a project's manifest when the editor has drawn it a new still.
pub(crate) const STILL_EVENT: &str = "projects://still";

/// Each change is a read, an edit and a write; two must not interleave.
static CHANGES: Mutex<()> = Mutex::new(());

#[derive(Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Library {
  /// Projects in Recently Deleted, oldest first.
  pub deleted: Vec<DeletedProject>,
  /// Folders the user added, in the order they were added.
  pub locations: Vec<PathBuf>,
  /// Project manifests, most recent first. A project on a drive that is not
  /// connected stays, so it comes back when the drive does.
  pub recents: Vec<PathBuf>,
}

/// A project set aside in Recently Deleted.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DeletedProject {
  /// When it was deleted, in milliseconds since the epoch.
  pub deleted_ms: u64,
  /// Its manifest where it waits, in the deleted folder beside where it was.
  pub file: PathBuf,
  /// The manifest it had, to put it back where it was.
  pub original: PathBuf,
}

impl Library {
  pub(super) fn remember(&mut self, file: &Path) {
    self.recents.retain(|recent| recent != file);
    self.recents.insert(0, file.to_path_buf());
    self.recents.truncate(MAX_RECENTS);
  }

  fn moved(&mut self, from: &Path, to: &Path) {
    for recent in &mut self.recents {
      if recent == from {
        *recent = to.to_path_buf();
      }
    }
  }
}

fn path(app: &AppHandle) -> Result<PathBuf, String> {
  app
    .path()
    .app_config_dir()
    .map(|directory| directory.join(LIBRARY_FILE))
    .map_err(|error| error.to_string())
}

/// The library as saved. A missing or unreadable file is an empty library:
/// it only ever lists projects, so nothing is lost by starting again.
pub(crate) fn load(app: &AppHandle) -> Library {
  path(app)
    .ok()
    .and_then(|path| std::fs::read(path).ok())
    .and_then(|bytes| serde_json::from_slice(&bytes).ok())
    .unwrap_or_default()
}

pub(super) fn change(app: &AppHandle, edit: impl FnOnce(&mut Library)) -> Result<(), String> {
  let _turn = CHANGES
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  let mut library = load(app);
  edit(&mut library);
  let path = path(app)?;
  if let Some(parent) = path.parent() {
    std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
  }
  let bytes = serde_json::to_vec_pretty(&library).map_err(|error| error.to_string())?;
  std::fs::write(path, bytes).map_err(|error| error.to_string())?;
  notify(app);
  Ok(())
}

/// Tells the browser to look again, for a change it cannot see in the
/// library, such as a new project in the projects folder.
pub(crate) fn notify(app: &AppHandle) {
  let _ = app.emit(CHANGED_EVENT, ());
}

/// Puts `file` at the top of the recent projects.
pub(crate) fn remember(app: &AppHandle, file: &Path) {
  if let Err(error) = change(app, |library| library.remember(file)) {
    eprintln!("Could not remember the project: {error}");
  }
}

pub(crate) fn forget(app: &AppHandle, file: &Path) -> Result<(), String> {
  change(app, |library| {
    library.recents.retain(|recent| recent != file)
  })
}

/// Moves the project whose manifest is `file` to the system Trash, from
/// wherever it is, and takes it off Recent and Recently Deleted.
pub(crate) fn trash(app: &AppHandle, file: &Path) -> Result<(), String> {
  super::move_to_trash(file)?;
  change(app, |library| {
    library.recents.retain(|recent| recent != file);
    library.deleted.retain(|deleted| deleted.file != file);
  })
}

/// Follows a project that was renamed.
pub(crate) fn moved(app: &AppHandle, from: &Path, to: &Path) -> Result<(), String> {
  change(app, |library| library.moved(from, to))
}

pub(crate) fn add_location(app: &AppHandle, folder: &Path) -> Result<(), String> {
  change(app, |library| {
    if !library.locations.iter().any(|location| location == folder) {
      library.locations.push(folder.to_path_buf());
    }
  })
}

pub(crate) fn remove_location(app: &AppHandle, folder: &Path) -> Result<(), String> {
  change(app, |library| {
    library.locations.retain(|location| location != folder);
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn a_reopened_project_moves_to_the_top_without_repeating() {
    let mut library = Library::default();
    for name in ["a", "b", "c", "a"] {
      library.remember(Path::new(name));
    }
    assert_eq!(
      library.recents,
      [Path::new("a"), Path::new("c"), Path::new("b")]
    );
  }

  #[test]
  fn keeps_only_the_most_recent_projects() {
    let mut library = Library::default();
    for index in 0..MAX_RECENTS + 5 {
      library.remember(Path::new(&index.to_string()));
    }
    assert_eq!(library.recents.len(), MAX_RECENTS);
    assert_eq!(
      library.recents.first().unwrap(),
      Path::new(&(MAX_RECENTS + 4).to_string())
    );
  }
}
