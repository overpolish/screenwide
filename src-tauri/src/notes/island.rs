// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The island while notes are taken: that it is listening and since when,
//! then that it is telling the speakers apart, then where the notes were
//! saved, or why notes could not be taken. As with
//! dictation, the island shows what it is sent (`notes://island`); a
//! dictation meanwhile shows over it and hands back.

use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use ts_rs::TS;

use crate::app_windows::island::{IslandClaim, IslandSlot};

static ISLAND: IslandSlot<NotesIslandState> = IslandSlot::new(IslandClaim::Notes, "notes://island");

/// What the island shows of notes being taken.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, TS)]
#[serde(tag = "phase", rename_all = "camelCase")]
#[ts(export)]
pub enum NotesIslandState {
  /// Listening since `startedMs` (milliseconds since the Unix epoch);
  /// `loading` while the models load, what is heard meanwhile waiting for
  /// them.
  Taking {
    #[serde(rename = "startedMs")]
    #[ts(type = "number")]
    started_ms: u64,
    loading: bool,
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
  /// No microphone is connected, the chosen one or any to fall back to.
  NoMicrophoneConnected,
  /// Screenwide may not hear the computer's audio (macOS's Screen
  /// Recording).
  NoSystemAudio,
  /// The notes could not start; the details are in the log.
  NotStarted,
  /// Something else went wrong part-way; the details are in the log.
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

/// How long a saved note or a problem stays on the island.
const NOTICE_SHOWN: Duration = Duration::from_secs(5);

pub(super) fn tell(app: &AppHandle, state: NotesIslandState) {
  ISLAND.tell(app, state);
}

/// Notes are done with the island: it goes back to whatever else holds it,
/// or tucks away.
pub(super) fn withdraw(app: &AppHandle) {
  ISLAND.withdraw(app);
}

/// The notes' models have loaded: the island stops saying they are loading.
pub(super) fn loaded(app: &AppHandle) {
  if let Some(NotesIslandState::Taking {
    started_ms,
    loading: true,
  }) = ISLAND.current()
  {
    ISLAND.update(
      app,
      NotesIslandState::Taking {
        started_ms,
        loading: false,
      },
    );
  }
}

/// Shows `state` for a moment, unless something else is shown first.
pub(super) fn show_for_a_moment(app: &AppHandle, state: NotesIslandState) {
  ISLAND.tell_for(app, state, NOTICE_SHOWN);
}

#[tauri::command]
pub fn notes_island_state() -> Option<NotesIslandState> {
  ISLAND.current()
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
