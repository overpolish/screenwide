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

/// Removes everything built for the artifact that is going away.
///
/// Every path that lets go of a recording - closing it, replacing it with
/// another, exporting it - comes through here, so no derivative outlives the
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

/// Lets go of `kind`'s artifact. Every way a project leaves the editor -
/// closed, exported, copied, or put away for another - comes through here or
/// through its replacement, which is where its project is tidied.
pub(super) fn take_artifact(app: &AppHandle, kind: EditorKind) -> Option<EditorArtifact> {
  let state = app.state::<EditorState>();
  let artifact = state
    .slot(kind)
    .artifact
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .take();
  if let Some(project) = artifact.as_ref().and_then(EditorArtifact::project) {
    super::project_look::clean_pictures(project);
  }
  artifact
}

/// Closes one workspace and puts its window away. What it held stays in its
/// project, to be opened again. The other workspace is untouched.
pub fn close(app: &AppHandle, kind: EditorKind) {
  if kind == EditorKind::Recording {
    clear_recording_preview(app);
  }
  drop(take_artifact(app, kind));
  let _ = window::hide(app, kind);
  emit_snapshot(app, kind);
}
