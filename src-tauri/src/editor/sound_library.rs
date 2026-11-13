// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The sound library as the editor uses it: songs and sound effects added
//! from disk, the copies a project carries of those its clips play, and how
//! the export and the preview find their files. Its commands are registered
//! here rather than in the app's one list (`invoke_handler.rs`):
//! [`with_commands`] routes them past it.

use std::path::{Path, PathBuf};

use screenwide_sound_track::store::{self, Sound};
use screenwide_sound_track::{AudioClipKind, RecordingAudioClip, SoundFile, SoundTrack};
use screenwide_timeline_edit::TimelinePlan;
use tauri::ipc::Invoke;
use tauri::{Emitter, Manager};

use super::{EditorArtifact, EditorState};

mod preview;
mod singing;
mod voice;

pub(crate) use singing::{ready_for_export, separate_singing};
pub(crate) use voice::ducks;
use voice::microphone;

const COMMANDS: [&str; 6] = [
  "browse_sounds",
  "import_sounds",
  "get_sound_library",
  "get_project_sounds",
  "get_sound_preview",
  "remove_sound",
];

/// `app_commands` with the sound library's commands beside them.
pub(crate) fn with_commands(
  app_commands: impl Fn(Invoke) -> bool + Send + Sync + 'static,
) -> impl Fn(Invoke) -> bool + Send + Sync + 'static {
  let sound_commands = sound_commands();
  move |invoke| {
    if COMMANDS.contains(&invoke.message.command()) {
      sound_commands(invoke)
    } else {
      app_commands(invoke)
    }
  }
}

fn sound_commands() -> impl Fn(Invoke) -> bool + Send + Sync + 'static {
  tauri::generate_handler![
    browse_sounds,
    import_sounds,
    get_sound_library,
    get_project_sounds,
    preview::get_sound_preview,
    remove_sound
  ]
}

/// Asks for songs or sound effects to add, in a picker parented to the
/// asking window. None where nothing was chosen.
#[tauri::command]
async fn browse_sounds(
  app: tauri::AppHandle,
  window: tauri::WebviewWindow,
) -> Result<Vec<PathBuf>, String> {
  let picked = tauri::async_runtime::spawn_blocking(move || {
    use tauri_plugin_dialog::DialogExt;

    app
      .dialog()
      .file()
      .set_title(screenwide_i18n::t!("dialog-choose-sounds"))
      .add_filter(
        screenwide_i18n::t!("dialog-sounds-filter"),
        &store::EXTENSIONS,
      )
      .set_parent(&window)
      .blocking_pick_files()
  })
  .await
  .map_err(|error| error.to_string())?;
  Ok(
    picked
      .unwrap_or_default()
      .into_iter()
      .filter_map(|path| path.into_path().ok())
      .collect(),
  )
}

/// The event every window hears when the library changes, so each reads it
/// again.
const LIBRARY_EVENT: &str = "editor://sound-library";

/// Adds each of `paths` that is a sound to the library and says what each
/// is, in order. Fails only where none could be added.
#[tauri::command]
async fn import_sounds(app: tauri::AppHandle, paths: Vec<PathBuf>) -> Result<Vec<Sound>, String> {
  let imported = tauri::async_runtime::spawn_blocking(move || {
    let mut sounds = Vec::new();
    let mut failure = None;
    for path in paths {
      match store::import(&path) {
        Ok(sound) => sounds.push(sound),
        Err(error) => {
          eprintln!(
            "Could not add {} to the sound library: {error}",
            path.display()
          );
          failure.get_or_insert(error);
        }
      }
    }
    match failure {
      Some(error) if sounds.is_empty() => Err(error),
      _ => Ok(sounds),
    }
  })
  .await
  .map_err(|error| error.to_string())??;
  let _ = app.emit(LIBRARY_EVENT, ());
  Ok(imported)
}

/// Every sound in the library, the most recently added first.
#[tauri::command]
async fn get_sound_library() -> Result<Vec<Sound>, String> {
  tauri::async_runtime::spawn_blocking(store::sounds)
    .await
    .map_err(|error| error.to_string())
}

/// The sounds `names` as the recording open in the editor holds them: its
/// own copies first, else the library's. A sound taken out of the library
/// still plays, and still shows its waveform, in the projects that use it.
#[tauri::command]
async fn get_project_sounds(
  app: tauri::AppHandle,
  names: Vec<String>,
) -> Result<Vec<Sound>, String> {
  let Some(project) = open_project(&app) else {
    return Ok(Vec::new());
  };
  tauri::async_runtime::spawn_blocking(move || {
    let folder = project_sounds(&project);
    names
      .iter()
      .filter_map(|name| store::resolve(folder.as_deref(), name).map(|(_, sound)| sound))
      .collect()
  })
  .await
  .map_err(|error| error.to_string())
}

