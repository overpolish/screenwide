// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A new version offered on the island. The island's page owns the updater:
//! it checks, and asks for the island when it finds a version to offer. The
//! island comes out for it only while it is free: nothing recorded or
//! exported, and the island out for nothing else. The recording controls
//! taking the island end the offer (`updates://island`), as does the island
//! going away, which its page hears for itself (`island://hide`).

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Emitter};

use crate::app_windows::island_placement::IslandPlacement;
use crate::app_windows::{self, WindowLabel};

const EVENT: &str = "updates://island";

/// The island was brought out to offer an update; it still offers it only
/// while it is out.
static ON_ISLAND: AtomicBool = AtomicBool::new(false);

fn on_island() -> bool {
  ON_ISLAND.load(Ordering::Relaxed) && app_windows::island::is_shown()
}

fn tell(app: &AppHandle, on_island: bool) {
  if ON_ISLAND.swap(on_island, Ordering::Relaxed) != on_island {
    let _ = app.emit_to(WindowLabel::Island.as_str(), EVENT, on_island);
  }
}

/// Something else is taking the island: the offer is over.
pub(crate) fn give_way(app: &AppHandle) {
  tell(app, false);
}

/// Where this build's updates come from: its own updater, the Microsoft
/// Store for the Store package, or nowhere for a development build.
#[derive(Clone, Copy, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum UpdateSource {
  Updater,
  Store,
  Development,
}

#[tauri::command]
pub fn update_source() -> UpdateSource {
  if crate::microsoft_store::is_packaged() {
    UpdateSource::Store
  } else if cfg!(debug_assertions) {
    UpdateSource::Development
  } else {
    UpdateSource::Updater
  }
}

/// Brings the island out at the top of the screen to offer an update, if it
/// is free. Returns whether it came out.
#[tauri::command]
pub fn present_update_island(app: AppHandle) -> tauri::Result<bool> {
  let is_free = crate::recording::is_idle(&app)
    && !crate::editor::save::is_exporting(&app)
    && !app_windows::is_recording_ui_visible()
    && !app_windows::island::is_shown();
  if !is_free {
    return Ok(false);
  }
  tell(&app, true);
  app_windows::island::place(&app, IslandPlacement::Top)?;
  app_windows::island::show(&app)?;
  Ok(true)
}

/// The update was put off or skipped: the island tucks away.
#[tauri::command]
pub fn withdraw_update_island(app: AppHandle) -> tauri::Result<()> {
  if !on_island() {
    return Ok(());
  }
  give_way(&app);
  app_windows::island::hide(&app)
}

/// Whether the island is out offering an update, for a page that loads after
/// it was told.
#[tauri::command]
pub fn update_island_shown() -> bool {
  on_island()
}
