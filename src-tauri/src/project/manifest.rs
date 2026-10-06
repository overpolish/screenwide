// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The `.screenwide` manifest: what a project holds, and reading and writing
//! it without ever leaving a half-written one behind.

use std::fs::File;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::editor::PersistedTimelineEdit;
use crate::recording::PrimaryRecordingKind;

const FORMAT_VERSION: u16 = 1;

/// Every manifest write is a read, a change and a replace. Autosaves and the
/// end of a capture can land together, so they take turns.
static WRITES: Mutex<()> = Mutex::new(());

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ProjectKind {
  Recording,
  Screenshot,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Manifest {
  pub version: u16,
  pub kind: ProjectKind,
  /// What was recorded, for a recording project.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub recording: Option<RecordingManifest>,
  /// The pictures taken, for a screenshot project.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub screenshot: Option<ScreenshotManifest>,
  /// The edit, from the editor's first save on. A project that has never
  /// been edited has none.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub timeline: Option<PersistedTimelineEdit>,
  /// How the project looks: a recording's canvas, cursor, keyboard overlay,
  /// camera bubble and tracks, or a screenshot's canvas and layers, from the
  /// editor's first save on. Kept as it was written and read by the editor,
  /// so a look this version cannot read leaves the project still opening,
  /// with the default look.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub look: Option<serde_json::Value>,
}

impl Manifest {
  pub(crate) fn recording(recording: RecordingManifest) -> Self {
    Self {
      version: FORMAT_VERSION,
      kind: ProjectKind::Recording,
      recording: Some(recording),
      screenshot: None,
      timeline: None,
      look: None,
    }
  }

  pub(crate) fn screenshot(screenshot: ScreenshotManifest) -> Self {
    Self {
      version: FORMAT_VERSION,
      kind: ProjectKind::Screenshot,
      recording: None,
      screenshot: Some(screenshot),
      timeline: None,
      look: None,
    }
  }

  /// What was recorded, or an error for a project that is not a recording.
  pub(crate) fn recorded(&self) -> Result<&RecordingManifest, String> {
    self
      .recording
      .as_ref()
      .filter(|_| self.kind == ProjectKind::Recording)
      .ok_or_else(|| "This project is not a recording".to_owned())
  }
}

/// A screenshot project's pictures, back to front, each as it was taken.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScreenshotManifest {
  pub layers: Vec<ScreenshotLayer>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScreenshotLayer {
  /// The picture, relative to the project folder.
  pub image: String,
  /// How many of its pixels one logical point spans.
  pub scale_factor: f64,
  /// The live annotations it covered when it was taken, which the editor
  /// seeds its layer with until it has saved the layer itself. Kept as
  /// written, like the look, so a shape this version cannot read is lost
  /// rather than stopping the project opening.
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub annotations: Vec<serde_json::Value>,
}

/// What the capture knew that the movie itself cannot say.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RecordingManifest {
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
  /// How the recording was made, which the browser marks.
  #[serde(default)]
  pub origin: RecordingOrigin,
}

/// Recorded from start to stop, or kept from the replay buffer.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum RecordingOrigin {
  #[default]
  Capture,
  Replay,
}

/// Each track, relative to the project folder with `/` between parts.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RecordingMedia {
  pub primary: String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub camera: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub cursor: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub keyboard: Option<String>,
}

impl RecordingMedia {
  /// Names absolute track paths inside the project whose manifest is `file`.
  pub(crate) fn relative_to(
    file: &Path,
    primary: &Path,
    camera: Option<&Path>,
    cursor: Option<&Path>,
    keyboard: Option<&Path>,
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

/// The absolute path of a track named in the manifest at `file`. A name that
/// would reach outside the project folder is refused: a project can come from
/// anyone, and must not be able to point the app at other files.
pub(crate) fn resolve(file: &Path, name: &str) -> Result<PathBuf, String> {
  let mut path = file
    .parent()
    .ok_or_else(|| "The project has no folder".to_owned())?
    .to_path_buf();
  let mut parts = 0;
  for part in name.split('/') {
    let mut components = Path::new(part).components();
    match (components.next(), components.next()) {
      (Some(Component::Normal(part)), None) => path.push(part),
      _ => return Err(format!("The project names a file it cannot hold: {name}")),
    }
    parts += 1;
  }
  if parts == 0 {
    return Err("The project names an empty file".to_owned());
  }
  Ok(path)
}

pub(crate) fn read(file: &Path) -> Result<Manifest, String> {
  #[derive(Deserialize)]
  struct Version {
    version: u16,
  }

  let bytes =
    std::fs::read(file).map_err(|error| format!("The project could not be read: {error}"))?;
  let Version { version } = serde_json::from_slice(&bytes)
    .map_err(|_| format!("{} is not a Screenwide project", file.display()))?;
  if version > FORMAT_VERSION {
    return Err("This project was made by a newer version of Screenwide".to_owned());
  }
  serde_json::from_slice(&bytes).map_err(|error| format!("The project file is damaged: {error}"))
}

pub(crate) fn write(file: &Path, manifest: &Manifest) -> Result<(), String> {
  let _turn = WRITES
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  replace(file, manifest)
}

/// Reads the manifest, lets `change` edit it, and writes it back when
/// `change` says it changed anything.
pub(crate) fn update(
  file: &Path,
  change: impl FnOnce(&mut Manifest) -> bool,
) -> Result<(), String> {
  let _turn = WRITES
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  let mut manifest = read(file)?;
  if change(&mut manifest) {
    replace(file, &manifest)?;
  }
  Ok(())
}

/// Writes beside the manifest and renames over it, which replaces it in one
/// step on both platforms: a crash leaves either the old manifest or the new.
fn replace(file: &Path, manifest: &Manifest) -> Result<(), String> {
  let name = file
    .file_name()
    .and_then(|name| name.to_str())
    .ok_or_else(|| "The project has no file name".to_owned())?;
  let temporary = file.with_file_name(format!(".{name}.tmp"));
  let bytes = serde_json::to_vec(manifest).map_err(|error| error.to_string())?;
  let written = File::create(&temporary).and_then(|mut output| {
    output.write_all(&bytes)?;
    output.sync_all()
  });
  written
    .and_then(|()| std::fs::rename(&temporary, file))
    .map_err(|error| {
      let _ = std::fs::remove_file(&temporary);
      format!("The project could not be saved: {error}")
    })
}
