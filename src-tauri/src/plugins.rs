// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The plugins the app is built with, kept apart from the command list so
//! that neither has to be read past to reach the other.

use tauri::{Builder, Wry};

/// Every plugin the app uses, in one place. The window-state plugin is
/// deliberately narrow: only the recording bar's position is remembered, and
/// not on the run that first places it.
pub(crate) fn with_plugins(builder: Builder<Wry>) -> Builder<Wry> {
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
        .with_filter(|label| label == crate::windows::WindowLabel::RecordingBar.as_str())
        .skip_initial_state(crate::windows::WindowLabel::RecordingBar.as_str())
        .build(),
    );
  #[cfg(target_os = "macos")]
  let builder = builder
    .plugin(tauri_plugin_macos_permissions::init())
    .plugin(tauri_nspanel::init());
  builder
}
