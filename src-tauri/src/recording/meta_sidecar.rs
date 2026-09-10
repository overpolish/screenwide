// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The small metadata sidecar a running recording leaves next to its movie.
//!
//! Everything in here is known while the capture is alive and nowhere in the
//! finished container. After a crash the movie on its own cannot say what
//! display scale it was captured at, and assuming 1x collapses a retina
//! recording's export sizes down to a single choice. The sidecar is written
//! once at start and consumed by whoever finishes the recording, so it never
//! outlives the movie it describes.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::PrimaryRecordingKind;

/// Appended to the recording's stem, matching how the cursor and keyboard
/// sidecars are named.
pub(crate) const SUFFIX: &str = ".meta.json";

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct RecordingMetaSidecar {
  pub has_microphone: bool,
  pub has_system_audio: bool,
  pub primary_kind: PrimaryRecordingKind,
  /// The captured pixels per logical display point, as `FinalizeInfo` carries
  /// it for a recording that was saved the ordinary way.
  pub source_scale_factor: f32,
}

impl Default for RecordingMetaSidecar {
  fn default() -> Self {
    Self {
      has_microphone: false,
      has_system_audio: false,
      primary_kind: PrimaryRecordingKind::Screen,
      // What recovery assumed before this sidecar existed, so a file written
      // by a future version that drops the field still reads sensibly.
      source_scale_factor: 1.0,
    }
  }
}

/// Where the sidecar for a given recording lives. Purely a name computation:
/// the file need not exist.
pub(crate) fn path_for(recording: &Path) -> Option<PathBuf> {
  let stem = recording.file_stem()?.to_str()?;
  Some(recording.with_file_name(format!("{stem}{SUFFIX}")))
}

/// Writes the sidecar next to its recording, temp file first so a crash
/// mid-write can never leave a half-written one to be read back.
pub(crate) fn write(recording: &Path, meta: &RecordingMetaSidecar) {
  let Some(path) = path_for(recording) else {
    return;
  };
  let Ok(contents) = serde_json::to_vec(meta) else {
    return;
  };
  let mut temporary = path.clone().into_os_string();
  temporary.push(".part");
  let temporary = PathBuf::from(temporary);
  if std::fs::write(&temporary, contents).is_ok() && std::fs::rename(&temporary, &path).is_err() {
    let _ = std::fs::remove_file(&temporary);
  }
}

pub(crate) fn read(recording: &Path) -> Option<RecordingMetaSidecar> {
  let contents = std::fs::read(path_for(recording)?).ok()?;
  serde_json::from_slice(&contents).ok()
}

pub(crate) fn remove(recording: &Path) {
  if let Some(path) = path_for(recording) {
    let _ = std::fs::remove_file(path);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn round_trips_through_serde() {
    let meta = RecordingMetaSidecar {
      has_microphone: true,
      has_system_audio: false,
      primary_kind: PrimaryRecordingKind::Screen,
      source_scale_factor: 2.0,
    };
    let encoded = serde_json::to_string(&meta).unwrap();

    assert!(encoded.contains("\"sourceScaleFactor\":2.0"));
    assert!(encoded.contains("\"hasMicrophone\":true"));
    assert!(encoded.contains("\"primaryKind\":\"screen\""));
    assert_eq!(
      serde_json::from_str::<RecordingMetaSidecar>(&encoded).unwrap(),
      meta
    );
  }

  #[test]
  fn reads_a_sidecar_written_by_a_version_that_knew_less() {
    // Forward compatibility runs both ways: a field this version has not seen
    // is ignored, and one an older writer omitted falls back to its default.
    let meta: RecordingMetaSidecar =
      serde_json::from_str(r#"{"sourceScaleFactor":3.0,"somethingNewer":7}"#).unwrap();

    assert_eq!(meta.source_scale_factor, 3.0);
    assert!(!meta.has_microphone);
    assert_eq!(meta.primary_kind, PrimaryRecordingKind::Screen);
  }

  #[test]
  fn names_itself_after_the_recording_it_describes() {
    assert_eq!(
      path_for(Path::new("/tmp/recording-20260808-143205.000.mov")),
      Some(PathBuf::from(
        "/tmp/recording-20260808-143205.000.meta.json"
      ))
    );
    assert_eq!(
      path_for(Path::new("/tmp/audio-20260808-143205.000.mov")),
      Some(PathBuf::from("/tmp/audio-20260808-143205.000.meta.json"))
    );
  }

  #[test]
  fn writes_reads_and_removes_a_sidecar() {
    let directory = std::env::temp_dir()
      .join("screenwide-tests")
      .join("meta-sidecar-lifecycle");
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).unwrap();
    let recording = directory.join("recording-20260808-143205.000.mov");
    let meta = RecordingMetaSidecar {
      has_microphone: false,
      has_system_audio: true,
      primary_kind: PrimaryRecordingKind::Audio,
      source_scale_factor: 2.0,
    };

    write(&recording, &meta);
    assert_eq!(read(&recording), Some(meta));
    // The temp file is renamed, never left behind.
    assert!(!directory
      .join("recording-20260808-143205.000.meta.json.part")
      .exists());

    remove(&recording);
    assert_eq!(read(&recording), None);
  }
}
