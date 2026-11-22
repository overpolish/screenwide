// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::atomic::Ordering;

use tauri::{AppHandle, Manager};

use super::{window, EditorKind, EditorState};

pub(super) fn focus_pending(app: &AppHandle, kind: EditorKind) {
  window::show(app, kind, |_| {});
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

fn is_reserved(app: &AppHandle, kind: EditorKind) -> bool {
  app
    .state::<EditorState>()
    .slot(kind)
    .reserved
    .load(Ordering::Acquire)
}

/// Screenshot tools open over whatever else is waiting, a recording under way
/// included: each workspace has its own window, so only a still already in
/// flight blocks a new one.
pub fn focus_if_screenshot_blocked(app: &AppHandle) -> bool {
  if !is_reserved(app, EditorKind::Screenshot) {
    return false;
  }
  // A reservation of its own has nothing to show yet; raise its window only if
  // that workspace already holds something the user can act on.
  if has_pending_kind(app, EditorKind::Screenshot) {
    focus_pending(app, EditorKind::Screenshot);
  }
  true
}

/// Takes `kind`'s reservation, unless a capture of that kind is already being
/// set up.
fn reserve(app: &AppHandle, kind: EditorKind) -> Result<(), String> {
  app
    .state::<EditorState>()
    .slot(kind)
    .reserved
    .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
    .map(|_| ())
    .map_err(|_| "Another capture is already starting".to_owned())
}

/// Reserves the recording workspace before countdown and stream
/// initialization, so only one recording is ever being set up. A recording
/// already open does not stand in the way: it is saved in its project, and the
/// new one takes its place when it finishes.
pub fn reserve_recording(app: &AppHandle) -> Result<(), String> {
  reserve(app, EditorKind::Recording)?;
  // Built while the recording runs, so it is ready when the recording ends.
  window::prepare(app, EditorKind::Recording);
  Ok(())
}

/// Screenshots append to an open screenshot workspace and are indifferent to a
/// recording, under way or waiting in its own window. Only a still already
/// being taken stands in the way.
pub fn reserve_screenshot(app: &AppHandle) -> Result<(), String> {
  reserve(app, EditorKind::Screenshot)
}

/// Lets `kind`'s reservation go: the capture arrived in its workspace, or came
/// to nothing.
pub(super) fn settle(app: &AppHandle, kind: EditorKind) {
  app
    .state::<EditorState>()
    .slot(kind)
    .reserved
    .store(false, Ordering::Release);
}

fn release(app: &AppHandle, expected: EditorKind) {
  let was_reserved = app
    .state::<EditorState>()
    .slot(expected)
    .reserved
    .swap(false, Ordering::AcqRel);
  // The window built for a capture that came to nothing goes with it.
  if was_reserved && !has_pending_kind(app, expected) {
    window::close(app, expected);
  }
}

pub fn release_recording(app: &AppHandle) {
  release(app, EditorKind::Recording);
}

pub fn release_screenshot(app: &AppHandle) {
  release(app, EditorKind::Screenshot);
}

/// Whether the screenshot editor's window is wanted though hidden: it holds
/// a screenshot, or one is being taken.
pub(crate) fn screenshot_window_needed(app: &AppHandle) -> bool {
  is_reserved(app, EditorKind::Screenshot) || has_pending_kind(app, EditorKind::Screenshot)
}
