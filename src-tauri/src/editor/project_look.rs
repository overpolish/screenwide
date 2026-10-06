// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How a project looks, kept in the project.
//!
//! The canvas, cursor, keyboard overlay and camera bubble a recording was
//! left with, and which of its tracks are used at what volume, saved in its
//! manifest as the editor changes them. Reopened, a project shows what it was
//! left showing, whatever another project has done since. A project with no
//! look saved takes the remembered one, which an export updates.
//!
//! The pictures a look or an edit shows are copied into the project, so it
//! looks the same on another computer.

mod cleanup;
mod pictures;

use super::*;
pub(crate) use cleanup::clean_pictures;
pub(crate) use pictures::{adopt_pictures, carry_pictures, keep_in_app_data};

/// Bumped when the look's shape changes in a way older saves cannot be read
/// as; a look of another version is ignored rather than guessed at.
const FORMAT_VERSION: u16 = 1;

/// One of a recording's video tracks, as the window names it.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum VideoTrack {
  Camera,
  Primary,
}

/// `RecordingProjectLook` in `src/features/editor/types.ts`.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectLook {
  pub bake_camera: bool,
  pub camera_overlay: CameraOverlaySettings,
  pub cursor_effects: cursor_effects::CursorEffectSettings,
  pub keyboard_effects: keyboard_effects::KeyboardEffectSettings,
  pub recording_output: RecordingOutputSettings,
  /// None for each where the recording's own default stands: every track
  /// on, at full volume.
  #[serde(default)]
  pub audio_track_volumes: Option<Vec<AudioTrackVolume>>,
  #[serde(default)]
  pub enabled_stream_indices: Option<Vec<usize>>,
  #[serde(default)]
  pub enabled_video_tracks: Option<Vec<VideoTrack>>,
}

/// The look as a manifest keeps it: a recording's [`ProjectLook`] or a
/// screenshot's workspace. The revision orders saves that can arrive out of
/// turn, so a slow older save never lands over a newer one.
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PersistedLook {
  look: serde_json::Value,
  revision: u64,
  version: u16,
}

/// The look saved in the project whose manifest is `project`, as written,
/// its pictures named where they are now. None for a project with none this
/// version reads.
pub(crate) fn saved_look(project: &Path) -> Option<serde_json::Value> {
  let persisted: PersistedLook =
    serde_json::from_value(crate::project::read(project).ok()?.look?).ok()?;
  if persisted.version != FORMAT_VERSION {
    return None;
  }
  let mut look = persisted.look;
  pictures::resolve_backgrounds(project, &mut look);
  Some(look)
}

/// The recording look saved in the project whose manifest is `project`.
pub fn for_project(project: &Path) -> Option<ProjectLook> {
  serde_json::from_value(saved_look(project)?).ok()
}

/// Saves the open recording's look in its project.
#[tauri::command]
pub async fn set_recording_project_look(
  app: AppHandle,
  artifact_id: u64,
  revision: u64,
  look: ProjectLook,
) -> Result<(), String> {
  let project = {
    let state = app.state::<EditorState>();
    let artifact = state
      .recording
      .artifact
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    match artifact.as_ref() {
      Some(EditorArtifact::Recording { id, project, .. }) if *id == artifact_id => project.clone(),
      _ => return Err("That recording is no longer open in the editor".to_owned()),
    }
  };
  // Off the async runtime: a background picture is copied in the first time
  // it is saved.
  tauri::async_runtime::spawn_blocking(move || persist(&project, revision, look))
    .await
    .map_err(|error| error.to_string())?
}

fn persist(project: &Path, revision: u64, mut look: ProjectLook) -> Result<(), String> {
  for output in [
    &mut look.recording_output.primary,
    &mut look.recording_output.camera,
  ] {
    // A recording's annotations are its edit's, saved with the timeline.
    output.annotations.clear();
  }
  save_look(
    project,
    revision,
    serde_json::to_value(look).map_err(|error| error.to_string())?,
  )
}

/// Saves `look` in the project at `revision`, with the pictures it shows
/// copied in, unless a newer save has landed first.
pub(crate) fn save_look(
  project: &Path,
  revision: u64,
  mut look: serde_json::Value,
) -> Result<(), String> {
  pictures::embed_backgrounds(project, &mut look);
  carry_pictures(project, &look);
  let value = serde_json::to_value(PersistedLook {
    look,
    revision,
    version: FORMAT_VERSION,
  })
  .map_err(|error| error.to_string())?;
  crate::project::update(project, |manifest| {
    let saved = manifest
      .look
      .as_ref()
      .and_then(|look| look.get("revision"))
      .and_then(serde_json::Value::as_u64);
    if saved.is_some_and(|saved| saved >= revision) {
      return false;
    }
    manifest.look = Some(value);
    true
  })
}

#[cfg(test)]
mod tests;
