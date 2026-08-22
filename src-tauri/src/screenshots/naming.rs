// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::{Path, PathBuf};

use chrono::NaiveDateTime;
use tauri::{AppHandle, Manager};

/// The naming macOS's own `screencapture` uses, which is the least surprising
/// thing to find sitting on a Desktop. Recordings are named the same way, from
/// the moment they started, so a session's files sit together in order.
pub fn capture_file_stem(captured_at: NaiveDateTime) -> String {
  captured_at
    .format("Screenwide %Y-%m-%d at %H.%M.%S")
    .to_string()
}

/// Appends " (2)", " (3)" and so on until the name is free, as both platforms'
/// file managers do. `exists` is injected so the walk can be tested without
/// touching a disk.
pub fn unique_path(
  directory: &Path,
  stem: &str,
  extension: &str,
  exists: &dyn Fn(&Path) -> bool,
) -> PathBuf {
  let mut candidate = directory.join(format!("{stem}.{extension}"));
  let mut suffix = 1_u32;

  while exists(&candidate) {
    suffix += 1;
    candidate = directory.join(format!("{stem} ({suffix}).{extension}"));
  }

  candidate
}

/// Where a still goes when it is not going to the clipboard. Both are the
/// platform's own screenshot destination.
pub fn screenshot_directory(app: &AppHandle) -> Result<PathBuf, String> {
  let path = app.path();

  #[cfg(target_os = "macos")]
  let directory = path.desktop_dir().map_err(|error| error.to_string())?;

  #[cfg(not(target_os = "macos"))]
  let directory = path
    .picture_dir()
    .map_err(|error| error.to_string())?
    .join("Screenshots");

  Ok(directory)
}
