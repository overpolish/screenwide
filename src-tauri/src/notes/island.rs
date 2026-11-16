// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The island while notes are taken: that it is listening and since when,
//! then that it is telling the speakers apart, then where the notes were
//! saved, or why notes could not be taken. As with
//! dictation, the island shows what it is sent (`notes://island`); a
//! dictation meanwhile shows over it and hands back.

use std::sync::{Mutex, PoisonError};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use ts_rs::TS;

use crate::app_windows::island_placement::IslandPlacement;
use crate::app_windows::{self, WindowLabel};

const EVENT: &str = "notes://island";

/// What the island shows of notes being taken.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, TS)]
#[serde(tag = "phase", rename_all = "camelCase")]
#[ts(export)]
pub enum NotesIslandState {
  /// Listening since `startedMs` (milliseconds since the Unix epoch).
  Taking {
    #[serde(rename = "startedMs")]
    #[ts(type = "number")]
    started_ms: u64,
  },
  /// Stopped: the last lines are written and the speakers told apart.
  Finishing,
  /// Saved, for a moment, with the notes' Markdown file.
  Saved { file: String },
  /// Notes could not be taken, or not finished.
  Problem { problem: NotesProblem },
}

/// Why notes could not be taken.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum NotesProblem {
  /// The models are not downloaded.
  NoModel,
  /// Screenwide may not use the microphone.
  NoMicrophone,
  /// Screenwide may not hear the computer's audio (macOS's Screen
  /// Recording).
  NoSystemAudio,
  /// Something else went wrong; the details are in the log.
  Failed,
}

/// What the island's buttons ask.
#[derive(Clone, Copy, Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum NotesIslandAction {
  Stop,
  /// Stop and throw the notes away, to Recently Deleted.
  Discard,
  /// Open Settings on the models, for a problem it fixes there.
  OpenSettings,
  /// Show the notes just saved.
  Open,
  Close,
}

static SHOWN: Mutex<Option<NotesIslandState>> = Mutex::new(None);

/// How long a saved note or a problem stays on the island.
const NOTICE_SHOWN: Duration = Duration::from_secs(5);

pub(super) fn tell(app: &AppHandle, state: NotesIslandState) {
  let mut shown = SHOWN.lock().unwrap_or_else(PoisonError::into_inner);
  let first = shown.is_none();
  if shown.as_ref() == Some(&state) {
    return;
  }
  *shown = Some(state.clone());
  drop(shown);
  let _ = app.emit_to(WindowLabel::Island.as_str(), EVENT, Some(state));
  if first {
    if let Err(error) = app_windows::island::place(app, IslandPlacement::Top)
      .and_then(|()| app_windows::island::show(app))
    {
      eprintln!("Could not show the island for notes: {error}");
    }
  }
}

/// Notes are done with the island: it goes, unless a recording, its
/// controls or a dictation keep it.
pub(super) fn withdraw(app: &AppHandle) {
  if SHOWN
    .lock()
    .unwrap_or_else(PoisonError::into_inner)
    .take()
    .is_none()
  {
    return;
  }
  let _ = app.emit_to(
    WindowLabel::Island.as_str(),
    EVENT,
    None::<NotesIslandState>,
  );
  let keeps = !crate::recording::is_idle(app)
    || app_windows::is_recording_ui_visible()
    || crate::dictation::island::dictation_island_state().is_some();
  let result = if keeps {
    app_windows::island::hand_over(app, IslandPlacement::Top)
  } else {
    app_windows::island::hide(app)
  };
  if let Err(error) = result {
    eprintln!("Could not put the island away after notes: {error}");
  }
}

/// Shows `state` for a moment, unless something else is shown first.
pub(super) fn show_for_a_moment(app: &AppHandle, state: NotesIslandState) {
  tell(app, state.clone());
  let app = app.clone();
  std::thread::spawn(move || {
    std::thread::sleep(NOTICE_SHOWN);
    if SHOWN
      .lock()
      .unwrap_or_else(PoisonError::into_inner)
      .as_ref()
      == Some(&state)
    {
      withdraw(&app);
    }
  });
}

/// Whether the island shows notes, which keeps it out when a dictation
/// hands back.
pub(crate) fn is_shown() -> bool {
  SHOWN
    .lock()
    .unwrap_or_else(PoisonError::into_inner)
    .is_some()
}

#[tauri::command]
pub fn notes_island_state() -> Option<NotesIslandState> {
  SHOWN.lock().unwrap_or_else(PoisonError::into_inner).clone()
}

#[tauri::command]
pub fn notes_island_action(app: AppHandle, action: NotesIslandAction) {
  let shown = notes_island_state();
  match action {
    NotesIslandAction::Stop => super::stop(&app),
    NotesIslandAction::Discard => super::discard(&app),
    NotesIslandAction::OpenSettings => {
      withdraw(&app);
      crate::settings::show_section(&app, "transcription");
    }
    NotesIslandAction::Open => {
      withdraw(&app);
      if let Some(NotesIslandState::Saved { file }) = shown {
        if let Err(error) = super::view::open(&app, std::path::Path::new(&file)) {
          eprintln!("Could not open the notes: {error}");
        }
      }
    }
    NotesIslandAction::Close => withdraw(&app),
  }
}
