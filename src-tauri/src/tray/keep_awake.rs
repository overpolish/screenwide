// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Keep Awake's place in the tray: one item, saying what choosing it does.

use tauri::menu::{IconMenuItem, IconMenuItemBuilder};
use tauri::{AppHandle, Wry};

use super::icons;
use crate::shortcuts::{shortcut_for, ShortcutAction};

pub(super) const ON_MENU_ID: &str = "keep-awake-on";
pub(super) const OFF_MENU_ID: &str = "keep-awake-off";

pub(super) fn item(app: &AppHandle) -> tauri::Result<IconMenuItem<Wry>> {
  let mut item = if crate::keep_awake::is_on() {
    IconMenuItemBuilder::with_id(OFF_MENU_ID, screenwide_i18n::t!("tray-keep-awake-off"))
      .icon(icons::load(icons::STOP)?)
  } else {
    IconMenuItemBuilder::with_id(ON_MENU_ID, screenwide_i18n::t!("tray-keep-awake-on"))
      .icon(icons::load(icons::KEEP_AWAKE)?)
  };
  if let Some(shortcut) = shortcut_for(app, ShortcutAction::ToggleKeepAwake) {
    item = item.accelerator(shortcut);
  }
  item.build(app)
}

/// Handles a Keep Awake item, reporting whether `id` was one.
pub(super) fn handle(app: &AppHandle, id: &str) -> bool {
  match id {
    ON_MENU_ID => crate::keep_awake::set(app, true),
    OFF_MENU_ID => crate::keep_awake::set(app, false),
    _ => return false,
  }
  true
}
