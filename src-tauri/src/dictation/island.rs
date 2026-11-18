// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The island while dictating. It comes out at the top of the screen with
//! the words as they are heard, says so while the last of them are heard and
//! pasted, confirms words added to or removed from the word list, and says
//! why when dictation cannot start or paste. The island never takes the
//! keyboard, so the app being dictated into keeps it.
//!
//! As with Text Recognition, the island shows what it is sent: each change
//! of state (`dictation://island`), and the microphone's level on its own
//! (`dictation://level`), as that changes many times a second.

use std::sync::{Mutex, PoisonError};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use ts_rs::TS;

use super::taught::DictationWordChange;
use crate::app_windows::island_placement::IslandPlacement;
use crate::app_windows::{self, WindowLabel};

const EVENT: &str = "dictation://island";
const LEVEL_EVENT: &str = "dictation://level";

/// What the island shows of a dictation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, TS)]
#[serde(tag = "phase", rename_all = "camelCase")]
#[ts(export)]
pub enum DictationIslandState {
  /// Listening: the text so far, its commands carried out, `settled` no
  /// longer changing and `unsettled` still being heard; the words this
  /// dictation has added to or removed from the word list; whether the text
  /// comes at each pause rather than as it is spoken; and whether the model
  /// is still loading, while what is said waits to be heard.
  Listening {
    settled: String,
    unsettled: String,
    taught: Vec<DictationWordChange>,
    #[serde(rename = "atPauses")]
    at_pauses: bool,
    loading: bool,
  },
  /// Stopped: the last words are being heard and pasted.
  Finishing,
  /// Pasted, with words added or removed: what came of each, for a moment.
  Taught { taught: Vec<DictationWordChange> },
  /// Dictation could not start, or could not paste.
  Problem { problem: DictationProblem },
}

/// Why dictation could not start or paste.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum DictationProblem {
  /// The model is not downloaded.
  NoModel,
  /// Screenwide may not use the microphone.
  NoMicrophone,
  /// Screenwide may not press keys in other apps (macOS's Accessibility).
  NoAccessibility,
  /// A password field had the keyboard.
  PasswordField,
  /// The app in front runs as administrator; the text is on the clipboard.
  Administrator,
  /// Nothing that takes text had the keyboard; the text is on the
  /// clipboard.
  OnClipboard,
  /// Something else went wrong.
  Failed,
}

impl DictationProblem {
  /// Whether Settings is where it is put right.
  const fn settings_fix(self) -> bool {
    matches!(
      self,
      Self::NoModel | Self::NoMicrophone | Self::NoAccessibility
    )
  }
}

/// What a press on the island does.
#[derive(Clone, Copy, Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum DictationIslandAction {
  /// Stops listening and pastes what was heard.
  Stop,
  /// Stops listening and drops what was heard.
  Cancel,
  /// Opens Settings where a problem is put right.
  OpenSettings,
  /// Puts a problem away.
  Close,
}

/// What the island was last told; none while dictation is not on it.
static SHOWN: Mutex<Option<DictationIslandState>> = Mutex::new(None);

pub(super) fn tell(app: &AppHandle, state: DictationIslandState) {
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
      eprintln!("Could not show the island for dictation: {error}");
    }
  }
}

pub(super) fn level(app: &AppHandle, decibels: f32) {
  let _ = app.emit_to(WindowLabel::Island.as_str(), LEVEL_EVENT, decibels);
}

/// Dictation is done with the island: it goes, unless the recording or its
/// controls keep it, and then hands over to them.
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
    None::<DictationIslandState>,
  );
  let keeps = !crate::recording::is_idle(app)
    || app_windows::is_recording_ui_visible()
    || crate::notes::island::is_shown();
  let result = if !keeps {
    app_windows::island::hide(app)
  } else {
    app_windows::island::hand_over(app, IslandPlacement::Top)
  };
  if let Err(error) = result {
    eprintln!("Could not put the island away after dictation: {error}");
  }
}

/// Whether the island shows `state` still.
fn shows(state: &DictationIslandState) -> bool {
  SHOWN
    .lock()
    .unwrap_or_else(PoisonError::into_inner)
    .as_ref()
    == Some(state)
}

/// How long a problem stays on the island unless it is put away first.
const PROBLEM_SHOWN: Duration = Duration::from_secs(5);
/// How long the words added or removed stay on the island after a paste.
const TAUGHT_SHOWN: Duration = Duration::from_secs(3);

pub(super) fn show_problem(app: &AppHandle, problem: DictationProblem) {
  show_for(
    app,
    DictationIslandState::Problem { problem },
    PROBLEM_SHOWN,
  );
}

pub(super) fn show_taught(app: &AppHandle, taught: Vec<DictationWordChange>) {
  show_for(app, DictationIslandState::Taught { taught }, TAUGHT_SHOWN);
}

/// Shows `state` for `shown`, unless something else is shown first.
fn show_for(app: &AppHandle, state: DictationIslandState, shown: Duration) {
  tell(app, state.clone());
  let app = app.clone();
  std::thread::spawn(move || {
    std::thread::sleep(shown);
    if shows(&state) {
      withdraw(&app);
    }
  });
}

#[tauri::command]
pub fn dictation_island_state() -> Option<DictationIslandState> {
  SHOWN.lock().unwrap_or_else(PoisonError::into_inner).clone()
}

#[tauri::command]
pub fn dictation_island_action(app: AppHandle, action: DictationIslandAction) {
  match action {
    DictationIslandAction::Stop => super::finish(&app),
    DictationIslandAction::Cancel => super::cancel(&app),
    DictationIslandAction::OpenSettings => {
      let fixed_in_settings = SHOWN
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .as_ref()
        .is_some_and(|state| {
          matches!(state, DictationIslandState::Problem { problem } if problem.settings_fix())
        });
      withdraw(&app);
      if fixed_in_settings {
        crate::settings::show_section(&app, "transcription");
      }
    }
    DictationIslandAction::Close => withdraw(&app),
  }
}
