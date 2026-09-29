// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The clock that reads a fresh stroke the hand has come to rest on.
//!
//! The pointer reports nothing while it is still, so no gesture sample can
//! notice a rest. A fresh stroke is watched instead, from the sample that
//! began it, by a short-lived thread that asks the manager every few frames
//! until the stroke ends; the stroke itself decides whether it has rested
//! long enough, and what it is taken for.

use std::time::{Duration, Instant};

use tauri::{AppHandle, Manager};

use super::state::{PreviewManager, ScreenshotPreviewState};

/// How often the watch asks: well inside the hold, so a stroke is read soon
/// after its rest has lasted.
const TICK: Duration = Duration::from_millis(40);

impl PreviewManager {
  /// The fresh stroke the gesture in hand is drawing, for a watch to follow.
  pub(super) fn stroke_in_hand(&self) -> Option<String> {
    let gesture = self.annotation_gesture.as_ref()?;
    gesture
      .edit
      .holds_stroke()
      .then(|| gesture.edit.selected_id().to_owned())
  }

  /// One tick of the watch over the stroke `id`: reads it if the hand has
  /// rested on it long enough, and shows what it was taken for. Answers
  /// whether the stroke is still being drawn, and so still worth watching.
  fn hold_annotation_stroke(&mut self, id: &str, now: Instant) -> bool {
    let Some(gesture) = self.annotation_gesture.as_mut() else {
      return false;
    };
    if gesture.edit.selected_id() != id {
      return false;
    }
    let pane_index = gesture.pane_index;
    let Some(item) = self
      .output
      .as_mut()
      .and_then(|output| output.items.get_mut(pane_index as usize))
    else {
      return false;
    };
    if gesture.edit.hold(&mut item.output.annotations, now) {
      // A fresh stroke is never chosen, and nor is what it was taken for.
      self.present_annotation_gesture(pane_index, None);
    }
    true
  }
}

/// Watches the stroke `id` of session `session_id` until it ends. Waiting on
/// the manager is safe here, off AppKit's main thread; the gesture samples
/// that meet this lock held only skip a sample, which the next one covers.
pub(super) fn watch(app: AppHandle, session_id: u64, id: String) {
  std::thread::spawn(move || loop {
    std::thread::sleep(TICK);
    let state = app.state::<ScreenshotPreviewState>();
    let Ok(mut manager) = state.0.lock() else {
      return;
    };
    if manager.session_id != Some(session_id)
      || !manager.hold_annotation_stroke(&id, Instant::now())
    {
      return;
    }
  });
}
