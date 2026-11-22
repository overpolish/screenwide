// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A capture opened straight in the editor, kept out of Projects until it is
//! saved.
//!
//! Its project is made in the app's cache rather than the projects folder
//! and left out of the library, so the editor works on it as on any other,
//! and Save moves it into the projects folder. Let go of unsaved, it is
//! deleted: copied, closed, deleted or exported with the project. Work kept
//! nowhere else is not deleted with it: closing the window asks first, and
//! a project opened in its place keeps it in Projects. A start sweeps away
//! any left by an app that stopped first.

use super::*;

mod closing;

pub use closing::close_screenshot_editor;

const FOLDER: &str = "temporary-screenshots";

fn folder(app: &AppHandle) -> Result<PathBuf, String> {
  app
    .path()
    .app_cache_dir()
    .map(|cache| cache.join(FOLDER))
    .map_err(|error| format!("The cache directory is not available: {error}"))
}

/// Makes a temporary project called `title`.
pub(super) fn create(
  app: &AppHandle,
  title: &str,
) -> Result<screenwide_project::NewProject, String> {
  screenwide_project::create_in(&folder(app)?, title)
}

pub(crate) fn is_temporary(app: &AppHandle, project: &Path) -> bool {
  folder(app).is_ok_and(|folder| project.starts_with(folder))
}

/// Deletes `project` if it is temporary, and says whether it was.
pub(crate) fn discard(app: &AppHandle, project: &Path) -> bool {
  if !is_temporary(app, project) {
    return false;
  }
  if let Some(root) = project.parent() {
    if let Err(error) = std::fs::remove_dir_all(root) {
      eprintln!("Could not delete a temporary screenshot: {error}");
    }
  }
  true
}

/// Lets go of a temporary `project` the editor no longer holds: kept in
/// Projects when it holds work kept nowhere else, deleted otherwise. Says
/// whether it was temporary.
pub(crate) fn let_go(app: &AppHandle, project: &Path) -> bool {
  if !is_temporary(app, project) {
    return false;
  }
  if take_unsaved(app) {
    match move_to_projects(app, project) {
      Ok(kept) => {
        super::super::project_name::settle_name(app, &kept);
      }
      Err(error) => eprintln!("Could not keep an unsaved screenshot: {error}"),
    }
    return true;
  }
  discard(app, project)
}

pub(crate) fn sweep(app: &AppHandle) {
  if let Ok(folder) = folder(app) {
    let _ = std::fs::remove_dir_all(folder);
  }
}

/// Records whether the open screenshot holds work kept nowhere else: a
/// capture not also on the clipboard, or a change since it was opened,
/// copied or exported.
pub(crate) fn set_unsaved(app: &AppHandle, unsaved: bool) {
  app
    .state::<EditorState>()
    .screenshot_unsaved
    .store(unsaved, Ordering::Release);
}

fn take_unsaved(app: &AppHandle) -> bool {
  app
    .state::<EditorState>()
    .screenshot_unsaved
    .swap(false, Ordering::AcqRel)
}

/// Moves the temporary `project` into the projects folder, remembered in
/// the library, and returns its manifest there.
fn move_to_projects(app: &AppHandle, project: &Path) -> Result<PathBuf, String> {
  let directory = crate::projects_folder::projects_directory(app)?;
  std::fs::create_dir_all(&directory)
    .map_err(|error| format!("The projects folder could not be created: {error}"))?;
  let kept = screenwide_project::move_into(project, &directory, 0, &mut |_| {})?;
  screenwide_project::library::remember(app, &kept);
  Ok(kept)
}

/// Save: the temporary screenshot open in the editor moves into Projects,
/// and the editor goes on with it there.
pub(crate) fn keep_open(app: &AppHandle) -> Result<(), String> {
  let state = app.state::<EditorState>();
  // Held through the move, so no save lands in the folder it is leaving.
  let mut artifact = state
    .screenshot
    .artifact
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  let Some(EditorArtifact::Screenshot { project, .. }) = artifact.as_mut() else {
    return Err("There is no screenshot to save".to_owned());
  };
  if !is_temporary(app, project) {
    return Ok(());
  }
  *project = move_to_projects(app, project)?;
  drop(artifact);
  set_unsaved(app, false);
  emit_snapshot(app, EditorKind::Screenshot);
  Ok(())
}

#[tauri::command]
pub fn keep_screenshot_project(app: AppHandle) -> Result<(), String> {
  keep_open(&app)
}
