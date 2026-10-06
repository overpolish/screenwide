// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::{AppHandle, Manager};

use super::{window, EditorKind, EditorState};

pub(super) fn focus_pending(app: &AppHandle, kind: EditorKind) {
  let _ = window::show(app, kind);
}

/// Whether the workspace has something open.
pub fn has_pending_kind(app: &AppHandle, kind: EditorKind) -> bool {
  app
    .state::<EditorState>()
    .slot(kind)
    .artifact
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .is_some()
}

/// Screenshot tools open over whatever else is waiting: each workspace has its
/// own window, so only a capture already in flight blocks a new one.
pub fn focus_if_screenshot_blocked(app: &AppHandle) -> bool {
  let state = app.state::<EditorState>();
  let reservation = *state
    .capture_reservation
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  let Some(kind) = reservation else {
    return false;
  };
  // A reservation of its own has nothing to show yet; raise its window only if
  // that workspace already holds something the user can act on.
  if has_pending_kind(app, kind) {
    focus_pending(app, kind);
  }
  true
}

/// Reserves the recording workspace before countdown and stream
/// initialization, so only one capture is ever being set up. A recording
/// already open does not stand in the way: it is saved in its project, and the
/// new one takes its place when it finishes.
pub fn reserve_recording(app: &AppHandle) -> Result<(), String> {
  let state = app.state::<EditorState>();
  let mut reservation = state
    .capture_reservation
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  if reservation.is_some() {
    Err("Another capture is already starting".to_owned())
  } else {
    *reservation = Some(EditorKind::Recording);
    Ok(())
  }
}

/// Screenshots append to an open screenshot workspace and are indifferent to a
/// recording waiting in its own window. Only a capture already being set up
/// stands in the way.
pub fn reserve_screenshot(app: &AppHandle) -> Result<(), String> {
  let state = app.state::<EditorState>();
  let mut reservation = state
    .capture_reservation
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  if reservation.is_some() {
    Err("Another capture is already starting".to_owned())
  } else {
    *reservation = Some(EditorKind::Screenshot);
    Ok(())
  }
}

fn release(app: &AppHandle, expected: EditorKind) {
  let state = app.state::<EditorState>();
  let mut reservation = state
    .capture_reservation
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  if *reservation == Some(expected) {
    *reservation = None;
  }
}

pub fn release_recording(app: &AppHandle) {
  release(app, EditorKind::Recording);
}

pub fn release_screenshot(app: &AppHandle) {
  release(app, EditorKind::Screenshot);
}
