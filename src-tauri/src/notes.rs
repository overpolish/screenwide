// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Meeting notes, wired into the app. Started from the recording island's
//! Notes, they listen to the microphone and the computer's audio, keep no
//! audio, and write what was said into a Markdown file in a project of their
//! own as the call goes (`screenwide_notes`). The island says they are being
//! taken (`island`); Stop tells the speakers apart and saves them, and the
//! notes open in a window to read, edit and copy the path of (`view`). How
//! they end, finished, thrown away or stopped for their project to be
//! deleted, is `ending`.

mod ending;
pub(crate) mod island;
pub(crate) mod sensitive;
pub(crate) mod view;

pub(crate) use self::ending::{discard, stop, stop_for_deletion};

use std::sync::{Arc, Mutex, PoisonError};
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Local;
use screenwide_capture::listening::Listening;
use screenwide_capture::microphone_choice::{available_microphone, SYSTEM_DEFAULT_MICROPHONE};
use screenwide_capture::monitor::RecordingMonitor;
use screenwide_capture::{RecordingStatus, SystemAudioSelection};
use screenwide_notes::{Failure, NotesModels, Session, Setup};
use screenwide_project::{Manifest, NotesManifest};
use screenwide_speech::catalogue;
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
/// taken or finished, so they are not saved over, renamed or moved
/// meanwhile.
pub(crate) fn is_taking_into(project: &std::path::Path) -> bool {
  ending::is_finishing(project)
    || taking()
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
    NotesProblem::NotStarted
  };
  if !models::can_take_notes(app) {
    return Err(NotesProblem::NoModel);
  }
  let paths = models::paths(app, catalogue::notes()).map_err(failed)?;
  let [model, speakers, voices, ..] = paths.as_slice() else {
    return Err(failed("The notes models are not as catalogued".to_owned()));
  };
  let vad = models::vad_model(app).map_err(failed)?;
  // Notes always hear the microphone, as a call has two sides: the one
  // chosen, or the system's default while it is not connected.
  let chosen = options
    .microphone_id
    .unwrap_or_else(|| SYSTEM_DEFAULT_MICROPHONE.to_owned());
  let microphone_id = available_microphone(&chosen).map_err(|error| {
    eprintln!("Could not find a microphone for notes: {error}");
    NotesProblem::NoMicrophoneConnected
  })?;
  crate::microphone::session_started(app);
  // The island reads the microphone's level from the recording's monitor.
  // A recording running keeps it, and the notes hear on one of their own.
  let monitor = if crate::recording::snapshot(app).status == RecordingStatus::Idle {
    let monitor = crate::recording::confidence_monitor(app);
    monitor.configure(true, true, false);
    monitor
  } else {
    Arc::new(RecordingMonitor::default())
  };
  let started_at = Local::now().naive_local();
  let title = started_at.format("Notes %Y-%m-%d at %H.%M.%S").to_string();
  let project = crate::projects_folder::create(app, &title).map_err(failed)?;
  let notes = project.root.join(NOTES_FILE);
  let manifest = Manifest::notes(NotesManifest {
    file: NOTES_FILE.to_owned(),
    duration_ms: None,
    not_private: Vec::new(),
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
      microphone_id: Some(microphone_id),
      system_audio: SystemAudioSelection {
        application_ids: options.system_audio_application_ids,
        enabled: true,
        process_ids: options.system_audio_process_ids,
      },
      monitor,
      on_failure: Arc::new(move |error: String| {
        eprintln!("Listening for notes failed: {error}");
        stop(&reporter);
      }),
    },
    path: notes.clone(),
    title,
  };
  // The island goes up once listening has begun, saying the models load.
  // Their loading, or failing to, can end first, so either waits for the
  // island to say the notes are being taken before it is told.
  let (told, wait_told) = std::sync::mpsc::channel::<()>();
  let wait_told = Arc::new(Mutex::new(wait_told));
  let wait_ready = Arc::clone(&wait_told);
  let ready = app.clone();
  let (failing, file, root) = (app.clone(), project.file.clone(), project.root.clone());
  let started_ms = now_ms();
  let session = Session::start(
    setup,
    move || {
      let _ = wait_ready
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .recv();
      island::loaded(&ready);
    },
    move |failure| match failure {
      Failure::Readying(error) => {
        eprintln!("Could not ready the notes: {error}");
        let _ = wait_told
          .lock()
          .unwrap_or_else(PoisonError::into_inner)
          .recv();
        ending::not_started(&failing, file, root);
      }
      Failure::Taking(error) => eprintln!("Taking notes failed: {error}"),
    },
  )
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
  // The island shows the notes from here, claimed before the controls let
  // go, as a recording's start does, so it never tucks away in between.
  // Escape no longer takes it.
  island::tell(
    app,
    NotesIslandState::Taking {
      started_ms,
      loading: true,
    },
  );
  if let Err(error) = crate::app_windows::close_recording_controls(app) {
    eprintln!("Could not put the recording controls away for notes: {error}");
  }
  let _ = told.send(());
  Ok(())
}

fn now_ms() -> u64 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map_or(0, |since| {
      u64::try_from(since.as_millis()).unwrap_or(u64::MAX)
    })
}
