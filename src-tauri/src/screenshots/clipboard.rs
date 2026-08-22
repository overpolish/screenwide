// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use chrono::Local;
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;

use super::{capture_file_stem, CapturedImage};

/// Reads the clipboard away from the tray's UI thread, then hands its image to
/// the screenshot workspace. `present_screenshot` appends when that workspace
/// already exists and opens it when it does not.
pub(crate) fn open_in_export(app: &AppHandle) {
  if let Err(error) = crate::exports::reserve_screenshot_workspace(app) {
    eprintln!("Could not open the clipboard screenshot: {error}");
    return;
  }

  let app = app.clone();
  tauri::async_runtime::spawn(async move {
    let clipboard_app = app.clone();
    let image = tauri::async_runtime::spawn_blocking(move || {
      let image = clipboard_app
        .clipboard()
        .read_image()
        .map_err(|error| error.to_string())?;
      Ok::<CapturedImage, String>(CapturedImage {
        rgba: image.rgba().to_vec(),
        width: image.width(),
        height: image.height(),
      })
    })
    .await
    .map_err(|error| error.to_string())
    .and_then(|result| result);

    let result = image.and_then(|image| {
      crate::exports::present_screenshot(&app, image, capture_file_stem(Local::now().naive_local()))
    });
    if let Err(error) = result {
      crate::exports::release_screenshot_workspace(&app);
      eprintln!("Could not open the clipboard screenshot: {error}");
    }
  });
}
