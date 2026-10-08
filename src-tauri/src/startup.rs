// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What happens once, when the application comes up.
//!
//! The builder in `lib.rs` says what this app is made of: its plugins, its
//! shared state and its commands. This says what it does before the user sees
//! it, in the order the steps depend on each other.

#[cfg(target_os = "macos")]
use tauri::{AppHandle, Manager};

use crate::{app_windows, editor};

/// Brings the application up. The order matters: settings are loaded before
/// anything reads them, and native input monitoring starts after the controls
/// it consults exist.
pub(crate) fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
  crate::shortcuts::diagnostics::initialize(app.handle());
  #[cfg(target_os = "windows")]
  crate::tooltip_window::initialize(app.handle())?;
  #[cfg(target_os = "windows")]
  crate::alert::initialize(app.handle())?;
  // Before any overlay can be shown: the watch has to be in place by the time
  // another application first takes the foreground over one.
  #[cfg(target_os = "windows")]
  app_windows::platform::initialize_band_guard()?;
  #[cfg(debug_assertions)]
  if let Some(preview_url) = std::env::var_os("SCREENWIDE_STORYBOOK_NATIVE_URL") {
    crate::storybook_native::show(app.handle(), &preview_url.to_string_lossy())?;
    return Ok(());
  }
  // The windows in `tauri.conf.json` carry English titles; the app's own
  // language replaces them before any is shown.
  for label in app_windows::WindowLabel::ALL {
    if let Some(window) = tauri::Manager::get_webview_window(app.handle(), label.as_str()) {
      let _ = window.set_title(&label.title());
    }
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
  // Windows passes a double-clicked project to the launch it starts, which
  // then opens that instead of the recording controls.
  let cwd = std::env::current_dir().unwrap_or_default();
  let launch_project = editor::project_argument(std::env::args_os().skip(1), &cwd);
  let show_recording_bar_on_launch = launch_project.is_none()
    && (recording_bar_preview_enabled
      || crate::settings::current(app.handle()).show_recording_bar_on_launch);

  // Create panels lazily so conversion cannot order a stale frame onscreen.
  #[cfg(not(target_os = "macos"))]
  {
    app_windows::initialize_recording_bar(app.handle())?;
    app_windows::initialize_recording_source_selector(app.handle())?;
    app_windows::initialize_region_selector(app.handle())?;
    app_windows::initialize_standalone_listbox(app.handle())?;
    app_windows::initialize_recording_dock(app.handle())?;
  }
  app_windows::initialize_predefined_windows(app.handle())?;
  app_windows::initialize_recording_bar_position(app.handle())?;
  app_windows::initialize_topology_management(app.handle());
  app_windows::manage_recording_bar_movement(app.handle());
  app_windows::manage_recording_dock_movement(app.handle());
  editor::initialize(app.handle());
  crate::shortcuts::initialize(app.handle());
  crate::system_accent::initialize(app.handle());
  crate::recording::sleep::initialize(app.handle());
  app_windows::manage_transient_popover_dismissal(app.handle());

  #[cfg(target_os = "macos")]
  crate::permissions::show_on_launch(app.handle(), show_recording_bar_on_launch)?;

  #[cfg(not(target_os = "macos"))]
  if show_recording_bar_on_launch {
    app_windows::show_recording_ui(app.handle())?;
  }
  crate::permissions::start_watcher(app.handle().clone());
  if let Some(file) = launch_project {
    editor::open_project_detached(app.handle(), file);
  }

  #[cfg(target_os = "macos")]
  hide_unrequested_windows(app.handle())?;

  Ok(())
}

/// Windows that native effects may have ordered on screen during setup. Their
/// first presentation always belongs to an explicit user action.
#[cfg(target_os = "macos")]
fn hide_unrequested_windows(app: &AppHandle) -> tauri::Result<()> {
  let app_handle = app.clone();
  let mut labels = vec![
    app_windows::WindowLabel::Projects,
    app_windows::WindowLabel::Settings,
  ];
  labels.extend(editor::export_window::LABELS);
  labels.extend(editor::EditorKind::ALL.map(editor::EditorKind::window_label));
  app.run_on_main_thread(move || {
    for label in &labels {
      if let Some(window) = app_handle.get_webview_window(label.as_str()) {
        let _ = app_windows::hide(&window);
      }
    }
  })?;
  Ok(())
}

/// Files Finder asked the app to open, at launch or while it runs. One editor
/// holds one project, so of several the last wins.
#[cfg(target_os = "macos")]
pub(crate) fn open_files(app: &AppHandle, urls: Vec<tauri::Url>) {
  let file = urls
    .into_iter()
    .filter_map(|url| url.to_file_path().ok())
    .rfind(|path| editor::is_project_file(path));
  if let Some(file) = file {
    editor::open_project_detached(app, file);
  }
}
