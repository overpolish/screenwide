// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::PathBuf;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use super::{download, language, words};
use screenwide_dictation::{phrases, Does};
use screenwide_speech::catalogue::{self, Model, ModelFile, Purpose, Writes, MODELS};

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
  /// For dictation's model: when its words appear.
  writes: Option<Writes>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TranscriptionState {
  language: String,
  models: Vec<ModelState>,
  system_language: String,
  /// The word list, in order.
  words: Vec<String>,
  /// The spoken commands of the language dictation is in, none in a
  /// language without them.
  dictation_commands: Vec<DictationCommand>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DictationCommand {
  said: &'static str,
  does: Does,
  /// The mark a punctuation command types.
  mark: Option<char>,
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

/// The model `purpose` runs with, once it is on this computer.
pub(crate) fn usable(app: &AppHandle, purpose: Purpose) -> Option<&'static Model> {
  catalogue::for_purpose(purpose).filter(|model| is_downloaded(app, model))
}

/// The voice activity model, which finds where speech is: bundled as a
/// resource, or beside the program when it runs unbundled during
/// development.
pub(crate) fn vad_model(app: &AppHandle) -> Result<PathBuf, String> {
  const NAME: &str = "silero-vad.bin";
  let bundled = app
    .path()
    .resource_dir()
    .ok()
    .map(|resources| resources.join("models").join(NAME));
  let beside = std::env::current_exe()
    .ok()
    .and_then(|program| program.parent().map(|folder| folder.join(NAME)));
  bundled
    .into_iter()
    .chain(beside)
    .find(|path| path.is_file())
    .ok_or_else(|| "The voice activity model is missing from this installation".to_owned())
}

/// The model dictation takes in the language transcription is set to, and
/// when its words appear.
pub(crate) fn dictation() -> (&'static Model, Writes) {
  catalogue::dictation(language::resolved(&language::current()).as_deref())
}

/// The model that corrects dictation in the language transcription is set
/// to, if it is one the model corrects.
pub(crate) fn dictation_cleanup() -> Option<&'static Model> {
  catalogue::dictation_cleanup(language::resolved(&language::current()).as_deref())
}

/// Each model in the catalogue's order, dictation's being the one for the
/// language transcription is set to, which may be the Transcripts model
/// listed again under Dictation, and dictation's cleanup model only for a
/// language it corrects.
pub(crate) fn state(app: &AppHandle) -> TranscriptionState {
  let (dictation, writes) = dictation();
  let cleanup = dictation_cleanup();
  let mut models = Vec::new();
  for model in MODELS {
    if model.purpose == Purpose::DictationCleanup {
      if cleanup.is_some_and(|cleanup| cleanup.id == model.id) {
        models.push(model_state(app, model, model.purpose.description()));
      }
    } else if model.purpose != Purpose::Dictation {
      models.push(model_state(app, model, model.purpose.description()));
    } else if !models
      .iter()
      .any(|listed: &ModelState| listed.purpose == Purpose::Dictation)
    {
      let shared = dictation.purpose == Purpose::Transcripts;
      let mut listed = model_state(
        app,
        dictation,
        catalogue::dictation_description(writes, shared),
      );
      listed.name = Purpose::Dictation.name();
      listed.purpose = Purpose::Dictation;
      listed.writes = Some(writes);
      models.push(listed);
    }
  }
  let dictation_commands = phrases(language::resolved(&language::current()).as_deref())
    .listed()
    .map(|(said, does, mark)| DictationCommand { said, does, mark })
    .collect();
  TranscriptionState {
    language: language::current(),
    models,
    system_language: language::system(),
    words: words::current(),
    dictation_commands,
  }
}

fn model_state(app: &AppHandle, model: &'static Model, description: String) -> ModelState {
  let progress = download::progress(model.id);
  let status = if progress.is_some() {
    Status::Downloading
  } else if is_downloaded(app, model) {
    Status::Downloaded
  } else {
    Status::Available
  };
  ModelState {
    description,
    id: model.id,
    name: model.purpose.name(),
    progress,
    purpose: model.purpose,
    size_bytes: model.size_bytes(),
    status,
    writes: None,
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

/// Deletes the files of models the catalogue no longer has, so a model
/// replaced by another takes no room once the app updates.
pub(crate) fn remove_retired(app: &AppHandle) {
  let Ok(directory) = directory(app) else {
    return;
  };
  for name in catalogue::RETIRED_FILES {
    match std::fs::remove_file(directory.join(name)) {
      Ok(()) => {}
      Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
      Err(error) => eprintln!("Could not remove the retired model {name}: {error}"),
    }
  }
}
