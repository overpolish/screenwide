// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

//! The voice the teleprompter follows: the microphone heard with
//! dictation's model, on the device, its words told to the teleprompter's
//! page as they are heard, which finds the place in the script they read.

use std::sync::{Mutex, PoisonError};

use screenwide_dictation::{Event, Session};
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use ts_rs::TS;

use crate::dictation::island::DictationProblem;

use super::window::LABEL;

const EVENT: &str = "teleprompter://voice";

static LISTENING: Mutex<Option<Session>> = Mutex::new(None);

/// What the voice tells the teleprompter as it is heard.
#[derive(Clone, Debug, Serialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum TeleprompterVoice {
  /// The model is loaded and hears what is said, everything said since
  /// listening began included.
  Ready,
  /// Every word heard since listening began, the last few still settling.
  Heard { words: Vec<String> },
  /// Listening stopped: the microphone or the model failed.
  Failed,
}

/// Why the voice cannot be followed.
#[derive(Clone, Copy, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum TeleprompterVoiceProblem {
  /// Dictation's model is not downloaded.
  NoModel,
  /// Screenwide may not use the microphone.
  NoMicrophone,
  Failed,
}

/// Starts following the voice, in place of any listening before.
#[tauri::command]
pub fn start_teleprompter_voice(app: AppHandle) -> Result<(), TeleprompterVoiceProblem> {
  if !crate::permissions::cached_snapshot(&app).microphone.granted {
    return Err(TeleprompterVoiceProblem::NoMicrophone);
  }
  let setup = crate::dictation::listening_setup(&app).map_err(|problem| match problem {
    DictationProblem::NoModel => TeleprompterVoiceProblem::NoModel,
    DictationProblem::NoMicrophone => TeleprompterVoiceProblem::NoMicrophone,
    _ => TeleprompterVoiceProblem::Failed,
  })?;
  stop();
  let emitter = app.clone();
  let session = Session::start(setup, None, move |event| {
    let told = match event {
      Event::Ready => TeleprompterVoice::Ready,
      Event::Heard(text) => TeleprompterVoice::Heard {
        words: text.words.into_iter().map(|word| word.text).collect(),
      },
      Event::Failed => TeleprompterVoice::Failed,
      Event::Level(_) => return,
    };
    let _ = emitter.emit_to(LABEL, EVENT, told);
  });
  *LISTENING.lock().unwrap_or_else(PoisonError::into_inner) = Some(session);
  Ok(())
}

#[tauri::command]
pub fn stop_teleprompter_voice() {
  stop();
}

/// Stops listening, dropping what was heard. The session finishes on a
/// thread of its own, which waits for its helper to let go.
pub(super) fn stop() {
  let session = LISTENING
    .lock()
    .unwrap_or_else(PoisonError::into_inner)
    .take();
  if let Some(session) = session {
    std::thread::spawn(move || {
      let _ = session.cancel();
    });
  }
}
