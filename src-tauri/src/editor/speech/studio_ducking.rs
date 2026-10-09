// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Studio ducking: while the voice speaks, the singing in the system audio
//! gets out of its way entirely, and the music makes way as Auto volume's
//! ducking always does. A sung word competes with a spoken one far more than
//! an instrument does. It needs a music separation model downloaded first,
//! from Settings or the switch; with the switch off, or without the model,
//! the system audio makes way as a whole. The work is part of making the
//! system audio's file (`duck`).

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use ts_rs::TS;

use super::choice_file;
use crate::transcription::catalogue::{self, Purpose};
use crate::transcription::models;

/// The make of the choice file.
const FORMAT_VERSION: u16 = 1;

/// Whether the singing in the system audio gets out of the voice's way.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum StudioDucking {
  #[default]
  Off,
  On,
}

fn choice_path(project_folder: &Path, microphone: usize) -> PathBuf {
  project_folder.join(format!("studio-ducking-{microphone}.json"))
}

/// What the project says for the `microphone`th track. A recording starts
/// with Studio ducking on, until someone turns it off; it is heard only once
/// the system audio's file is made with it, which needs the model.
pub(super) fn choice(project_folder: &Path, microphone: usize) -> StudioDucking {
  choice_file::read(&choice_path(project_folder, microphone), FORMAT_VERSION)
    .unwrap_or(StudioDucking::On)
}

pub(super) fn keep(
  project_folder: &Path,
  microphone: usize,
  choice: StudioDucking,
) -> Result<(), String> {
  choice_file::write(
    &choice_path(project_folder, microphone),
    FORMAT_VERSION,
    choice,
  )
  .map_err(|error| format!("Could not keep the Studio ducking choice: {error}"))
}

/// The separation model, where it is downloaded.
pub(super) fn model(app: &AppHandle) -> Option<PathBuf> {
  let model = catalogue::for_purpose(Purpose::StudioDucking)?;
  if !models::is_downloaded(app, model) {
    return None;
  }
  models::paths(app, model).ok()?.into_iter().next()
}

/// The separation model, where the switch is on for the `microphone`th track
/// and the model downloaded: the system audio making way for it is then made
/// with its singing separated.
pub(super) fn wanted(app: &AppHandle, project_folder: &Path, microphone: usize) -> Option<PathBuf> {
  (choice(project_folder, microphone) == StudioDucking::On)
    .then(|| model(app))
    .flatten()
}
