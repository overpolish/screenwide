// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The island while Quick Screenshot's overlay is up: it comes out at the top
//! of the screen as the shortcut opens the overlay, says where the area drawn
//! will go, and turns into the copied shot once one is taken, or tucks away
//! when the session is cancelled. Its close button ends the session as Escape
//! does.
//!
//! A still leaves Screenwide's windows out unless the Ruler is in it or the
//! user records Screenwide's windows; then the island is kept out of the shot
//! while it stays on screen (`screenshots::capture`).

use std::sync::{Mutex, PoisonError};

use serde::{Deserialize, Serialize};
#[cfg(target_os = "windows")]
use tauri::Manager;
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
  /// Copied and previewed.
  Screenshot,
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
  // Before the island changes, so a recording under way never takes a frame
  // of the still's island.
  app_windows::island_overlay::keep_still_out_of_recording(&app, true);
  tell(&app, Some(destination));
  // A shot copied before gives its island to this session, which reshapes
  // it in place.
  super::copied::withdraw(&app);
  let shown = app_windows::island::place(&app, IslandPlacement::Top)
    .and_then(|()| app_windows::island::show(&app));
  if let Err(error) = shown {
    eprintln!("Could not show the island for Quick Screenshot: {error}");
  }
}

/// Stands the island over the overlay once it is up. Windows orders topmost
/// windows by whichever was raised last, and the overlay is shown after the
/// island and raised again by every press on it, so there the island is
/// owned by it, as the Ruler's and Text Recognition's are. macOS already
/// orders the island above the overlay by level.
#[cfg(target_os = "windows")]
pub(crate) fn stand_over_overlay(app: &AppHandle) {
  if !is_up() {
    return;
  }
  if let Some(overlay) = app.get_webview_window(WindowLabel::RegionSelector.as_str()) {
    app_windows::island_overlay::stand_over(app, Some(&overlay));
  }
}

/// The shot this session took is copied, and its island takes this one's
/// place out where it is: only what it says changes.
pub(crate) fn give_way(app: &AppHandle) {
  if !is_up() {
    return;
  }
  tell(app, None);
  // Released before the overlay can close, which would take the island it
  // owns with it.
  #[cfg(target_os = "windows")]
  app_windows::island_overlay::stand_over(app, None);
}

/// The session ended. The island tucks away, unless the shot just copied, the
/// recording or its controls keep it, which take it back where they show.
pub(crate) fn withdraw(app: &AppHandle) {
  if !is_up() {
    return;
  }
  // Released before the overlay can close, which would take the island it
  // owns with it.
  #[cfg(target_os = "windows")]
  app_windows::island_overlay::stand_over(app, None);
  // The copied shot's island takes this one's place, out where it is, once
  // its picture is made (`give_way`); until then this one keeps saying what
  // it said, so the island never shows nothing in between.
  if super::copied::is_up() {
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
  app_windows::island_overlay::keep_still_out_of_recording(app, false);
}

#[tauri::command]
pub fn quick_screenshot_island_state() -> Option<QuickScreenshotDestination> {
  *shown()
}
