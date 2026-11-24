// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::OnceLock;

use tauri::AppHandle;

use super::Picture;

unsafe extern "C" {
  fn screenwide_menu_freeze_observe(changed: extern "C" fn(bool));
  fn screenwide_menu_freeze_menu_open() -> bool;
  fn screenwide_menu_freeze_close_menu();
  fn screenwide_menu_freeze_show(
    rgba: *const u8,
    pixels_wide: usize,
    pixels_high: usize,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
  );
  fn screenwide_menu_freeze_hide();
}

static APP: OnceLock<AppHandle> = OnceLock::new();

extern "C" fn menu_changed(open: bool) {
  if let Some(app) = APP.get() {
    crate::shortcuts::menu_keys::menu_changed(app, open);
  }
}

/// On the main thread, where AppKit tells of menus opening and closing.
pub(super) fn observe(app: &AppHandle) {
  let _ = APP.set(app.clone());
  unsafe { screenwide_menu_freeze_observe(menu_changed) };
}

pub(super) fn menu_open() -> bool {
  unsafe { screenwide_menu_freeze_menu_open() }
}

/// On the main thread.
pub(super) fn close_menu() {
  unsafe { screenwide_menu_freeze_close_menu() };
}

/// On the main thread.
pub(super) fn show(pictures: &[Picture]) {
  for picture in pictures {
    // AppKit places windows in points; the display's frame is the physical
    // one Tauri reports, over its own scale.
    let scale = picture.scale;
    unsafe {
      screenwide_menu_freeze_show(
        picture.image.rgba.as_ptr(),
        picture.image.width as usize,
        picture.image.height as usize,
        f64::from(picture.position.x) / scale,
        f64::from(picture.position.y) / scale,
        f64::from(picture.size.width) / scale,
        f64::from(picture.size.height) / scale,
      );
    }
  }
}

/// On the main thread.
pub(super) fn hide() {
  unsafe { screenwide_menu_freeze_hide() };
}
