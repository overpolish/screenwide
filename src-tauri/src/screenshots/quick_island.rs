// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The island while Quick Screenshot's overlay is up: it comes out at the top
//! of the screen as the shortcut opens the overlay, says where the area drawn
//! will go, and tucks away when the session ends, taken or cancelled. Its
//! close button ends the session as Escape does.
//!
//! A still leaves Screenwide's windows out unless the Ruler is in it or the
//! user records Screenwide's windows; only then does the island step out of
//! the shot at once rather than tuck away after it.

use std::sync::{Mutex, PoisonError};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use ts_rs::TS;

use crate::app_windows::island_placement::IslandPlacement;
use crate::app_windows::{self, WindowLabel};

const EVENT: &str = "quick-screenshot://island";

/// Where the area drawn goes, as the island says it.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum QuickScreenshotDestination {
  Editor,
  Clipboard,
  Reference,
}

/// What the island was last told; none while no session is on it.
static SHOWN: Mutex<Option<QuickScreenshotDestination>> = Mutex::new(None);

fn shown() -> std::sync::MutexGuard<'static, Option<QuickScreenshotDestination>> {
  SHOWN.lock().unwrap_or_else(PoisonError::into_inner)
}

fn tell(app: &AppHandle, destination: Option<QuickScreenshotDestination>) {
  let mut shown = shown();
  if *shown == destination {
    return;
  }
  *shown = destination;
  drop(shown);
  // Advisory: a screenshot must never fail because the island did not hear.
  let _ = app.emit_to(WindowLabel::Island.as_str(), EVENT, destination);
}

/// Whether a session's island is up, so the recording controls it replaces
/// leave the island's window to it.
pub(crate) fn is_up() -> bool {
  shown().is_some()
}

/// The overlay is up for a session going to `destination`, or the other
/// shortcut changed where it goes. Called once the Ruler and Annotate have
/// stepped back for the session, as both put the island away as they do.
#[tauri::command]
pub fn show_quick_screenshot_island(app: AppHandle, destination: QuickScreenshotDestination) {
  if !app_windows::region::is_screenshot_region_session() {
    return;
  }
  tell(&app, Some(destination));
  let shown = app_windows::island::place(&app, IslandPlacement::Top)
    .and_then(|()| app_windows::island::show(&app));
  if let Err(error) = shown {
    eprintln!("Could not show the island for Quick Screenshot: {error}");
  }
}

/// The shutter is about to fire. A still that takes Screenwide's windows
/// would take the island too, so it goes at once.
pub(crate) fn before_shutter(app: &AppHandle, includes_own_windows: bool) {
  if !includes_own_windows || !is_up() {
    return;
  }
  if let Err(error) = app_windows::island::hide_at_once(app) {
    eprintln!("Could not take the island out of the screenshot: {error}");
  }
}

/// The session ended. The island tucks away, unless the recording or its
/// controls keep it, which take it back where they show.
pub(crate) fn withdraw(app: &AppHandle) {
  if !is_up() {
    return;
  }
  tell(app, None);
  let result = if crate::recording::is_idle(app) && !app_windows::is_recording_ui_visible() {
    app_windows::island::hide(app)
  } else {
    app_windows::island::hand_over(app, IslandPlacement::Top)
  };
  if let Err(error) = result {
    eprintln!("Could not put the island away after Quick Screenshot: {error}");
  }
}

#[tauri::command]
pub fn quick_screenshot_island_state() -> Option<QuickScreenshotDestination> {
  *shown()
}
