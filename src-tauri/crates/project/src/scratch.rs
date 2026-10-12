// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Throwaway projects for tests, here and in the app.

use std::path::PathBuf;

use crate::folder::create_in;
use crate::{write, Manifest, PrimaryRecordingKind, RecordingManifest, RecordingMedia};

/// A fresh folder for test `name` under the temporary directory.
pub fn scratch(name: &str) -> PathBuf {
  let directory = std::env::temp_dir()
    .join("screenwide-tests")
    .join(format!("project-{name}-{}", std::process::id()));
  let _ = std::fs::remove_dir_all(&directory);
  directory
}

/// A recording manifest whose primary track is `primary`.
pub fn recording(primary: &str) -> RecordingManifest {
  RecordingManifest {
    duration_ms: Some(4_000),
    has_microphone: true,
    has_system_audio: false,
    media: RecordingMedia {
      primary: primary.to_owned(),
      camera: None,
      cursor: None,
      keyboard: None,
      moments: None,
    },
    primary_kind: PrimaryRecordingKind::Screen,
    source_scale_factor: 2.0,
    origin: crate::RecordingOrigin::Capture,
  }
}

/// A recording project with a manifest and no media, for tests of what is
/// saved in one. Returns the manifest.
pub fn scratch_project(name: &str) -> PathBuf {
  let directory = scratch(name);
  let project = create_in(&directory, "Scratch").unwrap();
  write(
    &project.file,
    &Manifest::recording(recording("media/recording.mov")),
  )
  .unwrap();
  project.file
}
