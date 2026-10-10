// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The camera's effects as the editor turns them. A change is kept and shown
//! at once; where it needs something made first, the matte or the colour
//! correction's measure, that is made with its progress told and shown as
//! soon as it is there.

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, WebviewWindow};
use ts_rs::TS;

use super::make::{self, CameraTool};
use super::stop::{self, Work};
use super::{choice, image, keep, live, CameraBackground, CameraEffects, CameraRecording};
use crate::editor::EditorState;

/// The event that tells the editor how far a camera effect has got.
const PROGRESS_EVENT: &str = "editor://camera-effects-progress";

/// How far a camera effect has got, 0 to 1.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CameraEffectsProgress {
  pub artifact_id: u64,
  pub fraction: f32,
  pub tool: CameraTool,
}

/// The camera of the recording open in the editor, if it is `artifact_id`.
pub(super) fn open(app: &AppHandle, artifact_id: u64) -> Result<CameraRecording, String> {
  let state = app.state::<EditorState>();
  let artifact = state
    .recording
    .artifact
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  let artifact = artifact
    .as_ref()
    .ok_or_else(|| "There is no recording open".to_owned())?;
  if !matches!(artifact, crate::editor::EditorArtifact::Recording { id, .. } if *id == artifact_id)
  {
    return Err("That recording is no longer available in the editor".to_owned());
  }
  CameraRecording::of(artifact).ok_or_else(|| "This recording has no camera".to_owned())
}

/// The bars being told, by recording and tool.
static TOLD: Mutex<Vec<(u64, CameraTool)>> = Mutex::new(Vec::new());

/// Tells the editor how far each tool has got. Once a tool has been told
/// anything, it is told it is full when this goes, whether the work
/// finished or failed, so no bar hangs part-way. Where other work on the
/// same tool is telling its bar already, this keeps quiet about that tool,
/// so the bar is not pulled back and forth between the two.
pub(super) struct Bars {
  app: AppHandle,
  artifact_id: u64,
  told: HashSet<CameraTool>,
}

impl Bars {
  pub(super) fn new(app: &AppHandle, artifact_id: u64) -> Self {
    Self {
      app: app.clone(),
      artifact_id,
      told: HashSet::new(),
    }
  }

  pub(super) fn tell(&mut self, tool: CameraTool, fraction: f32) {
    if !self.told.contains(&tool) {
      let mut told = told();
      if told.contains(&(self.artifact_id, tool)) {
        return;
      }
      told.push((self.artifact_id, tool));
      self.told.insert(tool);
    }
    self.emit(tool, fraction);
  }

  fn emit(&self, tool: CameraTool, fraction: f32) {
    let _ = self.app.emit(
      PROGRESS_EVENT,
      CameraEffectsProgress {
        artifact_id: self.artifact_id,
        fraction,
        tool,
      },
    );
  }
}

impl Drop for Bars {
  fn drop(&mut self) {
    let tools = std::mem::take(&mut self.told);
    told().retain(|(id, tool)| *id != self.artifact_id || !tools.contains(tool));
    for tool in tools {
      self.emit(tool, 1.0);
    }
  }
}

fn told() -> std::sync::MutexGuard<'static, Vec<(u64, CameraTool)>> {
  TOLD.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Makes what the project's effects still need, then shows the camera with
/// them.
fn ready(app: &AppHandle, artifact_id: u64, camera: &CameraRecording) -> Result<(), String> {
  let mut bars = Bars::new(app, artifact_id);
  let made = make::ready(app, camera, &mut |tool, fraction| {
    bars.tell(tool, fraction);
  });
  drop(bars);
  shown_changed(app, artifact_id, camera);
  made
}

