// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Meeting notes, wired into the app. Started from the recording island's
//! Notes, they listen to the microphone and the computer's audio, keep no
//! audio, and write what was said into a Markdown file in a project of their
//! own as the call goes (`screenwide_notes`). The island says they are being
//! taken (`island`); Stop tells the speakers apart and saves them, and the
//! notes open in a window to read, edit and copy the path of (`view`).

pub(crate) mod island;
pub(crate) mod view;

use std::sync::{Arc, Mutex, PoisonError};
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Local;
use screenwide_capture::listening::Listening;
use screenwide_capture::monitor::RecordingMonitor;
use screenwide_capture::SystemAudioSelection;
use screenwide_notes::{Failure, NotesModels, Session, Setup};
use screenwide_project::{Manifest, NotesManifest};
use screenwide_speech::catalogue::Purpose;
use serde::Deserialize;
use tauri::AppHandle;
use ts_rs::TS;

use self::island::{NotesIslandState, NotesProblem};
use crate::transcription::{language, models, words};

/// The notes' Markdown file, in its project's folder.
pub(crate) const NOTES_FILE: &str = "notes.md";

/// What to listen to, from the recording island's inputs.
#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StartNotesOptions {
  /// The microphone's id; none for the system's default.
  pub microphone_id: Option<String>,
  /// The apps whose sound is heard; none for all of the computer's audio.
  pub system_audio_application_ids: Vec<String>,
  #[ts(type = "number[]")]
  pub system_audio_process_ids: Vec<u32>,
}

/// Notes being taken: the session, and the project they are saved in.
struct Taking {
  session: Session,
  project: std::path::PathBuf,
  notes: std::path::PathBuf,
  started_ms: u64,
}

static TAKING: Mutex<Option<Taking>> = Mutex::new(None);

fn taking() -> std::sync::MutexGuard<'static, Option<Taking>> {
  TAKING.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Whether notes are being taken.
pub(crate) fn is_taking() -> bool {
  taking().is_some()
}

/// Whether the notes of the project whose manifest is `project` are being
/// taken, so they are not saved over meanwhile.
pub(crate) fn is_taking_into(project: &std::path::Path) -> bool {
  taking()
    .as_ref()
    .is_some_and(|taking| taking.project == project)
}

#[tauri::command]
pub fn start_notes(app: AppHandle, options: StartNotesOptions) {
  if is_taking() {
    return;
  }
  // Opening the computer's audio waits on the system, which must not hold up
  // the thread the windows draw on.
  std::thread::spawn(move || {
    if let Err(problem) = start(&app, options) {
      island::show_for_a_moment(&app, NotesIslandState::Problem { problem });
    }
  });
}
#[tauri::command]
pub fn stop_notes(app: AppHandle) {
  stop(&app);
}

fn start(app: &AppHandle, options: StartNotesOptions) -> Result<(), NotesProblem> {
  let permissions = crate::permissions::cached_snapshot(app);
  if !permissions.microphone.granted {
    return Err(NotesProblem::NoMicrophone);
  }
  if !permissions.screen_recording.granted {
    return Err(NotesProblem::NoSystemAudio);
  }
  let failed = |error: String| {
    eprintln!("Could not take notes: {error}");
    NotesProblem::Failed
  };
  let model = models::usable(app, Purpose::Notes).ok_or(NotesProblem::NoModel)?;
  let paths = models::paths(app, model).map_err(failed)?;
  let [model, speakers, voices] = paths.as_slice() else {
    return Err(failed("The notes models are not as catalogued".to_owned()));
  };
  let vad = models::vad_model(app).map_err(failed)?;
  let started_at = Local::now().naive_local();
  let title = started_at.format("Notes %Y-%m-%d at %H.%M.%S").to_string();
  let project = crate::projects_folder::create(app, &title).map_err(failed)?;
  let notes = project.root.join(NOTES_FILE);
  let manifest = Manifest::notes(NotesManifest {
    file: NOTES_FILE.to_owned(),
    duration_ms: None,
  });
  screenwide_project::write(&project.file, &manifest).map_err(failed)?;
  let reporter = app.clone();
  let setup = Setup {
    models: NotesModels {
      model: model.clone(),
      vad,
      speakers: speakers.clone(),
      voices: voices.clone(),
    },
    language: language::resolved(&language::current()),
    words: words::current(),
    listening: Listening {
      microphone_id: options.microphone_id,
      system_audio: SystemAudioSelection {
        application_ids: options.system_audio_application_ids,
        enabled: true,
        process_ids: options.system_audio_process_ids,
      },
      monitor: Arc::new(RecordingMonitor::default()),
      on_failure: Arc::new(move |error: String| {
        eprintln!("Listening for notes failed: {error}");
        stop(&reporter);
      }),
    },
    path: notes.clone(),
    title,
  };
  // The island goes up once listening has begun, before the models have
  // loaded. A failure to load them can arrive first, so it waits for the
  // island to say the notes are being taken before saying they failed.
  let (told, wait_told) = std::sync::mpsc::channel::<()>();
  let (failing, file, root) = (app.clone(), project.file.clone(), project.root.clone());
  let started_ms = now_ms();
  let session = Session::start(setup, move |failure| match failure {
    Failure::Readying(error) => {
      eprintln!("Could not ready the notes: {error}");
      let _ = wait_told.recv();
      not_started(&failing, file, root);
    }
    Failure::Taking(error) => eprintln!("Taking notes failed: {error}"),
  })
  .map_err(|error| {
    project.remove();
    failed(error)
  })?;
  *taking() = Some(Taking {
    session,
    project: project.file.clone(),
    notes,
    started_ms,
  });
  screenwide_project::library::remember(app, &project.file);
  // The controls are put away as a recording's start puts them away: the
  // island shows the notes from here, and Escape no longer takes it.
  if let Err(error) = crate::app_windows::close_recording_controls(app) {
    eprintln!("Could not put the recording controls away for notes: {error}");
  }
  island::tell(app, NotesIslandState::Taking { started_ms });
  let _ = told.send(());
  Ok(())
}

/// Notes whose models never loaded: listening stops, the project they were
/// to be written into goes, and the island says they could not be taken.
fn not_started(app: &AppHandle, file: std::path::PathBuf, root: std::path::PathBuf) {
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

/// Stops listening and finishes the notes on a thread of their own, the
/// island saying so meanwhile.
pub(crate) fn stop(app: &AppHandle) {
  let Some(taken) = taking().take() else {
    return;
  };
  island::tell(app, NotesIslandState::Finishing);
  let app = app.clone();
  std::thread::spawn(move || {
    let finished = taken.session.finish();
    let duration_ms = now_ms().saturating_sub(taken.started_ms);
    let _ = screenwide_project::update(&taken.project, |manifest| {
      if let Some(notes) = manifest.notes.as_mut() {
        notes.duration_ms = Some(duration_ms);
      }
      true
    });
    screenwide_project::library::notify(&app);
    match finished {
      Ok(()) => island::show_for_a_moment(
        &app,
        NotesIslandState::Saved {
          file: taken.notes.to_string_lossy().into_owned(),
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

fn now_ms() -> u64 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map_or(0, |since| {
      u64::try_from(since.as_millis()).unwrap_or(u64::MAX)
    })
}
