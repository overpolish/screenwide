// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// Retire working sources only after every requested output has been published.
pub(super) fn finish(
  saved: &Path,
  saved_camera: Option<PathBuf>,
  working: &Path,
  camera: Option<&RecordingCamera>,
  include_camera: bool,
  cursor: Option<&RecordingCursor>,
  keyboard: Option<&RecordingKeyboard>,
) -> Result<(), String> {
  if !saved.is_file() || saved_camera.as_ref().is_some_and(|path| !path.is_file()) {
    let _ = std::fs::remove_file(saved);
    if let Some(path) = saved_camera {
      let _ = std::fs::remove_file(path);
    }
    return Err("The exported recording did not finish publishing".to_owned());
  }
  let _ = std::fs::remove_file(working);
  if !include_camera {
    if let Some(camera) = camera {
      let _ = std::fs::remove_file(&camera.path);
    }
  }
  recording_sidecar::remove_working_files(cursor, keyboard);
  Ok(())
}
