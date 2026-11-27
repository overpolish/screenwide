// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Keep Awake's place in the tray: an item for Keep Awake and one for Keep
//! Screen On, each saying what choosing it does.

use tauri::menu::{IconMenuItem, IconMenuItemBuilder};
use tauri::{AppHandle, Wry};

use super::icons;
use crate::shortcuts::{shortcut_for, ShortcutAction};

pub(super) const ON_MENU_ID: &str = "keep-awake-on";
pub(super) const OFF_MENU_ID: &str = "keep-awake-off";
pub(super) const SCREEN_ON_MENU_ID: &str = "keep-screen-on-on";
pub(super) const SCREEN_OFF_MENU_ID: &str = "keep-screen-on-off";

pub(super) fn item(app: &AppHandle) -> tauri::Result<IconMenuItem<Wry>> {
  let item = if crate::keep_awake::is_on() {
    IconMenuItemBuilder::with_id(OFF_MENU_ID, screenwide_i18n::t!("tray-keep-awake-off"))
      .icon(icons::load(icons::STOP)?)
  } else {
    IconMenuItemBuilder::with_id(ON_MENU_ID, screenwide_i18n::t!("tray-keep-awake-on"))
      .icon(icons::load(icons::KEEP_AWAKE)?)
  };
  with_shortcut(app, item, ShortcutAction::ToggleKeepAwake)
}

pub(super) fn screen_on_item(app: &AppHandle) -> tauri::Result<IconMenuItem<Wry>> {
  let item = if crate::keep_awake::is_screen_on() {
    IconMenuItemBuilder::with_id(
      SCREEN_OFF_MENU_ID,
      screenwide_i18n::t!("tray-keep-screen-on-off"),
    )
    .icon(icons::load(icons::STOP)?)
  } else {
    IconMenuItemBuilder::with_id(
      SCREEN_ON_MENU_ID,
      screenwide_i18n::t!("tray-keep-screen-on-on"),
    )
    .icon(icons::load(icons::KEEP_SCREEN_ON)?)
  };
  with_shortcut(app, item, ShortcutAction::ToggleKeepScreenOn)
}

fn with_shortcut(
  app: &AppHandle,
  mut item: IconMenuItemBuilder<'_>,
  action: ShortcutAction,
) -> tauri::Result<IconMenuItem<Wry>> {
  if let Some(shortcut) = shortcut_for(app, action) {
    item = item.accelerator(shortcut);
  }
  item.build(app)
}

/// Handles a Keep Awake or Keep Screen On item, reporting whether `id` was
/// one.
pub(super) fn handle(app: &AppHandle, id: &str) -> bool {
  match id {
    ON_MENU_ID => crate::keep_awake::set(app, true),
    OFF_MENU_ID => crate::keep_awake::set(app, false),
    SCREEN_ON_MENU_ID => crate::keep_awake::set_screen_on(app, true),
    SCREEN_OFF_MENU_ID => crate::keep_awake::set_screen_on(app, false),
    _ => return false,
  }
  true
}
