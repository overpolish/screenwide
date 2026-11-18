// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How notes end: finished, thrown away, stopped for their project to be
//! deleted, or never started. Finishing takes a while, as the voices are told
//! apart over the whole call, and the notes are still written meanwhile, so
//! their project counts as being taken until it is done: a name given in
//! that time waits in the manifest, as one given to a project open in the
//! Editor does, and the folder takes it once the notes are written a last
//! time.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use tauri::{AppHandle, Emitter};

use super::island::{self, NotesIslandState, NotesProblem};
use super::{now_ms, taking, NOTES_FILE};
use crate::app_windows::WindowLabel;

/// Tells the projects window that notes it may be showing moved, from the
/// manifest first given to the one they settled at.
const MOVED_EVENT: &str = "notes://moved";

/// The manifests of the projects whose notes are being finished.
static FINISHING: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());

fn finishing() -> std::sync::MutexGuard<'static, Vec<PathBuf>> {
  FINISHING.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Whether the notes of the project whose manifest is `project` are being
/// finished.
pub(super) fn is_finishing(project: &Path) -> bool {
  finishing().iter().any(|finishing| finishing == project)
}

/// Stops listening and finishes the notes on a thread of their own, the
/// island saying so meanwhile.
pub(crate) fn stop(app: &AppHandle) {
  let Some(taken) = taking().take() else {
    return;
  };
  finishing().push(taken.project.clone());
  island::tell(app, NotesIslandState::Finishing);
  let app = app.clone();
  std::thread::spawn(move || {
    let finished = taken.session.finish();
    let duration_ms = now_ms().saturating_sub(taken.started_ms);
    let mut title = None;
    let _ = screenwide_project::update(&taken.project, |manifest| {
      title.clone_from(&manifest.title);
      if let Some(notes) = manifest.notes.as_mut() {
        notes.duration_ms = Some(duration_ms);
      }
      true
    });
    if let Some(title) = &title {
      if let Err(error) = screenwide_notes::retitle(&taken.notes, title) {
        eprintln!("Could not head the notes with their new name: {error}");
      }
    }
    finishing().retain(|finishing| *finishing != taken.project);
    let project = crate::editor::settle_name(&app, &taken.project);
    if project != taken.project {
      let _ = app.emit_to(
        WindowLabel::Projects.as_str(),
        MOVED_EVENT,
        (&taken.project, &project),
      );
    }
    screenwide_project::library::notify(&app);
    match finished {
      Ok(()) => island::show_for_a_moment(
        &app,
        NotesIslandState::Saved {
          file: project
            .with_file_name(NOTES_FILE)
            .to_string_lossy()
            .into_owned(),
        },
      ),
      Err(error) => {
        eprintln!("Could not finish the notes: {error}");
        island::show_for_a_moment(
          &app,
          NotesIslandState::Problem {
            problem: NotesProblem::Failed,
          },
        );
      }
    }
  });
}

/// Stops listening and sets the notes aside in Recently Deleted, where they
/// can still be brought back; the island goes at once.
pub(crate) fn discard(app: &AppHandle) {
  let Some(taken) = taking().take() else {
    return;
  };
  island::withdraw(app);
  let app = app.clone();
  std::thread::spawn(move || {
    if let Err(error) = taken.session.abandon() {
      eprintln!("Notes thrown away stopped badly: {error}");
    }
    if let Err(error) = screenwide_project::delete(&app, &taken.project) {
      eprintln!("Could not set the thrown-away notes aside: {error}");
    }
    screenwide_project::library::notify(&app);
  });
}

/// Stops the notes being taken into the project whose manifest is
/// `project`, if they are, so it can be deleted or moved: the island goes,
/// and the session lets go on a thread of its own, its next write failing
/// once the folder has gone. Refuses while they are being finished, as they
/// are written a last time when that ends.
pub(crate) fn stop_for_deletion(app: &AppHandle, project: &Path) -> Result<(), String> {
  if is_finishing(project) {
    return Err(screenwide_i18n::t!("project-browser-notes-finishing"));
  }
  let taken = {
    let mut taking = taking();
    if !taking
      .as_ref()
      .is_some_and(|taking| taking.project == project)
    {
      return Ok(());
    }
    taking.take()
  };
  let Some(taken) = taken else {
    return Ok(());
  };
  island::withdraw(app);
  std::thread::spawn(move || {
    let _ = taken.session.abandon();
  });
  Ok(())
}

/// Notes whose models never loaded: listening stops, the project they were
/// to be written into goes, and the island says they could not be taken.
pub(super) fn not_started(app: &AppHandle, file: PathBuf, root: PathBuf) {
  let taken = {
    let mut taking = taking();
    if !taking.as_ref().is_some_and(|taking| taking.project == file) {
      return;
    }
    taking.take()
  };
  let Some(taken) = taken else {
    return;
  };
  island::show_for_a_moment(
    app,
    NotesIslandState::Problem {
      problem: NotesProblem::Failed,
    },
  );
  let app = app.clone();
  // Off the notes' own thread, which this was called on and which letting
  // the session go waits for.
  std::thread::spawn(move || {
    let _ = taken.session.abandon();
    let _ = std::fs::remove_dir_all(&root);
    if let Err(error) = screenwide_project::library::forget(&app, &file) {
      eprintln!("Could not forget the notes that never started: {error}");
    }
    screenwide_project::library::notify(&app);
  });
}
