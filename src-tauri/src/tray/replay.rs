// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The replay buffer's place in the tray: turning it on, and saving and
//! turning it off while it runs.

use tauri::menu::{IconMenuItemBuilder, MenuBuilder};
use tauri::{AppHandle, Emitter, Wry};

use super::icons;
use crate::app_windows::WindowLabel;
use crate::recording::replay::ReplayStatus;

pub(super) const SAVE_MENU_ID: &str = "save-replay";
pub(super) const START_MENU_ID: &str = "start-replay";
pub(super) const STOP_MENU_ID: &str = "stop-replay";

/// Asks the recording bar to turn the buffer on. The bar's settings live in
/// its webview, so it builds the options, exactly as its own toggle does.
const START_REQUESTED_EVENT: &str = "replay://start-requested";

/// Adds the replay controls where this platform has a replay buffer.
pub(super) fn append<'m>(
  app: &AppHandle,
  builder: MenuBuilder<'m, Wry, AppHandle>,
) -> tauri::Result<MenuBuilder<'m, Wry, AppHandle>> {
  let replay = crate::recording::replay::snapshot(app);
  if !replay.available {
    return Ok(builder);
  }
  if replay.status == ReplayStatus::Off {
    return Ok(builder.separator().icon(
      START_MENU_ID,
      "Turn On Replay Buffer",
      icons::load(icons::REPLAY)?,
    ));
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
    START_MENU_ID => {
      request_start(app);
      true
    }
    STOP_MENU_ID => {
      crate::recording::replay::stop(app);
      true
    }
    _ => false,
  }
}

fn request_start(app: &AppHandle) {
  // Without permission the capture cannot start; say why instead.
  #[cfg(target_os = "macos")]
  if !crate::permissions::has_required_recording_permissions(app) {
    let _ = crate::permissions::show_permissions_window(app);
    return;
  }
  if let Err(error) = app.emit_to(
    WindowLabel::RecordingBar.as_str(),
    START_REQUESTED_EVENT,
    (),
  ) {
    eprintln!("Could not ask the recording bar to turn on the replay buffer: {error}");
  }
}
