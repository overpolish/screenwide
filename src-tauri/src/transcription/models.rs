// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::PathBuf;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use super::catalogue::{Model, ModelFile, Purpose, MODELS};
use super::{download, language};

const CHANGED_EVENT: &str = "transcription://changed";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
enum Status {
  Available,
  Downloaded,
  Downloading,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ModelState {
  description: String,
  id: &'static str,
  name: String,
  /// 0 to 1 while downloading.
  progress: Option<f32>,
  purpose: Purpose,
  size_bytes: u64,
  status: Status,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TranscriptionState {
  language: String,
  models: Vec<ModelState>,
  system_language: String,
}

/// Where models are kept: the computer's own data folder rather than one
/// that roams with a Windows profile, since they are large and can always
/// be fetched again.
pub(super) fn directory(app: &AppHandle) -> Result<PathBuf, String> {
  app
    .path()
    .app_local_data_dir()
    .map(|path| path.join("transcription").join("models"))
    .map_err(|error| error.to_string())
}

pub(super) fn file_path(app: &AppHandle, file: &ModelFile) -> Result<PathBuf, String> {
  directory(app).map(|directory| directory.join(file.name))
}

/// Where each of `model`'s files is kept, in the catalogue's order.
pub(crate) fn paths(app: &AppHandle, model: &Model) -> Result<Vec<PathBuf>, String> {
  model
    .files
    .iter()
    .map(|file| file_path(app, file))
    .collect()
}

/// Whether every file of `model` is on this computer. Each checksum was
/// checked as the file arrived and a download only takes the final name
/// once it passes, so the size alone tells a whole file from anything else.
pub(crate) fn is_downloaded(app: &AppHandle, model: &Model) -> bool {
  model.files.iter().all(|file| {
    file_path(app, file)
      .and_then(|path| std::fs::metadata(path).map_err(|error| error.to_string()))
      .is_ok_and(|metadata| metadata.len() == file.size_bytes)
  })
}

/// The model `purpose` runs with: its own, or for transcribing, failing that
/// any other transcribing model on this computer, more slowly perhaps but
/// never not at all.
pub(crate) fn usable(app: &AppHandle, purpose: Purpose) -> Option<&'static Model> {
  let (own, others): (Vec<_>, Vec<_>) = MODELS.iter().partition(|model| model.purpose == purpose);
  own
    .into_iter()
    .chain(
      others
        .into_iter()
        .filter(|model| purpose.transcribes() && model.purpose.transcribes()),
    )
    .find(|model| is_downloaded(app, model))
}

pub(crate) fn state(app: &AppHandle) -> TranscriptionState {
  let models = MODELS
    .iter()
    .map(|model| {
      let progress = download::progress(model.id);
      let status = if progress.is_some() {
        Status::Downloading
      } else if is_downloaded(app, model) {
        Status::Downloaded
      } else {
        Status::Available
      };
      ModelState {
        description: model.purpose.description(),
        id: model.id,
        name: model.purpose.name(),
        progress,
        purpose: model.purpose,
        size_bytes: model.size_bytes(),
        status,
      }
    })
    .collect();
  TranscriptionState {
    language: language::current(),
    models,
    system_language: language::system(),
  }
}

/// Tells every window, download progress included, so each shows one state.
pub(super) fn changed(app: &AppHandle) {
  let _ = app.emit(CHANGED_EVENT, state(app));
}

pub(super) fn remove(app: &AppHandle, model: &Model) -> Result<(), String> {
  for path in paths(app, model)? {
    match std::fs::remove_file(path) {
      Ok(()) => {}
      Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
      Err(error) => return Err(format!("Could not remove the model: {error}")),
    }
  }
  Ok(())
}
