// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::image::Image;

pub const CANCEL: &[u8] = include_bytes!("../../icons/menu/cancel.png");
pub const CLIPBOARD: &[u8] = include_bytes!("../../icons/menu/clipboard.png");
pub const DISCARD: &[u8] = include_bytes!("../../icons/menu/discard.png");
pub const OPEN: &[u8] = include_bytes!("../../icons/menu/open.png");
pub const PAUSE: &[u8] = include_bytes!("../../icons/menu/pause.png");
pub const QUIT: &[u8] = include_bytes!("../../icons/menu/quit.png");
pub const RESUME: &[u8] = include_bytes!("../../icons/menu/resume.png");
pub const RULER: &[u8] = include_bytes!("../../icons/menu/ruler.png");
pub const SETTINGS: &[u8] = include_bytes!("../../icons/menu/settings.png");
pub const STOP: &[u8] = include_bytes!("../../icons/menu/stop.png");
pub const TEXT: &[u8] = include_bytes!("../../icons/menu/text.png");

pub fn load(bytes: &'static [u8]) -> tauri::Result<Image<'static>> {
  let image = Image::from_bytes(bytes)?;
  #[cfg(target_os = "windows")]
  {
    use windows::Win32::Graphics::Gdi::{GetSysColor, COLOR_MENUTEXT};
    // Follow the native menu's text color, including high-contrast themes.
    let color = unsafe { GetSysColor(COLOR_MENUTEXT) };
    let mut rgba = image.rgba().to_vec();
    for pixel in rgba.chunks_exact_mut(4) {
      pixel[0] = color as u8;
      pixel[1] = (color >> 8) as u8;
      pixel[2] = (color >> 16) as u8;
    }
    return Ok(Image::new_owned(rgba, image.width(), image.height()));
  }
  #[cfg(not(target_os = "windows"))]
  Ok(image)
}

#[cfg(target_os = "macos")]
pub fn apply_templates(tray: &tauri::tray::TrayIcon) -> tauri::Result<()> {
  tray.with_inner_tray_icon(|inner| {
    // Tauri runs this closure on the main thread. Template images let AppKit
    // supply light/dark, selected, and disabled menu colors automatically.
    let Some(mtm) = objc2::MainThreadMarker::new() else {
      return;
    };
    let Some(menu) = inner.ns_status_item().and_then(|item| item.menu(mtm)) else {
      return;
    };
    for item in menu.itemArray() {
      if let Some(image) = item.image() {
        image.setTemplate(true);
      }
    }
  })
}
