// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What happens once, when the application comes up.
//!
//! The builder in `lib.rs` says what this app is made of: its plugins, its
//! shared state and its commands. This says what it does before the user sees
//! it, in the order the steps depend on each other.

#[cfg(target_os = "macos")]
use tauri::{AppHandle, Manager};

use crate::{editor, windows};

/// Brings the application up. The order matters: settings are loaded before
/// anything reads them, and native input monitoring starts after the controls
/// it consults exist.
pub(crate) fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
  crate::shortcuts::diagnostics::initialize(app.handle());
  #[cfg(target_os = "windows")]
  crate::tooltip_window::initialize(app.handle())?;
  #[cfg(debug_assertions)]
  if let Some(preview_url) = std::env::var_os("SCREENWIDE_STORYBOOK_NATIVE_URL") {
    crate::storybook_native::show(app.handle(), &preview_url.to_string_lossy())?;
    return Ok(());
  }

  #[cfg(target_os = "macos")]
  {
    editor::initialize_cursor_artwork();
  }
  #[cfg(desktop)]
  crate::tray::initialize(app)?;
  crate::settings::initialize(app.handle());
  // Load controls before native input monitoring starts.
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  crate::glide::settings::initialize(app.handle());
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  crate::glide::initialize(app.handle()).map_err(std::io::Error::other)?;
  let recording_bar_preview_enabled =
    cfg!(debug_assertions) && std::env::var_os("SCREENWIDE_SHOW_RECORDING_BAR").is_some();
  let show_recording_bar_on_launch = recording_bar_preview_enabled
    || crate::settings::current(app.handle()).show_recording_bar_on_launch;

  // Create panels lazily so conversion cannot order a stale frame onscreen.
  #[cfg(not(target_os = "macos"))]
  {
    windows::initialize_recording_bar(app.handle())?;
    windows::initialize_recording_source_selector(app.handle())?;
    windows::initialize_region_selector(app.handle())?;
    windows::initialize_standalone_listbox(app.handle())?;
    windows::initialize_recording_dock(app.handle())?;
  }
  windows::initialize_predefined_windows(app.handle())?;
  windows::initialize_recording_bar_position(app.handle())?;
  windows::initialize_topology_management(app.handle());
  windows::manage_recording_bar_movement(app.handle());
  windows::manage_recording_dock_movement(app.handle());
  editor::initialize(app.handle());
  let has_pending_export = editor::has_pending_workspace(app.handle());
  crate::shortcuts::initialize(app.handle());
  crate::system_accent::initialize(app.handle());
  windows::manage_transient_popover_dismissal(app.handle());

  #[cfg(target_os = "macos")]
  crate::permissions::show_on_launch(
    app.handle(),
    show_recording_bar_on_launch,
    has_pending_export,
  )?;

  #[cfg(not(target_os = "macos"))]
  if show_recording_bar_on_launch && !has_pending_export {
    windows::show_recording_ui(app.handle())?;
  }
  crate::permissions::start_watcher(app.handle().clone());

  #[cfg(target_os = "macos")]
  hide_unrequested_windows(app.handle())?;

  Ok(())
}

/// Windows that native effects may have ordered on screen during setup. Their
/// first presentation always belongs to an explicit user action.
#[cfg(target_os = "macos")]
fn hide_unrequested_windows(app: &AppHandle) -> tauri::Result<()> {
  let app_handle = app.clone();
  // Recovery may have put an artifact in one workspace; that window is
  // the one presentation the user did ask for, so it alone is spared.
  let mut labels = vec![windows::WindowLabel::Settings];
  labels.extend(editor::export_window::LABELS);
  labels.extend(
    editor::EditorKind::ALL
      .into_iter()
      .filter(|kind| !editor::has_pending_workspace_kind(app, *kind))
      .map(editor::EditorKind::window_label),
  );
  app.run_on_main_thread(move || {
    for label in &labels {
      if let Some(window) = app_handle.get_webview_window(label.as_str()) {
        let _ = windows::hide(&window);
      }
    }
  })?;
  Ok(())
}
