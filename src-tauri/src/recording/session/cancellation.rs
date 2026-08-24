// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Intentional-cancellation markers and destructive capture teardown.

use std::path::{Path, PathBuf};

use super::CaptureHandles;

/// Marks a working movie as deliberately discarded before native capture
/// teardown begins. Recovery checks this sibling first, so a process exit
/// while an encoder is still joining cannot resurrect the partial movie.
pub(in crate::recording) fn mark_capture_cancelled(handles: &CaptureHandles) -> Result<(), String> {
  std::fs::write(cancelled_marker(&handles.output_path), []).map_err(|error| error.to_string())
}

pub(crate) fn cancelled_marker(path: &Path) -> PathBuf {
  let mut name = path.as_os_str().to_owned();
  name.push(".cancelled");
  PathBuf::from(name)
}

pub(in crate::recording) fn discard_capture(handles: Option<CaptureHandles>) {
  let Some(CaptureHandles {
    cursor,
    output_path,
    session,
    ..
  }) = handles
  else {
    return;
  };

  // Callers normally create this synchronously before detaching teardown.
  // Keep the blocking-only paths safe as well (late startup cancellation and
  // failure cleanup).
  let marker = cancelled_marker(&output_path);
  let _ = std::fs::write(&marker, []);
  session.cancel();
  if let Some(cursor) = cursor {
    cursor.cancel();
  }
  let removed = std::fs::remove_file(output_path).is_ok();
  if removed {
    let _ = std::fs::remove_file(marker);
  }
}
