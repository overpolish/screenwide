// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Studio ducking: while the voice speaks, the system audio and the songs
//! laid under the recording make way for it, their singing out of its way
//! entirely, as a sung word competes with a spoken one far more than an
//! instrument does. It needs the music separation model downloaded first,
//! from Settings or the switch; with the switch off, without the model, or
//! with the microphone not heard, nothing makes way. The system audio's
//! work is making its file (`duck`); the songs' is separating their singing
//! (`sound_library::separate_singing`).

use std::path::{Path, PathBuf};

use tauri::AppHandle;

use crate::transcription::models;
use screenwide_speech::catalogue::{self, Purpose};
pub(crate) use screenwide_speech::duck::{choice, keep, StudioDucking};

/// The separation model, where it is downloaded.
pub(crate) fn model(app: &AppHandle) -> Option<PathBuf> {
  let model = catalogue::for_purpose(Purpose::StudioDucking)?;
  if !models::is_downloaded(app, model) {
    return None;
  }
  models::paths(app, model).ok()?.into_iter().next()
}

/// The separation model, where the switch is on for the `microphone`th track
/// and the model downloaded: the system audio and the songs then make way
/// for it.
pub(crate) fn wanted(app: &AppHandle, project_folder: &Path, microphone: usize) -> Option<PathBuf> {
  (choice(project_folder, microphone) == StudioDucking::On)
    .then(|| model(app))
    .flatten()
}
