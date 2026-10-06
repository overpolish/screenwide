// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The replay buffer's place in the tray: saving and turning it off while it
//! runs, and saying that it does in the tooltip.

use tauri::menu::{IconMenuItemBuilder, MenuBuilder};
use tauri::{AppHandle, Wry};

use super::icons;
use crate::recording::replay::ReplayStatus;

pub(super) const SAVE_MENU_ID: &str = "save-replay";
pub(super) const STOP_MENU_ID: &str = "stop-replay";

/// Adds the replay controls while the buffer is on or starting.
pub(super) fn append<'m>(
  app: &AppHandle,
  builder: MenuBuilder<'m, Wry, AppHandle>,
) -> tauri::Result<MenuBuilder<'m, Wry, AppHandle>> {
  let replay = crate::recording::replay::snapshot(app);
  if replay.status == ReplayStatus::Off {
    return Ok(builder);
  }
  let mut save = IconMenuItemBuilder::with_id(SAVE_MENU_ID, "Save Replay")
    .icon(icons::load(icons::REPLAY)?)
    .enabled(replay.status == ReplayStatus::On && !replay.saving);
  if let Some(shortcut) =
    crate::shortcuts::shortcut_for(app, crate::shortcuts::ShortcutAction::SaveReplay)
  {
    save = save.accelerator(shortcut);
  }
  let save = save.build(app)?;
  Ok(builder.separator().item(&save).icon(
    STOP_MENU_ID,
    "Turn Off Replay Buffer",
    icons::load(icons::STOP)?,
  ))
}

/// Handles a replay item, reporting whether `id` was one.
pub(super) fn handle(app: &AppHandle, id: &str) -> bool {
  match id {
    SAVE_MENU_ID => {
      if let Err(error) = crate::recording::replay::save(app) {
        eprintln!("Could not save the replay from the tray: {error}");
      }
      true
    }
    STOP_MENU_ID => {
      crate::recording::replay::stop(app);
      true
    }
    _ => false,
  }
}
