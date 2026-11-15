// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The island while dictating. It comes out at the top of the screen with
//! the words as they are heard, says so while the last of them are heard and
//! pasted, and says why when dictation cannot start or paste. The island
//! never takes the keyboard, so the app being dictated into keeps it.
//!
//! As with Text Recognition, the island shows what it is sent: each change
//! of state (`dictation://island`), and the microphone's level on its own
//! (`dictation://level`), as that changes many times a second.

use std::sync::{Mutex, PoisonError};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use ts_rs::TS;

use crate::app_windows::island_placement::IslandPlacement;
use crate::app_windows::{self, WindowLabel};

const EVENT: &str = "dictation://island";
const LEVEL_EVENT: &str = "dictation://level";

/// What the island shows of a dictation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, TS)]
#[serde(tag = "phase", rename_all = "camelCase")]
#[ts(export)]
pub enum DictationIslandState {
  /// Listening: the words heard so far, `settled` no longer changing and
  /// `unsettled` still being heard.
  Listening { settled: String, unsettled: String },
  /// Stopped: the last words are being heard and pasted.
  Finishing,
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
  /// Transcription is set to a language other than English.
  NotEnglish,
  /// Screenwide may not use the microphone.
  NoMicrophone,
  /// Screenwide may not press keys in other apps (macOS's Accessibility).
  NoAccessibility,
  /// A password field had the keyboard.
  PasswordField,
  /// The app in front runs as administrator.
  Administrator,
  /// Something else went wrong.
  Failed,
}

impl DictationProblem {
  /// Whether Settings is where it is put right.
  const fn settings_fix(self) -> bool {
    matches!(
      self,
      Self::NoModel | Self::NotEnglish | Self::NoMicrophone | Self::NoAccessibility
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
  let result = if crate::recording::is_idle(app) && !app_windows::is_recording_ui_visible() {
    app_windows::island::hide(app)
  } else {
    app_windows::island::hand_over(app, IslandPlacement::Top)
  };
  if let Err(error) = result {
    eprintln!("Could not put the island away after dictation: {error}");
  }
}

/// Whether the island shows `problem` still.
pub(super) fn shows(problem: DictationProblem) -> bool {
  *SHOWN.lock().unwrap_or_else(PoisonError::into_inner)
    == Some(DictationIslandState::Problem { problem })
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
