// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::{AppHandle, Manager};

use super::{emit_snapshot, EditorKind, EditorState};
use crate::app_windows::{self, on_demand};

/// Shows `kind`'s window, building it first when it is not open, and runs
/// `then` once it is out. A window that cannot be shown would hold its
/// workspace with no way to act on it, so the workspace is let go instead: a
/// recording stays in its project to be opened again.
pub fn show(app: &AppHandle, kind: EditorKind, then: impl FnOnce(&AppHandle) + Send + 'static) {
  let presenter = app.clone();
  on_demand::open(app, kind.window_label(), move |window| {
    let shown = window.and_then(|window| {
      app_windows::present(&presenter, &window).map_err(|error| error.to_string())
    });
    match shown {
      Ok(()) => then(&presenter),
      Err(error) => {
        eprintln!("Could not show the editor: {error}");
        let state = presenter.state::<EditorState>();
        *state
          .slot(kind)
          .artifact
          .lock()
          .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
        *state
          .capture_reservation
          .lock()
          .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
        emit_snapshot(&presenter, kind);
      }
    }
  });
}

/// Builds `kind`'s window without showing it, for a capture starting now.
pub fn prepare(app: &AppHandle, kind: EditorKind) {
  on_demand::prepare(app, kind.window_label());
}

/// Puts `kind`'s window away, to be shown again with what it holds.
pub fn hide(app: &AppHandle, kind: EditorKind) -> tauri::Result<()> {
  let_go_of_window_work(app, kind);
  if let Some(window) = app.get_webview_window(kind.window_label().as_str()) {
    app_windows::hide_without_focus_transfer(&window)?;
  }
  Ok(())
}

/// Destroys `kind`'s window, its page, picture and tool panel with it. The
/// next open builds them again.
pub fn close(app: &AppHandle, kind: EditorKind) {
  let_go_of_window_work(app, kind);
  // A page stops its preview as it closes, but not one destroyed, so it is
  // stopped here before the picture it draws into goes.
  match kind {
    EditorKind::Recording => super::recording_preview_player::stop_all(app),
    EditorKind::Screenshot => super::screenshot_preview::stop_all(app),
  }
  on_demand::close(app, kind.window_label());
}

fn let_go_of_window_work(app: &AppHandle, kind: EditorKind) {
  // Nothing measures the export's size for a window that is away.
  if kind == EditorKind::Recording {
    super::preview::cancel_export_work(&app.state::<super::EditorState>());
  }
  // A tool panel hangs off this window and stays up through an outside press,
  // so it has to be taken away with the editor it belongs to.
  app_windows::options::close_standalone_listbox_for_parent(app, kind.window_label().as_str());
}
