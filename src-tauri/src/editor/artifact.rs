// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn snapshots(app: &AppHandle) -> EditorSnapshots {
  EditorSnapshots {
    recording: snapshot(app, EditorKind::Recording),
    screenshot: snapshot(app, EditorKind::Screenshot),
  }
}

/// Broadcast rather than sent to the owning window: the recording bar tracks
/// what is waiting too. Receivers route the payload by its `workspace`.
pub(super) fn emit_snapshot(app: &AppHandle, kind: EditorKind) {
  let _ = app.emit(EDITOR_CHANGED_EVENT, snapshot(app, kind));
}

pub(super) fn delete_working_file(artifact: &EditorArtifact) {
  if let EditorArtifact::Recording {
    camera,
    cursor,
    keyboard,
    path,
    ..
  } = artifact
  {
    let _ = std::fs::remove_file(path);
    if let Some(camera) = camera {
      let _ = std::fs::remove_file(&camera.path);
    }
    recording_sidecar::remove_working_files(cursor.as_ref(), keyboard.as_ref());
    timeline_edit::remove_for_recording(path);
  }
}

/// Removes everything built for the artifact that is going away.
///
/// Every path that lets go of a recording - discarding it, replacing it with a
/// new capture, saving it - comes through here, so no derivative outlives the
/// artifact it was made from.
pub(super) fn clear_recording_preview(app: &AppHandle) {
  super::recording_preview_player::stop_all(app);
  let state = app.state::<EditorState>();
  state
    .recording_preview
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .take();
  state
    .compression_estimates
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .clear();
}

/// The next artifact identity. Two consecutive captures are otherwise
/// indistinguishable, and the window needs to tell them apart.
pub(super) fn next_id(app: &AppHandle) -> u64 {
  app
    .state::<EditorState>()
    .generation
    .fetch_add(1, Ordering::SeqCst)
    .wrapping_add(1)
}

pub(super) fn take_artifact(app: &AppHandle, kind: EditorKind) -> Option<EditorArtifact> {
  let state = app.state::<EditorState>();
  let artifact = state
    .slot(kind)
    .artifact
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .take();

  artifact
}

/// Drops one workspace's pending artifact and puts its window away. Cancelling
/// and closing that window are the same act; the other workspace is untouched.
pub fn discard(app: &AppHandle, kind: EditorKind) {
  if kind == EditorKind::Recording {
    clear_recording_preview(app);
  }
  if let Some(artifact) = take_artifact(app, kind) {
    delete_working_file(&artifact);
  }
  let _ = window::hide(app, kind);
  emit_snapshot(app, kind);
}
