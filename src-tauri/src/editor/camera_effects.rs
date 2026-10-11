// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The camera's effects: its background taken away, blurred or replaced by a
//! picture, and Enhance, drawn live wherever the camera is drawn, so turning
//! one on or off shows at once.
//!
//! What is made ahead is only what cannot be worked out per frame: the matte
//! (`matte`), how much of each pixel of the camera is the person, found once
//! by a video matting model in the transcriber program, and the measure of
//! the recording Enhance corrects its colour by (`grade`). The preview and
//! the export read the matte beside the camera (`live`), and the effects
//! pass (`gpu`, and `cpu` for thumbnails and stills) draws the person over
//! what the background is from the two.
//!
//! The camera is the recording's camera track, or for a recording of the
//! camera alone, the recording itself.

mod balance;
pub(crate) mod commands;
pub(crate) mod cpu;
pub(crate) mod gpu;
mod grade;
mod image;
pub(crate) mod live;
mod look;
mod make;
mod matte;
pub(crate) mod matte_frames;
pub(crate) mod picks;
pub(crate) mod preview;
mod sharpen;
mod stop;
#[cfg(test)]
mod tests;
pub(crate) mod tidy;

use std::path::{Path, PathBuf};

use serde::{Deserialize, Deserializer, Serialize};
use ts_rs::TS;

use super::speech::choice_file;
use super::{EditorArtifact, PrimaryRecordingKind};

/// The make of the choice file.
const FORMAT_VERSION: u16 = 1;

/// What is behind the person on camera.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum CameraBackground {
  /// The room, as recorded.
  #[default]
  Kept,
  /// Nothing: whatever the camera is drawn over shows through.
  Transparent,
  /// The room, blurred.
  Blurred,
  /// The picture chosen for it.
  Image,
}

/// The camera's effects, as the project keeps them.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CameraEffects {
  pub background: CameraBackground,
  /// Whether a picture has been chosen for an `Image` background; it is
  /// kept in the project, so choosing another background keeps it for next
  /// time.
  pub has_image: bool,
  /// The camera at its best: sharpened, enlarged along its edges wherever it
  /// is drawn larger than it was recorded, and its white balance and
  /// exposure corrected with a natural look.
  pub enhance: bool,
}

/// The effects as a project may have kept them. Enhance once only
/// sharpened, beside a separate colour switch; a project with either on
/// reads as enhanced.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Kept {
  background: CameraBackground,
  has_image: bool,
  #[serde(default)]
  colour: bool,
  #[serde(default)]
  enhance: bool,
}

impl<'de> Deserialize<'de> for CameraEffects {
  fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    let kept = Kept::deserialize(deserializer)?;
    Ok(Self {
      background: kept.background,
      has_image: kept.has_image,
      enhance: kept.enhance || kept.colour,
    })
  }
}

impl CameraEffects {
  /// Whether the background is taken away, which needs the matte.
  pub(crate) fn removes_background(&self) -> bool {
    self.background != CameraBackground::Kept
  }
}

/// The camera recording the effects are drawn on, and how long it is.
#[derive(Clone, Debug)]
pub(crate) struct CameraRecording {
  pub(crate) path: PathBuf,
  pub(crate) duration_ms: u64,
  pub(crate) project: PathBuf,
}

impl CameraRecording {
  /// The camera of the recording `artifact`: its camera track, or the
  /// recording itself where it is of the camera alone.
  pub(crate) fn of(artifact: &EditorArtifact) -> Option<Self> {
    let EditorArtifact::Recording {
      camera,
      duration_ms,
      path,
      primary_kind,
      project,
      ..
    } = artifact
    else {
      return None;
    };
    let (path, duration_ms) = match (camera, primary_kind) {
      (Some(camera), _) => (camera.path.clone(), camera.duration_ms),
      (None, PrimaryRecordingKind::Camera) => (path.clone(), *duration_ms),
      (None, _) => return None,
    };
    Some(Self {
      path,
      duration_ms,
      project: project.clone(),
    })
  }

  pub(crate) fn folder(&self) -> PathBuf {
    project_folder(&self.project)
  }
}

/// The folder the project whose manifest is `project` keeps its files in.
pub(crate) fn project_folder(project: &Path) -> PathBuf {
  project
    .parent()
    .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

fn choice_path(folder: &Path) -> PathBuf {
  folder.join("camera-effects.json")
}

/// What the project says. A recording starts with none of the effects.
pub(crate) fn choice(folder: &Path) -> CameraEffects {
  choice_file::read(&choice_path(folder), FORMAT_VERSION).unwrap_or_default()
}

fn keep(folder: &Path, effects: &CameraEffects) -> Result<(), String> {
  choice_file::write(&choice_path(folder), FORMAT_VERSION, effects)
    .map_err(|error| format!("Could not keep the camera's effects: {error}"))
}
