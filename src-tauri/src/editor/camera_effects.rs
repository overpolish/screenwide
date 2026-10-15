// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The camera's effects: its background taken away, blurred or replaced by a
//! picture over the stretches of the timeline its ranges cover, and Enhance,
//! drawn live wherever the camera is drawn, so turning one on or off shows at
//! once. The ranges are part of the recording's edit, so they are undone and
//! carried across cuts with the rest of it; Enhance and each range's picture
//! are kept beside it in the project.
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

mod carry;
pub(crate) mod commands;
mod edit_saved;
mod image;
pub(crate) mod live;
mod make;
mod matte;
pub(crate) mod picks;
mod stop;
#[cfg(test)]
mod tests;
pub(crate) mod tidy;

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{EditorArtifact, PrimaryRecordingKind};
use screenwide_export::camera_effects::CameraBackground;
use screenwide_speech::choice_file;

pub(crate) use carry::carry_background;
pub(crate) use edit_saved::edit_saved;

/// The make of the choice file.
const FORMAT_VERSION: u16 = 1;

/// The camera's effects as the editor is told them: the ones kept beside
/// the edit, which of the edit's ranges have a picture of their own, and
/// how far the matte the ranges read is made.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CameraEffects {
  /// The camera at its best: sharpened, enlarged along its edges wherever it
  /// is drawn larger than it was recorded, and its white balance and
  /// exposure corrected with a natural look.
  pub enhance: bool,
  /// The ids of the ranges whose picture is kept in the project. A range
  /// keeps its picture while it shows another background, for next time.
  pub pictures: Vec<String>,
  /// The stretches of the edited video the ranges cover whose matte is not
  /// made yet, in order: the camera is drawn as recorded there until it is.
  pub pending: Vec<PendingStretch>,
  /// Whether the whole camera is matted, which picking objects reads.
  pub can_pick_objects: bool,
}

/// A stretch of the edited video, `[start_ms, end_ms)`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PendingStretch {
  pub start_ms: u64,
  pub end_ms: u64,
}

/// The effects the project keeps beside its edit.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Choice {
  pub(crate) enhance: bool,
}

/// The effects as a project may have kept them. Enhance once only
/// sharpened, beside a separate colour switch; a project with either on
/// reads as enhanced. The background was once one choice for the whole
/// camera, which [`carry_background`] moves into the edit as a range.
#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct Kept {
  background: CameraBackground,
  colour: bool,
  enhance: bool,
}

impl From<Kept> for Choice {
  fn from(kept: Kept) -> Self {
    Self {
      enhance: kept.enhance || kept.colour,
    }
  }
}

/// The camera recording the effects are drawn on, and how long it is.
#[derive(Clone, Debug)]
pub(crate) struct CameraRecording {
  pub(crate) path: PathBuf,
  pub(crate) duration_ms: u64,
  /// How long the recording the edit is made of runs: the camera's own
  /// length for a recording of the camera alone, the screen's otherwise.
  pub(crate) recording_ms: u64,
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
    let (path, camera_ms) = match (camera, primary_kind) {
      (Some(camera), _) => (camera.path.clone(), camera.duration_ms),
      (None, PrimaryRecordingKind::Camera) => (path.clone(), *duration_ms),
      (None, _) => return None,
    };
    Some(Self {
      path,
      duration_ms: camera_ms,
      recording_ms: *duration_ms,
      project: project.clone(),
    })
  }

  pub(crate) fn folder(&self) -> PathBuf {
    project_folder(&self.project)
  }

  /// Whether the project's edit takes the background away anywhere, which
  /// needs the matte.
  pub(crate) fn removes_background(&self) -> bool {
    !super::timeline_edit::saved_camera_background_ranges(&self.project).is_empty()
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

fn kept(folder: &Path) -> Kept {
  choice_file::read(&choice_path(folder), FORMAT_VERSION).unwrap_or_default()
}

/// What the project says. A recording starts with none of the effects.
pub(crate) fn choice(folder: &Path) -> Choice {
  kept(folder).into()
}

fn keep(folder: &Path, effects: &Choice) -> Result<(), String> {
  choice_file::write(&choice_path(folder), FORMAT_VERSION, effects)
    .map_err(|error| format!("Could not keep the camera's effects: {error}"))
}

/// The effects as the editor is told them, for `camera`.
pub(crate) fn as_told(camera: &CameraRecording) -> CameraEffects {
  let folder = camera.folder();
  CameraEffects {
    enhance: choice(&folder).enhance,
    pictures: image::chosen(&folder),
    pending: matte::pending(camera)
      .into_iter()
      .map(|[start_ms, end_ms]| PendingStretch { start_ms, end_ms })
      .collect(),
    can_pick_objects: matte::is_made(&folder),
  }
}
