// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The `.screenwide` manifest: what a project holds, and reading and writing
//! it without ever leaving a half-written one behind.

mod recording;

use std::fs::File;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use screenwide_timeline_edit::PersistedTimelineEdit;

pub use self::recording::{RecordingManifest, RecordingMedia, RecordingOrigin};
use crate::ReadError;

const FORMAT_VERSION: u16 = 1;

/// Every manifest write is a read, a change and a replace. Autosaves and the
/// end of a capture can land together, so they take turns.
static WRITES: Mutex<()> = Mutex::new(());

/// Holds the manifests' write turn, for a change that must not land in the
/// middle of one, such as moving the folder a manifest is being written in.
pub(super) fn hold_writes() -> MutexGuard<'static, ()> {
  WRITES
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum PrimaryRecordingKind {
  #[default]
  Screen,
  Camera,
  Audio,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectKind {
  Recording,
  Screenshot,
  Notes,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
  pub version: u16,
  pub kind: ProjectKind,
  /// What was recorded, for a recording project.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub recording: Option<RecordingManifest>,
  /// The pictures taken, for a screenshot project.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub screenshot: Option<ScreenshotManifest>,
  /// The notes taken, for a notes project.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub notes: Option<NotesManifest>,
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
  /// The name the project was given while an editor had it open. Its folder
  /// and manifest take it once the editor lets the project go, since the
  /// editor's files cannot move under it; until then this is its name.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub title: Option<String>,
}

impl Manifest {
  pub fn recording(recording: RecordingManifest) -> Self {
    Self {
      version: FORMAT_VERSION,
      kind: ProjectKind::Recording,
      recording: Some(recording),
      screenshot: None,
      notes: None,
      timeline: None,
      look: None,
      title: None,
    }
  }

  pub fn screenshot(screenshot: ScreenshotManifest) -> Self {
    Self {
      version: FORMAT_VERSION,
      kind: ProjectKind::Screenshot,
      recording: None,
      screenshot: Some(screenshot),
      notes: None,
      timeline: None,
      look: None,
      title: None,
    }
  }

  pub fn notes(notes: NotesManifest) -> Self {
    Self {
      version: FORMAT_VERSION,
      kind: ProjectKind::Notes,
      recording: None,
      screenshot: None,
      notes: Some(notes),
      timeline: None,
      look: None,
      title: None,
    }
  }

  /// What was recorded, or an error for a project that is not a recording.
  pub fn recorded(&self) -> Result<&RecordingManifest, String> {
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
pub struct ScreenshotManifest {
  pub layers: Vec<ScreenshotLayer>,
}

/// A notes project's notes: the Markdown file they are written in, relative
/// to the project folder, how long the call they were taken from ran, and
/// the words marked as not private in them, which are not pointed out
/// again. Those are kept here rather than in the notes, which stay plain for
/// whatever reads them.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotesManifest {
  pub file: String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub duration_ms: Option<u64>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub not_private: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenshotLayer {
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
  /// The canvas's own layer, whose picture is empty: it holds what is drawn
  /// on a blank canvas and is never drawn or chosen as a picture.
  #[serde(default, skip_serializing_if = "std::ops::Not::not")]
  pub blank: bool,
}

/// The absolute path of a track named in the manifest at `file`. A name that
/// would reach outside the project folder is refused: a project can come from
/// anyone, and must not be able to point the app at other files.
pub fn resolve(file: &Path, name: &str) -> Result<PathBuf, String> {
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

pub fn read(file: &Path) -> Result<Manifest, ReadError> {
  #[derive(Deserialize)]
  struct Version {
    version: u16,
  }

  let bytes = std::fs::read(file).map_err(ReadError::Unreadable)?;
  let Version { version } =
    serde_json::from_slice(&bytes).map_err(|_| ReadError::NotProject(file.to_path_buf()))?;
  if version > FORMAT_VERSION {
    return Err(ReadError::Newer);
  }
  serde_json::from_slice(&bytes).map_err(ReadError::Damaged)
}

pub fn write(file: &Path, manifest: &Manifest) -> Result<(), String> {
  let _turn = WRITES
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  replace(file, manifest)
}

/// Reads the manifest, lets `change` edit it, and writes it back when
/// `change` says it changed anything.
pub fn update(file: &Path, change: impl FnOnce(&mut Manifest) -> bool) -> Result<(), String> {
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
