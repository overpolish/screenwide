// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The island while a picture is dragged with Option held: it comes out at
//! the top of the screen to say that letting go keeps the picture as a
//! reference (`drop`), and tucks away as the drag ends or Option is let go,
//! handing the island back to the recording or its controls if they hold
//! it.

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Emitter};

use crate::app_windows::island_placement::IslandPlacement;
use crate::app_windows::{self, WindowLabel};

const EVENT: &str = "reference-drop://island";

static SHOWN: AtomicBool = AtomicBool::new(false);

fn tell(app: &AppHandle, shown: bool) -> bool {
  if SHOWN.swap(shown, Ordering::AcqRel) == shown {
    return false;
  }
  // Advisory: a drop must never fail because the island did not hear.
  let _ = app.emit_to(WindowLabel::Island.as_str(), EVENT, shown);
  true
}

/// A picture is dragged with Option held, so letting go keeps it.
pub(crate) fn show(app: &AppHandle) {
  if !tell(app, true) {
    return;
  }
  let shown = app_windows::island::place(app, IslandPlacement::Top)
    .and_then(|()| app_windows::island::show(app));
  if let Err(error) = shown {
    eprintln!("Could not show the island for a reference drop: {error}");
  }
}

/// The drag ended or Option was let go. The island tucks away, unless the
/// recording or its controls keep it, which take it back where they show.
pub(crate) fn withdraw(app: &AppHandle) {
  if !tell(app, false) {
    return;
  }
  let result = if crate::recording::is_idle(app) && !app_windows::is_recording_ui_visible() {
    app_windows::island::hide(app)
  } else {
    app_windows::island::hand_over(app, IslandPlacement::Top)
  };
  if let Err(error) = result {
    eprintln!("Could not put the island away after a reference drop: {error}");
  }
}

/// Whether the hint is out, for an island window that opens while it is.
#[tauri::command]
pub fn reference_drop_island_state() -> bool {
  SHOWN.load(Ordering::Acquire)
}
