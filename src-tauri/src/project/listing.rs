// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the browser shows of each project, read from its manifest alone.

use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde::Serialize;
use ts_rs::TS;

use super::{is_cancelled, read, ProjectKind, RecordingManifest, RecordingOrigin, EXTENSION};
use crate::recording::PrimaryRecordingKind;

/// What a project holds, as the browser draws it.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export, rename = "ProjectKind")]
pub(crate) enum SummaryKind {
  Audio,
  Camera,
  Screen,
  Screenshot,
}

impl From<PrimaryRecordingKind> for SummaryKind {
  fn from(kind: PrimaryRecordingKind) -> Self {
    match kind {
      PrimaryRecordingKind::Audio => Self::Audio,
      PrimaryRecordingKind::Camera => Self::Camera,
      PrimaryRecordingKind::Screen => Self::Screen,
    }
  }
}

/// What a recording captured, for its card to show at a glance.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub(crate) struct ProjectTracks {
  pub camera: bool,
  pub microphone: bool,
  pub screen: bool,
  pub system_audio: bool,
}

impl From<&RecordingManifest> for ProjectTracks {
  fn from(recording: &RecordingManifest) -> Self {
    Self {
      // A camera recording's camera is its primary movie; a screen
      // recording's is the second one it names.
      camera: recording.primary_kind == PrimaryRecordingKind::Camera
        || recording.media.camera.is_some(),
      microphone: recording.has_microphone,
      screen: recording.primary_kind == PrimaryRecordingKind::Screen,
      system_audio: recording.has_system_audio,
    }
  }
}

/// A project as the browser lists it, read from its manifest alone.
#[derive(Clone, Debug, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub(crate) struct ProjectSummary {
  /// Whether the manifest could be read. A project on a drive that is not
  /// connected, or one that was moved away, is listed but cannot be opened.
  pub available: bool,
  /// When it goes on to the system Trash, for a project in Recently
  /// Deleted; None elsewhere.
  pub expires_ms: Option<u64>,
  pub duration_ms: Option<u64>,
  pub file: PathBuf,
  pub kind: Option<SummaryKind>,
  /// When the manifest last changed, which is when the project was last
  /// edited.
  pub modified_ms: Option<u64>,
  /// Everything in the project's folder, in bytes: what keeping it costs.
  pub size_bytes: Option<u64>,
  /// Kept from the replay buffer rather than recorded start to stop.
  pub replay: bool,
  pub title: String,
  /// None for a screenshot, and for a project that could not be read.
  pub tracks: Option<ProjectTracks>,
}

pub(crate) fn summarize(file: &Path) -> ProjectSummary {
  let title = file
    .file_stem()
    .and_then(|stem| stem.to_str())
    .unwrap_or_default()
    .to_owned();
  let modified_ms = std::fs::metadata(file)
    .and_then(|metadata| metadata.modified())
    .ok()
    .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
    .and_then(|age| u64::try_from(age.as_millis()).ok());
  match read(file) {
    Ok(manifest) => {
      let recording = manifest
        .recording
        .as_ref()
        .filter(|_| manifest.kind == ProjectKind::Recording);
      ProjectSummary {
        available: true,
        expires_ms: None,
        duration_ms: recording.and_then(|recording| recording.duration_ms),
        file: file.to_path_buf(),
        kind: match manifest.kind {
          ProjectKind::Recording => recording.map(|recording| recording.primary_kind.into()),
          ProjectKind::Screenshot => Some(SummaryKind::Screenshot),
        },
        modified_ms,
        size_bytes: file.parent().map(folder_size),
        replay: recording.is_some_and(|recording| recording.origin == RecordingOrigin::Replay),
        // A name given while it is open stands until its folder takes it.
        title: manifest.title.clone().unwrap_or(title),
        tracks: recording.map(ProjectTracks::from),
      }
    }
    Err(_) => ProjectSummary {
      available: false,
      expires_ms: None,
      duration_ms: None,
      file: file.to_path_buf(),
      kind: None,
      modified_ms,
      size_bytes: None,
      replay: false,
      title,
      tracks: None,
    },
  }
}

/// The bytes of every file under `root`. Links are not followed, so a link
/// to something outside the project neither counts nor loops.
pub(crate) fn folder_size(root: &Path) -> u64 {
  let mut total = 0;
  let mut folders = vec![root.to_path_buf()];
  while let Some(folder) = folders.pop() {
    let Ok(entries) = std::fs::read_dir(&folder) else {
      continue;
    };
    for entry in entries.flatten() {
      let Ok(metadata) = entry.metadata() else {
        continue;
      };
      if metadata.is_dir() {
        folders.push(entry.path());
      } else if metadata.is_file() {
        total += metadata.len();
      }
    }
  }
  total
}

/// Every project directly inside `folder`, most recently edited first.
pub(crate) fn in_folder(folder: &Path) -> Vec<ProjectSummary> {
  let Ok(entries) = std::fs::read_dir(folder) else {
    return Vec::new();
  };
  let mut projects: Vec<_> = entries
    .flatten()
    .map(|entry| entry.path())
    // Hidden folders, Recently Deleted's among them, hold no project here.
    .filter(|path| {
      path.is_dir()
        && !path
          .file_name()
          .is_some_and(|name| name.to_string_lossy().starts_with('.'))
    })
    .filter_map(|root| manifest_in(&root))
    .filter(|file| !is_cancelled(file))
    .map(|file| summarize(&file))
    .collect();
  projects.sort_by_key(|project| std::cmp::Reverse(project.modified_ms));
  projects
}

/// A project folder's manifest: the one named after the folder, or, in a
/// folder renamed outside the app, whichever manifest it holds.
fn manifest_in(root: &Path) -> Option<PathBuf> {
  let name = root.file_name()?.to_str()?;
  let named = root.join(format!("{name}.{EXTENSION}"));
  if named.is_file() {
    return Some(named);
  }
  std::fs::read_dir(root)
    .ok()?
    .flatten()
    .map(|entry| entry.path())
    .find(|path| {
      path.is_file()
        && path
          .extension()
          .is_some_and(|extension| extension.eq_ignore_ascii_case(EXTENSION))
    })
}
