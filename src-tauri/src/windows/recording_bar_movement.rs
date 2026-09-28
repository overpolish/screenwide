// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn contain_recording_bar(app: &AppHandle) -> tauri::Result<()> {
  let bar = app
    .get_webview_window(WindowLabel::RecordingBar.as_str())
    .ok_or_else(|| tauri::Error::WindowNotFound)?;
  contain_window_on_its_monitor(app, &bar)
}

pub fn manage_recording_bar_movement(app: &AppHandle) {
  let Some(window) = app.get_webview_window(WindowLabel::RecordingBar.as_str()) else {
    return;
  };
  // AppKit repositions the source selector itself, so the handler captures
  // nothing there.
  #[cfg(not(target_os = "macos"))]
  let app = app.clone();

  window.on_window_event(move |event| {
    let WindowEvent::Moved(_) = event else {
      return;
    };

    // AppKit moves the source selector as a native child of the bar, avoiding
    // the visible delay caused by chasing Moved events with a second window.
    #[cfg(not(target_os = "macos"))]
    let _ = source_selector::reposition(&app);

    #[cfg(target_os = "windows")]
    watch_for_recording_bar_mouse_up(app.clone());
  });
}

#[cfg(target_os = "windows")]
pub(super) fn watch_for_recording_bar_mouse_up(app: AppHandle) {
  drag_release::after_mouse_up(&BAR_DRAG_ACTIVE, move || {
    let _ = finish_recording_bar_drag(app);
  });
}

#[tauri::command]
pub fn finish_recording_bar_drag(app: AppHandle) -> Result<(), String> {
  contain_recording_bar(&app).map_err(|error| error.to_string())?;
  source_selector::reposition(&app).map_err(|error| error.to_string())?;
  app
    .save_window_state(StateFlags::POSITION)
    .map_err(|error| error.to_string())?;
  Ok(())
}
