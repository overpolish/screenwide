// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The island while Quick Screenshot's overlay is up: it comes out at the top
//! of the screen as the shortcut opens the overlay, says where the area drawn
//! will go, and turns into the copied shot once one is taken, or tucks away
//! when the session is cancelled. Its close button ends the session as Escape
//! does.
//!
//! A still leaves Screenwide's windows out unless the Ruler or a frozen menu
//! (`menu_freeze`) is in it or the user records Screenwide's windows; then
//! the island is kept out of the shot while it stays on screen
//! (`screenshots::capture`).

use serde::{Deserialize, Serialize};
use tauri::AppHandle;
#[cfg(target_os = "windows")]
use tauri::Manager;
use ts_rs::TS;

use crate::app_windows;
use crate::app_windows::island::{IslandClaim, IslandSlot};
#[cfg(target_os = "windows")]
use crate::app_windows::WindowLabel;

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
static ISLAND: IslandSlot<QuickScreenshotDestination> =
  IslandSlot::new(IslandClaim::QuickScreenshot, "quick-screenshot://island");

/// Whether a session's island is up, so the recording controls it replaces
/// leave the island's window to it.
pub(crate) fn is_up() -> bool {
  ISLAND.is_up()
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
  ISLAND.tell(&app, destination);
  // A shot copied before gives its island to this session, which reshapes
  // it in place.
  super::copied::withdraw(&app);
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
  ISLAND.withdraw(app);
  // Released before the overlay can close, which would take the island it
  // owns with it.
  #[cfg(target_os = "windows")]
  app_windows::island_overlay::stand_over(app, None);
}

/// The session ended. The island goes back to whatever else holds it, or
/// tucks away, unless the shot just copied takes its place.
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
  ISLAND.withdraw(app);
  app_windows::island_overlay::keep_still_out_of_recording(app, false);
}

#[tauri::command]
pub fn quick_screenshot_island_state() -> Option<QuickScreenshotDestination> {
  ISLAND.current()
}