/// The camera's effects for the recording open in the editor, as its
/// project says. Where one is on and what it needs is still to make, as when
/// a model arrived after it was turned on, that is made in the background
/// with its progress told.
#[tauri::command]
pub fn get_recording_camera_effects(
  app: AppHandle,
  artifact_id: u64,
) -> Result<CameraEffects, String> {
  let camera = open(&app, artifact_id)?;
  let effects = choice(&camera.folder());
  if make::has_work(&app, &camera, &effects) {
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
      if let Err(error) = ready(&app, artifact_id, &camera) {
        eprintln!("Could not make the camera's effects: {error}");
      }
    });
  }
  Ok(effects)
}

/// Gives the camera `effects`, shown at once where nothing needs making
/// first. Where making it fails, the camera keeps the effects it had.
/// Answers with what the project says now.
#[tauri::command]
pub async fn set_recording_camera_effects(
  app: AppHandle,
  artifact_id: u64,
  effects: CameraEffects,
) -> Result<CameraEffects, String> {
  let camera = open(&app, artifact_id)?;
  if effects.background == CameraBackground::Image && !image::path(&camera.folder()).is_file() {
    return Err("Choose a picture for the background first".to_owned());
  }
  apply(app, artifact_id, camera, effects).await
}

/// Asks for a picture to put behind the person and puts it there, or does
/// nothing when the picker is dismissed. Answers with what the project says
/// now.
#[tauri::command]
pub async fn choose_recording_camera_background(
  app: AppHandle,
  window: WebviewWindow,
  artifact_id: u64,
) -> Result<CameraEffects, String> {
  let camera = open(&app, artifact_id)?;
  let folder = camera.folder();
  let picker = app.clone();
  let chosen = tauri::async_runtime::spawn_blocking(move || {
    use tauri_plugin_dialog::DialogExt;

    let picked: Option<PathBuf> = picker
      .dialog()
      .file()
      .set_title(crate::i18n::t!("dialog-choose-image"))
      .add_filter(
        crate::i18n::t!("dialog-pictures-filter"),
        crate::editor::images::import::PICTURE_EXTENSIONS,
      )
      .set_parent(&window)
      .blocking_pick_file()
      .and_then(|path| path.into_path().ok());
    picked.map(|path| image::import(&path, &folder)).transpose()
  })
  .await
  .map_err(|error| error.to_string())??;
  let folder = camera.folder();
  if chosen.is_none() {
    return Ok(choice(&folder));
  }
  let effects = CameraEffects {
    background: CameraBackground::Image,
    ..choice(&folder)
  };
  apply(app, artifact_id, camera, effects).await
}

/// Keeps `effects`, shows them, and makes what they need, putting back the
/// effects the camera had where that fails and nothing has changed them
/// since. Turning background removal off stops what was being made for it.
/// Answers with what the project says once done.
async fn apply(
  app: AppHandle,
  artifact_id: u64,
  camera: CameraRecording,
  effects: CameraEffects,
) -> Result<CameraEffects, String> {
  tauri::async_runtime::spawn_blocking(move || {
    let folder = camera.folder();
    let before = choice(&folder);
    let effects = CameraEffects {
      has_image: image::path(&folder).is_file(),
      ..effects
    };
    keep(&folder, &effects)?;
    if !effects.removes_background() {
      for work in [Work::Matte, Work::Following, Work::Composing] {
        stop::stop(&folder, work);
      }
    }
    shown_changed(&app, artifact_id, &camera);
    if make::has_work(&app, &camera, &effects) {
      if let Err(error) = ready(&app, artifact_id, &camera) {
        if choice(&folder) == effects {
          keep(&folder, &before)?;
          shown_changed(&app, artifact_id, &camera);
        }
        return Err(error);
      }
    }
    Ok(choice(&folder))
  })
  .await
  .map_err(|error| error.to_string())?
}

/// Shows the camera with its effects as the project has them now.
pub(super) fn shown_changed(app: &AppHandle, artifact_id: u64, camera: &CameraRecording) {
  crate::editor::recording_preview_player::set_camera_effects(app, artifact_id, live::load(camera));
}
