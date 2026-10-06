// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The plugins the app is built with, kept apart from the command list so
//! that neither has to be read past to reach the other.

use tauri::{Builder, Wry};

/// Every plugin the app uses, in one place. The window-state plugin is
/// deliberately narrow: only the recording bar's position is remembered, and
/// not on the run that first places it.
pub(crate) fn with_plugins(builder: Builder<Wry>) -> Builder<Wry> {
  // First, so a second launch hands over before any other plugin starts: a
  // double-clicked project reaches the running app instead of a new one.
  #[cfg(target_os = "windows")]
  let builder = builder.plugin(tauri_plugin_single_instance::init(|app, args, cwd| {
    match crate::editor::project_argument(args, std::path::Path::new(&cwd)) {
      Some(file) => crate::editor::open_project_detached(app, file),
      None => {
        let _ = crate::app_windows::show_recording_ui(app);
      }
    }
  }));
  let builder = builder
    .plugin(tauri_plugin_autostart::init(
      tauri_plugin_autostart::MacosLauncher::LaunchAgent,
      None,
    ))
    .plugin(tauri_plugin_clipboard_manager::init())
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_global_shortcut::Builder::new().build())
    .plugin(tauri_plugin_opener::init())
    .plugin(tauri_plugin_process::init())
    .plugin(tauri_plugin_updater::Builder::new().build())
    .plugin(
      tauri_plugin_window_state::Builder::default()
        .with_state_flags(tauri_plugin_window_state::StateFlags::POSITION)
        .with_filter(|label| label == crate::app_windows::WindowLabel::RecordingBar.as_str())
        .skip_initial_state(crate::app_windows::WindowLabel::RecordingBar.as_str())
        .build(),
    );
  #[cfg(target_os = "macos")]
  let builder = builder
    .plugin(tauri_plugin_macos_permissions::init())
    .plugin(tauri_nspanel::init());
  builder
}
