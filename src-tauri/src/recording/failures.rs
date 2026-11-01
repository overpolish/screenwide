// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Failures the recording island says: a recording that could not start or
//! ran into a problem, and a replay buffer that could not turn on, stopped or
//! could not save.
//!
//! They queue here rather than in the island's page, so none is lost to a
//! page that is not listening yet, and the island shows the one in front
//! until it is dismissed. A failure can come from the tray, a shortcut or the
//! computer waking with nothing of Screenwide's on screen, so the island
//! comes out for it.

use std::collections::VecDeque;
use std::sync::{Mutex, MutexGuard};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use ts_rs::TS;

use crate::app_windows::{self, WindowLabel};

/// Sent to the island each time the failure in front changes.
const FAILURES_EVENT: &str = "recording://failures";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RecordingFailure {
  /// Names this failure, so dismissing it never takes the one behind it.
  pub id: u32,
  /// What failed, in the user's language.
  pub title: String,
  /// Why, which stays in English: it comes from the capture layers.
  pub message: String,
}

#[derive(Default)]
struct Queue {
  last_id: u32,
  failures: VecDeque<RecordingFailure>,
}

impl Queue {
  /// Queues a failure unless the same one is already showing or waiting, and
  /// reports whether it was queued. Retrying a start that keeps failing for
  /// one reason is one problem, and says so once.
  fn enqueue(&mut self, title: &str, message: &str) -> bool {
    if self
      .failures
      .iter()
      .any(|failure| failure.title == title && failure.message == message)
    {
      return false;
    }
    self.last_id = self.last_id.wrapping_add(1);
    self.failures.push_back(RecordingFailure {
      id: self.last_id,
      title: title.to_owned(),
      message: message.to_owned(),
    });
    true
  }

  /// Takes failure `id` away, reporting whether it was there.
  fn dismiss(&mut self, id: u32) -> bool {
    let before = self.failures.len();
    self.failures.retain(|failure| failure.id != id);
    self.failures.len() != before
  }
}

#[derive(Default)]
pub struct RecordingFailures(Mutex<Queue>);

fn queue(app: &AppHandle) -> MutexGuard<'_, Queue> {
  app
    .state::<RecordingFailures>()
    .inner()
    .0
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// The island asks for the failure in front again, so an event that crosses
/// another never leaves it showing the wrong one.
fn changed(app: &AppHandle) {
  let _ = app.emit_to(WindowLabel::Island.as_str(), FAILURES_EVENT, ());
}

/// Tells the user something failed, on the island. Callable from any thread,
/// with any lock held: the island is brought out on the main thread.
pub(super) fn report(app: &AppHandle, title: &str, message: &str) {
  if queue(app).enqueue(title, message) {
    changed(app);
  }
  // Even a failure already waiting brings the island out again, in case it
  // was put away since. During a recording it is out already, and a start
  // that failed brings it back itself once the recording is idle again.
  let task_app = app.clone();
  let _ = app.run_on_main_thread(move || {
    if super::is_idle(&task_app) && !app_windows::is_recording_ui_visible() {
      if let Err(error) = app_windows::show_recording_ui(&task_app) {
        eprintln!("Could not show the recording island for a failure: {error}");
      }
    }
  });
}

/// Forgets every failure: a new recording is starting, and anything that
/// fails again says so again.
pub(super) fn clear(app: &AppHandle) {
  let had_any = {
    let mut queue = queue(app);
    let had_any = !queue.failures.is_empty();
    queue.failures.clear();
    had_any
  };
  if had_any {
    changed(app);
  }
}

/// The failure the island shows, if any.
#[tauri::command]
pub fn recording_failure(app: AppHandle) -> Option<RecordingFailure> {
  queue(&app).failures.front().cloned()
}

/// The user has read failure `id`; the next one, if any, takes its place.
#[tauri::command]
pub fn dismiss_recording_failure(app: AppHandle, id: u32) {
  if queue(&app).dismiss(id) {
    changed(&app);
  }
}

#[cfg(test)]
mod tests {
  use super::Queue;

  #[test]
  fn a_repeated_failure_is_said_once() {
    let mut queue = Queue::default();

    assert!(queue.enqueue("Recording could not start", "No camera"));
    for _ in 0..4 {
      assert!(!queue.enqueue("Recording could not start", "No camera"));
    }

    assert_eq!(queue.failures.len(), 1);
  }

  #[test]
  fn different_failures_wait_their_turn() {
    let mut queue = Queue::default();

    assert!(queue.enqueue("Recording ran into a problem", "Disk full"));
    assert!(queue.enqueue("Recording could not finish", "Disk full"));

    assert_eq!(queue.failures.len(), 2);
    let front = queue.failures[0].id;
    assert!(queue.dismiss(front));
    assert!(!queue.dismiss(front));
    assert_eq!(
      queue.failures.front().map(|failure| failure.title.as_str()),
      Some("Recording could not finish")
    );
  }
}
