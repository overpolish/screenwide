// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Face tracking: where the face is over a recording's camera, found once by
//! the helper and kept beside the camera file, for the camera and the scenes
//! that follow it. The face is found the first time the camera or a scene
//! asks to follow it, listed with the models' work (`model_work`) while it
//! is; the preview follows it from then on, and every export reads it.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use screenwide_export::scenes::face_track::{load, track_path};
use screenwide_speech::runner::Transcriber;
use screenwide_transcriber::Job;
use tauri::{AppHandle, Manager};

use super::CameraRecording;
use crate::editor::model_work::{self, ModelWork};

/// The face detection model, bundled as a resource.
const MODEL: &str = "blazeface-short-range.onnx";

/// The cameras whose face is being found.
static FINDING: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());

/// The model, bundled as a resource, or in the source tree when the app runs
/// unbundled during development.
fn model_path(app: &AppHandle) -> Result<PathBuf, String> {
  let bundled = app
    .path()
    .resource_dir()
    .ok()
    .map(|resources| resources.join("models").join(MODEL));
  let source = Path::new(env!("CARGO_MANIFEST_DIR"))
    .join("assets")
    .join(MODEL);
  bundled
    .into_iter()
    .chain([source])
    .find(|path| path.is_file())
    .ok_or_else(|| "The face tracking model is missing from this installation".to_owned())
}

/// Finds the face in the camera of the recording `artifact_id` where it is
/// not found yet, in the background, telling the editor how far it has got
/// and handing the preview the path once it is there. Answers whether the
/// face is found already.
#[tauri::command]
pub fn find_recording_faces(app: AppHandle, artifact_id: u64) -> Result<bool, String> {
  let camera = super::commands::open(&app, artifact_id)?;
  let path = camera.path.clone();
  if track_path(&path).is_file() {
    return Ok(true);
  }
  {
    let mut finding = FINDING
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if finding.contains(&path) {
      return Ok(false);
    }
    finding.push(path.clone());
  }
  let model = model_path(&app);
  tauri::async_runtime::spawn_blocking(move || {
    let found = model.and_then(|model| find(&app, artifact_id, &camera, model));
    FINDING
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
      .retain(|other| *other != path);
    match found {
      Ok(()) => {
        crate::editor::recording_preview_player::set_face_path(&app, artifact_id, load(&path))
      }
      Err(error) if screenwide_speech::stop::is_stopped(&error) => {}
      Err(error) => eprintln!("Could not find the face in the camera: {error}"),
    }
  });
  Ok(false)
}

fn find(
  app: &AppHandle,
  artifact_id: u64,
  camera: &CameraRecording,
  model: PathBuf,
) -> Result<(), String> {
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  let (width, height) = screenwide_export::media_preview::recording_info(&camera.path)
    .map(|info| (info.width, info.height))
    .ok_or_else(|| "The camera recording cannot be read".to_owned())?;
  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  let (width, height) = (0, 0);
  let work = model_work::Job::begin(app, artifact_id, ModelWork::FollowFace);
  Transcriber::start()?.stopped_by(work.stop()).find_faces(
    Job::FindFaces {
      model,
      ffmpeg: screenwide_sidecar::ffmpeg_path(),
      camera: camera.path.clone(),
      width,
      height,
      duration_ms: camera.duration_ms,
      output: track_path(&camera.path),
    },
    &mut |fraction| work.tell(fraction),
  )
}
