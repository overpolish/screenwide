// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A new version offered on the island. The island's page owns the updater:
//! it checks, and asks for the island when it finds a version to offer. The
//! island comes out for it only while it is free: nothing recorded or
//! exported, and the island out for nothing else. Anything else claiming the
//! island ends the offer (`IslandClaim::Update`).

use tauri::AppHandle;

use crate::app_windows::island::{self, IslandClaim};
#[cfg(target_os = "windows")]
use crate::microsoft_store::updates as store;

/// Where this build's updates come from: its own updater, the Microsoft
/// Store for the Store package (`store_update`), or nowhere for a
/// development build.
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

/// The update the Microsoft Store has for the Store package, with its
/// version when the Store's catalog says which; null when it has none.
#[cfg(target_os = "windows")]
#[tauri::command]
pub async fn store_update() -> Result<Option<store::StoreUpdate>, String> {
  store::find().await
}

/// Only Windows has the Microsoft Store.
#[cfg(not(target_os = "windows"))]
#[tauri::command]
pub fn store_update() -> Option<()> {
  None
}

/// Opens Screenwide's page in the Microsoft Store, where the user installs
/// the update. The app does not install it itself: that would close it from
/// under the user and needs the Store's own window parented to one of ours.
#[tauri::command]
pub fn open_store_page(app: AppHandle) -> Result<(), String> {
  #[cfg(target_os = "windows")]
  {
    use tauri_plugin_opener::OpenerExt;

    app
      .opener()
      .open_url(store::STORE_PAGE_URL, None::<&str>)
      .map_err(|error| error.to_string())
  }
  #[cfg(not(target_os = "windows"))]
  {
    let _ = app;
    Err("The Microsoft Store is only on Windows".to_owned())
  }
}

/// Brings the island out at the top of the screen to offer an update, if it
/// is free. Returns whether it came out.
#[tauri::command]
pub fn present_update_island(app: AppHandle) -> bool {
  let is_free = crate::recording::is_idle(&app)
    && !crate::editor::save::is_exporting(&app)
    && island::is_unclaimed();
  if is_free {
    island::claim(&app, IslandClaim::Update);
  }
  is_free
}

/// The update was put off or skipped: the island tucks away.
#[tauri::command]
pub fn withdraw_update_island(app: AppHandle) {
  island::release(&app, IslandClaim::Update);
}