#[tauri::command]
async fn remove_sound(app: tauri::AppHandle, name: String) -> Result<(), String> {
  tauri::async_runtime::spawn_blocking(move || store::remove(&name))
    .await
    .map_err(|error| error.to_string())??;
  let _ = app.emit(LIBRARY_EVENT, ());
  Ok(())
}

/// The manifest of the recording open in the editor, if one is.
fn open_project(app: &tauri::AppHandle) -> Option<PathBuf> {
  let state = app.state::<EditorState>();
  let artifact = state
    .recording
    .artifact
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  match artifact.as_ref() {
    Some(EditorArtifact::Recording { project, .. }) => Some(project.clone()),
    _ => None,
  }
}

/// Separates, in the background, what the songs of the recording open in
/// the editor need now that Studio ducking is on, which wants every song's
/// singing, not only the instrumental ones'.
pub(crate) fn separate_open_songs(app: &tauri::AppHandle) {
  let open = {
    let state = app.state::<EditorState>();
    let artifact = state
      .recording
      .artifact
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    match artifact.as_ref() {
      Some(EditorArtifact::Recording {
        id,
        project,
        duration_ms,
        ..
      }) => Some((project.clone(), *id, *duration_ms)),
      _ => None,
    }
  };
  let Some((project, id, duration_ms)) = open else {
    return;
  };
  if let Some((_, edit)) = super::timeline_edit::for_project(&project, id, duration_ms) {
    separate_singing(app, &project, &edit.audio_clips);
  }
}

/// Where the project whose manifest is `project` keeps its sounds.
fn project_sounds(project: &Path) -> Option<PathBuf> {
  screenwide_project::resolve(project, store::PROJECT_FOLDER).ok()
}

/// Copies into the project each sound `clips` play that it does not hold
/// yet, so it plays on another computer.
pub(crate) fn carry(project: &Path, clips: &[RecordingAudioClip]) {
  if let Some(folder) = project_sounds(project) {
    store::carry(&folder, clips.iter().map(|clip| clip.sound.as_str()));
  }
}

/// Hands the library every sound the project carries that it lacks.
pub(crate) fn adopt(project: &Path) {
  if let Some(folder) = project_sounds(project) {
    store::adopt(&folder);
  }
}

/// The file the sound `name` plays from, the project's own copy first, with
/// the singing separated from it so far.
fn file(project_sounds: Option<&Path>, name: &str) -> Option<SoundFile> {
  store::resolve(project_sounds, name).map(|(path, sound)| SoundFile {
    separated: store::Separated::read(&path),
    path,
    duration_ms: sound.duration_ms,
    loudness_lufs: sound.loudness_lufs,
    loudest_lufs: sound.loudest_lufs,
    levels: sound.levels,
    music: sound.music.unwrap_or_default(),
  })
}

/// `clips` with the files they play, leaving out the kinds `muted`, under
/// the recorded stream `voice` where there is one, levelled against it as
/// it is heard, the songs making way for it where `ducks` says.
pub(crate) fn sound_track(
  project: &Path,
  (clips, muted): (Vec<RecordingAudioClip>, Vec<AudioClipKind>),
  (voice, ducks): (Option<usize>, bool),
) -> SoundTrack {
  let folder = project_sounds(project);
  SoundTrack {
    clips,
    muted,
    voice,
    voice_lufs: voice.map(|microphone| voice::voice_lufs(project, microphone)),
    ducks,
    ..SoundTrack::default()
  }
  .with_files(|name| file(folder.as_deref(), name))
}

/// Finds the files `plan`'s clips play, from the project's copies first,
/// and the microphone they play under, which the songs make way for where
/// `app` is given and says they do ([`ducks`]).
pub(crate) fn find_sounds(plan: &mut TimelinePlan, project: &Path, app: Option<&tauri::AppHandle>) {
  if plan.sound_track().clips.is_empty() {
    return;
  }
  let voice = microphone(project);
  let heard_at = voice.map(|microphone| voice::voice_lufs(project, microphone));
  let ducking = voice
    .zip(app)
    .is_some_and(|(microphone, app)| ducks(app, project, microphone));
  let folder = project_sounds(project);
  plan.find_sounds((voice, heard_at, ducking), |name| {
    file(folder.as_deref(), name)
  });
}
