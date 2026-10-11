// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the camera's effects need made ahead, each once and kept: the matte,
//! for taking the background away, and the measure of the recording
//! Enhance corrects its colour by. Everything else is drawn live, so
//! changing an effect after these are made shows at once.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use ts_rs::TS;

use super::grade;
use super::{matte, picks, CameraEffects, CameraRecording};

/// Which camera effect is working, each with a bar of its own.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum CameraTool {
  /// Finding the person.
  Background,
  /// Measuring the recording's colour.
  Colour,
  /// Following the objects picked through the recording.
  Objects,
}

/// Enhance's measure of the recording's colour. A measure kept by an earlier
/// make of the app has no gains, so it does not read and is made again.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Measured {
  /// Whether the person was measured, which a matte made later changes.
  pub matte: bool,
  /// The correction, in linear light.
  pub gains: [f64; 3],
}

pub(super) fn measured_path(folder: &Path) -> PathBuf {
  folder.join("camera-colour.json")
}

/// The measure kept for the project in `folder`, where it is current.
pub(super) fn measured(folder: &Path) -> Option<Measured> {
  let bytes = std::fs::read(measured_path(folder)).ok()?;
  serde_json::from_slice::<Measured>(&bytes)
    .ok()
    .filter(|measured| measured.matte == matte::is_made(folder))
}

/// One camera made at a time, so a switch turned while a recording is
/// readied on opening waits for the files rather than writing them again.
static MAKING: Mutex<()> = Mutex::new(());

/// Whether `effects` still need something made that this computer can make.
pub(super) fn has_work(app: &AppHandle, camera: &CameraRecording, effects: &CameraEffects) -> bool {
  let folder = camera.folder();
  let unmatted = effects.removes_background() && !matte::is_made(&folder);
  (unmatted && matte::model(app).is_some())
    || (effects.removes_background() && picks::needs_composing(&folder))
    || (effects.enhance && measured(&folder).is_none())
}

/// Makes what the project's effects need for `camera`, telling `progress`
/// how far each tool has got. They are read once it is this camera's turn,
/// so a switch turned off meanwhile makes nothing.
pub(super) fn ready(
  app: &AppHandle,
  camera: &CameraRecording,
  progress: &mut dyn FnMut(CameraTool, f32),
) -> Result<(), String> {
  let _making = MAKING
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  let folder = camera.folder();
  let effects = super::choice(&folder);
  if effects.removes_background() {
    let model =
      matte::model(app).ok_or_else(|| "Download the background removal model first".to_owned())?;
    matte::prepare(camera, model, &mut |fraction| {
      progress(CameraTool::Background, fraction);
    })?;
    if picks::needs_composing(&folder) {
      picks::compose(camera, &mut |fraction| {
        progress(CameraTool::Objects, fraction);
      })?;
    }
  }
  if effects.enhance && measured(&folder).is_none() {
    progress(CameraTool::Colour, 0.0);
    let matted = matte::is_made(&folder);
    let (_, rate) = matte::shape(&camera.path);
    let frames = (camera.duration_ms as f64 * rate / 1_000.0).ceil() as u64;
    let matte = matted.then(|| matte::path(&folder));
    let gains = grade::measure(&camera.path, matte.as_deref(), (rate, frames))?;
    let measured = Measured {
      matte: matted,
      gains,
    };
    let bytes = serde_json::to_vec(&measured).map_err(|error| error.to_string())?;
    std::fs::write(measured_path(&folder), bytes)
      .map_err(|error| format!("Could not keep the camera's colour: {error}"))?;
    progress(CameraTool::Colour, 1.0);
  }
  Ok(())
}
