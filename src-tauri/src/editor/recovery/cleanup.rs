// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(in crate::editor) fn orphaned_recordings(directory: &Path) -> Vec<(PathBuf, SystemTime)> {
  let Ok(entries) = std::fs::read_dir(directory) else {
    return Vec::new();
  };

  entries
    .filter_map(|entry| {
      let path = entry.ok()?.path();
      // A mixed preview is an `.mp4` in this same folder, and offering one
      // back as an unsaved recording would hand the user a derivative in place
      // of what they actually recorded.
      if media_preview::is_preview_file(&path) {
        return None;
      }
      if crate::recording::cancelled_marker(&path).is_file() {
        return None;
      }
      if !path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with("recording-") || name.starts_with("audio-"))
      {
        return None;
      }
      let extension = path.extension()?;
      if WORKING_RECORDING_EXTENSIONS
        .iter()
        .any(|candidate| extension.eq_ignore_ascii_case(candidate))
      {
        Some((
          path.clone(),
          std::fs::metadata(&path).ok()?.modified().ok()?,
        ))
      } else {
        None
      }
    })
    .collect()
}

/// Finishes cleanup for a cancellation interrupted by process shutdown. The
/// marker is retained if deletion fails so the movie remains non-recoverable.
pub(in crate::editor) fn sweep_cancelled_recordings(directory: &Path) {
  let Ok(entries) = std::fs::read_dir(directory) else {
    return;
  };
  for entry in entries.flatten() {
    let marker = entry.path();
    let Some(name) = marker.file_name().and_then(|name| name.to_str()) else {
      continue;
    };
    let Some(recording_name) = name.strip_suffix(".cancelled") else {
      continue;
    };
    let recording = marker.with_file_name(recording_name);
    if !recording.exists() || std::fs::remove_file(&recording).is_ok() {
      let _ = std::fs::remove_file(marker);
    }
  }
}
