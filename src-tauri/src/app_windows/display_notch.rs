// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Whether this machine's displays have a notch, for a page that draws the
//! island as it would come out here, such as a setting's preview.

use serde::Serialize;
use tauri::AppHandle;

use super::island::IslandNotch;

/// The notch of the display the island comes out on, and of the main
/// display, the one with the menu bar; none where a display has no notch,
/// and none on Windows.
#[derive(Clone, Copy, Debug, Default, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct DisplayNotches {
  pub island: Option<IslandNotch>,
  pub main: Option<IslandNotch>,
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub async fn display_notches(app: AppHandle) -> DisplayNotches {
  use super::island_macos::{island_screen, notch_of, target_display};
  let island = target_display(&app);
  dispatch2::run_on_main(move |mtm| DisplayNotches {
    island: island_screen(mtm, island).as_deref().and_then(notch_of),
    main: island_screen(mtm, None).as_deref().and_then(notch_of),
  })
}

#[cfg(not(target_os = "macos"))]
#[tauri::command]
pub async fn display_notches(_app: AppHandle) -> DisplayNotches {
  DisplayNotches::default()
}
