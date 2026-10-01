// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The clock that reads a fresh stroke the hand has come to rest on, in a
//! recording. The twin of the screenshot editor's `annotation_hold`: the
//! pointer reports nothing while it is still, so a short-lived thread asks
//! the manager every few frames until the stroke ends.

use std::time::{Duration, Instant};

use super::*;

/// How often the watch asks: well inside the hold, so a stroke is read soon
/// after its rest has lasted.
const TICK: Duration = Duration::from_millis(40);

impl PreviewPlayerManager {
  /// The fresh stroke the gesture in hand is drawing, for a watch to follow.
  pub(super) fn stroke_in_hand(&self) -> Option<String> {
    let gesture = self.annotation.gesture.as_ref()?;
    gesture
      .edit
      .holds_stroke()
      .then(|| gesture.edit.selected_id().to_owned())
  }

  /// One tick of the watch over the stroke `id`: reads it if the hand has
  /// rested on it long enough, and shows what it was taken for in the clips
  /// the drag draws through. Answers whether the stroke is still being
  /// drawn, and so still worth watching.
  fn hold_annotation_stroke(&mut self, id: &str, now: Instant) -> bool {
    let Some(gesture) = self.annotation.gesture.as_mut() else {
      return false;
    };
    if gesture.edit.selected_id() != id {
      return false;
    }
    if !gesture.edit.hold(&mut gesture.working, now) {
      return true;
    }
    let (pane, position) = (gesture.pane, gesture.position);
    let clips = gesture::provisional_clips(&gesture.before, &gesture.working, pane, position);
    if let Some(Ok(mut shown)) = self
      .sources
      .as_ref()
      .map(|sources| sources.annotation_clips.write())
    {
      *shown = clips;
    }
    self.publish_annotation_handles();
    if !self.redraw_annotation_frame(pane, position) {
      let _ = self.restart(PlaybackMode::InteractiveStill);
    }
    true
  }
}

/// Watches the stroke `id` while `clips` are the clips the manager draws,
/// until the stroke ends. Waiting on the manager is safe here, off AppKit's
/// main thread; the gesture samples that meet this lock held only skip a
/// sample, which the next one covers.
pub(super) fn watch(app: AppHandle, clips: Arc<RwLock<Vec<RecordingAnnotationClip>>>, id: String) {
  std::thread::spawn(move || loop {
    std::thread::sleep(TICK);
    let state = app.state::<RecordingPreviewPlayerState>();
    let Ok(mut manager) = state.0.lock() else {
      return;
    };
    let current = manager
      .sources
      .as_ref()
      .is_some_and(|sources| Arc::ptr_eq(&sources.annotation_clips, &clips));
    if !current || !manager.hold_annotation_stroke(&id, Instant::now()) {
      return;
    }
  });
}
