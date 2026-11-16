// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a recording project's manifest says of the recording.

use std::path::{Component, Path};

use serde::{Deserialize, Serialize};

use super::PrimaryRecordingKind;

/// What the capture knew that the movie itself cannot say.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingManifest {
  /// How long the recording is, once it has stopped. Kept here so the project
  /// browser can say it without opening the movie.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub duration_ms: Option<u64>,
  pub has_microphone: bool,
  pub has_system_audio: bool,
  pub media: RecordingMedia,
  pub primary_kind: PrimaryRecordingKind,
  /// The captured pixels per logical display point, which decides the export
  /// sizes offered.
  pub source_scale_factor: f32,
  /// A window's own corner radius, in the movie's pixels, which has it black.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub corner_radius: Option<f32>,
  /// How the recording was made, which the browser marks.
  #[serde(default)]
  pub origin: RecordingOrigin,
}

/// Recorded from start to stop, or kept from the replay buffer.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RecordingOrigin {
  #[default]
  Capture,
  Replay,
}

/// Each track, relative to the project folder with `/` between parts.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingMedia {
  pub primary: String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub camera: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub cursor: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub keyboard: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub moments: Option<String>,
}

impl RecordingMedia {
  /// Names absolute track paths inside the project whose manifest is `file`.
  pub fn relative_to(
    file: &Path,
    primary: &Path,
    camera: Option<&Path>,
    cursor: Option<&Path>,
    keyboard: Option<&Path>,
    moments: Option<&Path>,
  ) -> Result<Self, String> {
    let root = file
      .parent()
      .ok_or_else(|| "The project has no folder".to_owned())?;
    let name = |path: &Path| relative(root, path);
    Ok(Self {
      primary: name(primary)?,
      camera: camera.map(name).transpose()?,
      cursor: cursor.map(name).transpose()?,
      keyboard: keyboard.map(name).transpose()?,
      moments: moments.map(name).transpose()?,
    })
  }
}

fn relative(root: &Path, path: &Path) -> Result<String, String> {
  let inside = path
    .strip_prefix(root)
    .map_err(|_| format!("{} is outside its project", path.display()))?;
  let parts = inside
    .components()
    .map(|component| match component {
      Component::Normal(part) => part.to_str().map(str::to_owned),
      _ => None,
    })
    .collect::<Option<Vec<_>>>()
    .ok_or_else(|| format!("{} cannot be named in a project", path.display()))?;
  Ok(parts.join("/"))
}
