// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Intentional-cancellation markers and destructive capture teardown.

use super::CaptureHandles;

/// Marks the project as deliberately discarded before native capture
/// teardown begins, so a process exit while an encoder is still joining
/// leaves a project the next launch deletes rather than one that opens.
pub(in crate::recording) fn mark_capture_cancelled(handles: &CaptureHandles) -> Result<(), String> {
  handles.project.mark_cancelled()
}

pub(in crate::recording) fn discard_capture(handles: Option<CaptureHandles>) {
  let Some(CaptureHandles {
    sidecars,
    project,
    session,
    ..
  }) = handles
  else {
    return;
  };

  // Callers normally mark the project before detaching teardown. Keep the
  // blocking-only paths safe as well (late startup cancellation and failure
  // cleanup).
  let _ = project.mark_cancelled();
  session.cancel();
  sidecars.cancel();
  project.remove();
}
