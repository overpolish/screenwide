// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How much of the running recording is on disk, for the pill to show.

use std::{fs, path::Path};

use tauri::AppHandle;

use super::session::media_directory;

/// The bytes the running recording has written so far, or None outside one.
pub(super) fn written_bytes(app: &AppHandle) -> Option<u64> {
  media_directory(app).map(|media| directory_bytes(&media))
}

/// Adds up the files under `directory`. One that cannot be read, or vanishes
/// mid-walk, counts as nothing rather than failing the whole reading.
fn directory_bytes(directory: &Path) -> u64 {
  let Ok(entries) = fs::read_dir(directory) else {
    return 0;
  };
  entries
    .flatten()
    .map(|entry| {
      let path = entry.path();
      // Asked of the file itself rather than taken from the listing: on
      // Windows the listing's size for a file still open for writing lags
      // what has been written.
      match fs::metadata(&path) {
        Ok(metadata) if metadata.is_dir() => directory_bytes(&path),
        Ok(metadata) => metadata.len(),
        Err(_) => 0,
      }
    })
    .sum()
}
