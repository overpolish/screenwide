// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The answer a gesture gets when the window under the cursor cannot be moved.
//!
//! No session opens for such a window: there is nothing to carry, and a
//! preview that tracked the fingers would promise a placement the platform is
//! going to refuse. Instead the preview appears on the anchor showing a lock,
//! holds for a beat, and fades. The cursor is neither hidden nor pinned - the
//! hand keeps the pointer it already had.

use tauri::AppHandle;

use super::events::{emit, GlideInputEvent};
use super::{finish, finish_with_fade, BusyLease};

/// How long the lock stays on screen before the fade begins. Long enough to
/// read as a deliberate answer to the gesture, short enough that it never
/// stands between the hand and the next one.
const LOCKED_HOLD: std::time::Duration = std::time::Duration::from_millis(700);

/// Shows the lock feedback for a gesture that never became a session.
///
/// The busy lease is taken before the preview appears and released only once
/// the fade completes, so the immovable window cannot be gestured at again
/// while its own refusal is still on screen. `finish_with_fade` emits the
/// closing `End` and hides the preview, and runs the completions exactly once
/// on every path - including its own failure - so the lease is released once
/// however the fade ends.
pub(crate) fn present_locked(app: &AppHandle, session_id: u64, x: f64, y: f64) {
  let busy = BusyLease::acquire();
  if let Err(error) = present(app, session_id, x, y) {
    eprintln!("Could not present the Glide lock: {error}");
    return;
  }
  let hold_app = app.clone();
  let spawned = std::thread::Builder::new()
    .name("glide-locked-hold".to_owned())
    .spawn(move || {
      std::thread::sleep(LOCKED_HOLD);
      finish_with_fade(
        &hold_app,
        x,
        y,
        // Nothing was taken from the cursor, so nothing has to be given back.
        Box::new(|| {}),
        Box::new(move || {
          let _busy = busy;
        }),
      );
    });
  if let Err(error) = spawned {
    eprintln!("Could not hold the Glide lock open: {error}");
    finish(app, x, y, true);
  }
}

/// Places the preview on the anchor, tells it what it is showing, and reveals
/// it without blocking hover: the lock is a message, not a target.
#[cfg(target_os = "macos")]
fn present(app: &AppHandle, session_id: u64, x: f64, y: f64) -> Result<(), String> {
  let (result_tx, result_rx) = std::sync::mpsc::sync_channel(1);
  let main_app = app.clone();
  app
    .run_on_main_thread(move || {
      let _ = result_tx.send(present_on_main(&main_app, session_id, x, y));
    })
    .map_err(|error| error.to_string())?;
  result_rx
    .recv()
    .map_err(|_| "The Glide preview main-thread operation was interrupted".to_owned())?
}

#[cfg(target_os = "macos")]
fn present_on_main(app: &AppHandle, session_id: u64, x: f64, y: f64) -> Result<(), String> {
  crate::windows::position_glide_preview(app, x, y).map_err(|error| error.to_string())?;
  emit(app, GlideInputEvent::Locked { session_id })?;
  crate::windows::show_glide_preview(app, false).map_err(|error| error.to_string())
}

/// Places the preview on the anchor, tells it what it is showing, and reveals
/// it without blocking hover: the lock is a message, not a target.
#[cfg(target_os = "windows")]
fn present(app: &AppHandle, session_id: u64, x: f64, y: f64) -> Result<(), String> {
  super::place_preview_physical(app, x.round() as i32, y.round() as i32)?;
  emit(app, GlideInputEvent::Locked { session_id })?;
  crate::windows::show_glide_preview(app, false).map_err(|error| error.to_string())
}
