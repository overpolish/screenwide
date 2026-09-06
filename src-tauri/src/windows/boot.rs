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
  WindowLabel::RecordingOptions,
  WindowLabel::StandaloneListbox,
  WindowLabel::RecordingDock,
];

pub fn initialize_predefined_windows(app: &AppHandle) -> tauri::Result<()> {
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
  for &label in DISMISSED_ON_CLOSE.iter().chain(NORMAL) {
    hide_instead_of_close(app, label);
  }

  Ok(())
}
