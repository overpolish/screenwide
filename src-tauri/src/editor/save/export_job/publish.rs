// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// Confirms every requested output was published, removing the partial set
/// otherwise so a failed export never leaves half of itself behind.
pub(super) fn finish(saved: &Path, saved_camera: Option<PathBuf>) -> Result<(), String> {
  if !saved.is_file() || saved_camera.as_ref().is_some_and(|path| !path.is_file()) {
    let _ = std::fs::remove_file(saved);
    if let Some(path) = saved_camera {
      let _ = std::fs::remove_file(path);
    }
    return Err("The exported recording did not finish publishing".to_owned());
  }
  Ok(())
}
