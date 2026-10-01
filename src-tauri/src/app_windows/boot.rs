// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Which predefined windows exist and how they come up.
//!
//! Held here rather than in the application's setup closure, so that adding a
//! window is a change to the window layer alone.

use tauri::{AppHandle, Manager};

use super::{hide_instead_of_close, initialize_editor, initialize_normal_window, WindowLabel};

/// Ordinary app windows: created hidden, and closing one hides it rather than
/// destroying a window that has to be reopenable by label.
const NORMAL: &[WindowLabel] = &[WindowLabel::Settings];

/// Windows that come up hidden but are dismissed by their own feature code.
const HIDDEN_ON_LAUNCH: &[WindowLabel] = &[WindowLabel::Update];

/// Overlays and transients, which are only ever hidden, never closed.
const DISMISSED_ON_CLOSE: &[WindowLabel] = &[
  WindowLabel::RecordingBar,
  WindowLabel::RecordingSourceSelector,
  WindowLabel::RegionSelector,
  WindowLabel::StandaloneListbox,
  WindowLabel::ToolPanelRecording,
  WindowLabel::ToolPanelScreenshot,
  WindowLabel::RecordingDock,
];

/// The windows whose page carries the title bar. On macOS they are configured
/// with a native overlay title bar, so the real traffic lights sit over the
/// page; on Windows the page draws its own caption buttons and wants no native
/// frame. `decorations` is a single configuration key for both platforms, so
/// the Windows half is undone here.
#[cfg(target_os = "windows")]
const TITLE_BAR_IN_PAGE: &[WindowLabel] = &[
  WindowLabel::EditorRecording,
  WindowLabel::EditorScreenshot,
  WindowLabel::ExportRecording,
  WindowLabel::ExportScreenshot,
  WindowLabel::Settings,
  WindowLabel::Update,
];

/// Takes the native frame back off the title-bar windows. Every one of them is
/// created hidden, so nothing is on screen while the frame changes.
#[cfg(target_os = "windows")]
fn remove_native_frames(app: &AppHandle) -> tauri::Result<()> {
  for &label in TITLE_BAR_IN_PAGE {
    let Some(window) = app.get_webview_window(label.as_str()) else {
      continue;
    };
    // Dropping the frame keeps the outer size, which would hand the page the
    // caption's height as extra content. Put the configured size back.
    let inner = window.inner_size()?;
    window.set_decorations(false)?;
    window.set_size(inner)?;
  }

  Ok(())
}

pub fn initialize_predefined_windows(app: &AppHandle) -> tauri::Result<()> {
  #[cfg(target_os = "windows")]
  remove_native_frames(app)?;
  for &label in NORMAL.iter().chain(HIDDEN_ON_LAUNCH) {
    if let Some(window) = app.get_webview_window(label.as_str()) {
      initialize_normal_window(&window)?;
    }
  }
  crate::editor::export_window::initialize(app)?;
  for kind in crate::editor::EditorKind::ALL {
    let label = kind.window_label();
    if let Some(window) = app.get_webview_window(label.as_str()) {
      initialize_editor(&window)?;
    }
    // Closing an editor window cancels only its own pending capture.
    hide_instead_of_close(app, label);
  }
  for &label in DISMISSED_ON_CLOSE
    .iter()
    .chain(NORMAL)
    .chain(HIDDEN_ON_LAUNCH)
  {
    hide_instead_of_close(app, label);
  }

  // The tool panels are floating flyouts like the recording bar, but have no
  // initialiser of their own: they are only ever shown. Their corners are
  // rounded here, once, with the rest of the boot.
  #[cfg(target_os = "windows")]
  for label in [
    WindowLabel::ToolPanelRecording,
    WindowLabel::ToolPanelScreenshot,
  ] {
    if let Some(window) = app.get_webview_window(label.as_str()) {
      super::platform::round_corners(&window)?;
    }
  }

  // Predefined pages start hidden, including panels converted lazily later.
  #[cfg(target_os = "macos")]
  for window in app.webview_windows().into_values() {
    if !window.is_visible()? {
      super::webview_visibility::hide_webview(&window)?;
    }
  }

  Ok(())
}
