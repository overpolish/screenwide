// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::{AppHandle, Manager, PhysicalPosition, WebviewWindow, WindowEvent};
use tauri_plugin_window_state::{StateFlags, WindowExt};

use super::{geometry::keep_window_on_a_monitor, platform, WindowLabel};

/// Asks before an editor throws away work that exists nowhere else.
///
/// An empty workspace has nothing to lose and closes straight away. When a
/// capture is waiting, the confirmation sheet takes the question and the
/// discard happens only on the user's say-so. Nothing here waits on the
/// answer: a close request arrives on the main thread, and blocking there
/// would freeze every window.
fn confirm_editor_close(app: &AppHandle, window: &WebviewWindow, kind: crate::editor::EditorKind) {
  if !crate::editor::has_pending_workspace_kind(app, kind) {
    crate::editor::discard(app, kind);
    return;
  }

  let (title, message) = match kind {
    crate::editor::EditorKind::Recording => (
      "Delete this recording?",
      "Closing the editor deletes the unsaved recording. This cannot be undone.",
    ),
    crate::editor::EditorKind::Screenshot => (
      "Delete this screenshot?",
      "Closing the editor deletes the unsaved screenshot. This cannot be undone.",
    ),
  };
  // A second close request while the sheet is up is the same question, and
  // `ask` drops it rather than stacking another sheet on top.
  crate::confirm_sheet::ask(
    app,
    window,
    crate::confirm_sheet::ConfirmCopy {
      cancel_label: "Cancel".to_owned(),
      confirm_label: "Delete".to_owned(),
      message: message.to_owned(),
      title: title.to_owned(),
    },
    move |app, confirmed| {
      if confirmed {
        crate::editor::discard(app, kind);
      }
    },
  );
}

pub fn hide_instead_of_close(app: &AppHandle, label: WindowLabel) {
  if let Some(window) = app.get_webview_window(label.as_str()) {
    let app = app.clone();
    let window_to_hide = window.clone();
    window.on_window_event(move |event| {
      if let WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close();
        match label {
          // Closing an editor window cancels only its own pending capture.
          WindowLabel::EditorRecording => {
            confirm_editor_close(&app, &window_to_hide, crate::editor::EditorKind::Recording);
          }
          WindowLabel::EditorScreenshot => {
            confirm_editor_close(&app, &window_to_hide, crate::editor::EditorKind::Screenshot);
          }
          // Closing an export options window returns to its editor.
          WindowLabel::ExportRecording => {
            let _ =
              crate::editor::hide_export_options_for(&app, crate::editor::EditorKind::Recording);
          }
          WindowLabel::ExportScreenshot => {
            let _ =
              crate::editor::hide_export_options_for(&app, crate::editor::EditorKind::Screenshot);
          }
          WindowLabel::Settings => {
            let _ = crate::settings::hide_settings(app.clone());
          }
          #[cfg(target_os = "macos")]
          WindowLabel::Permissions => {
            let _ = crate::permissions::dismiss_permissions_window(app.clone());
          }
          _ => {
            let _ = super::hide_without_focus_transfer(&window_to_hide);
          }
        }
      }
    });
  }
}

#[cfg(target_os = "macos")]
pub fn get_or_create<F>(
  app: &AppHandle,
  label: WindowLabel,
  create: F,
) -> tauri::Result<WebviewWindow>
where
  F: FnOnce() -> tauri::Result<WebviewWindow>,
{
  app
    .get_webview_window(label.as_str())
    .map_or_else(create, Ok)
}

pub fn show(window: &WebviewWindow, focus: bool) -> tauri::Result<()> {
  #[cfg(target_os = "macos")]
  super::dismissal::cancel_pending_dismissal(window)?;
  platform::prepare_to_show(window)?;
  window.show()?;
  window.unminimize()?;
  if focus {
    window.set_focus()?;
  }

  Ok(())
}

pub fn initialize_recording_bar_position(app: &AppHandle) -> tauri::Result<()> {
  let Some(window) = app.get_webview_window(WindowLabel::RecordingBar.as_str()) else {
    return Ok(());
  };
  let Some(monitor) = window.current_monitor()? else {
    return Ok(());
  };

  let monitor_position = monitor.position();
  let monitor_size = monitor.size();
  let window_size = window.outer_size()?;

  window.set_position(PhysicalPosition {
    x: monitor_position.x + (monitor_size.width.saturating_sub(window_size.width) / 2) as i32,
    y: monitor_position.y + monitor_size.height.saturating_sub(window_size.height + 100) as i32,
  })?;

  // Restoring after the fallback means the first launch has a sensible
  // position while later launches respect where the user moved the bar.
  let _ = window.restore_state(StateFlags::POSITION);
  keep_window_on_a_monitor(app, &window)?;

  Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn initialize_recording_bar(app: &AppHandle) -> tauri::Result<()> {
  if let Some(window) = app.get_webview_window(WindowLabel::RecordingBar.as_str()) {
    platform::initialize_recording_bar(&window)?;
  }

  Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn initialize_recording_source_selector(app: &AppHandle) -> tauri::Result<()> {
  if let Some(window) = app.get_webview_window(WindowLabel::RecordingSourceSelector.as_str()) {
    platform::initialize_recording_source_selector(&window)?;
  }

  Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn initialize_region_selector(app: &AppHandle) -> tauri::Result<()> {
  if let Some(window) = app.get_webview_window(WindowLabel::RegionSelector.as_str()) {
    platform::initialize_region_selector(&window)?;
    window.set_ignore_cursor_events(true)?;
  }

  Ok(())
}

/// The shared listbox and both editors' tool panels come up the same way:
/// each is a panel opened from another window, so each gets the overlay
/// treatment the listbox has always had.
#[cfg(not(target_os = "macos"))]
pub fn initialize_standalone_listbox(app: &AppHandle) -> tauri::Result<()> {
  for label in [
    WindowLabel::StandaloneListbox,
    WindowLabel::ToolPanelRecording,
    WindowLabel::ToolPanelScreenshot,
  ] {
    if let Some(window) = app.get_webview_window(label.as_str()) {
      platform::initialize_standalone_listbox(&window)?;
    }
  }

  Ok(())
}

pub fn initialize_editor(window: &WebviewWindow) -> tauri::Result<()> {
  platform::initialize_editor(window)?;
  // A bundled macOS application can order its ordinary main window onscreen
  // during application activation even when it was configured as invisible.
  // Export only becomes visible when an artifact is presented.
  super::hide(window)?;

  crate::editor::preview_platform::prewarm(window.clone());

  let app = window.app_handle().clone();
  let export = window.clone();
  window.on_window_event(move |event| {
    if matches!(event, WindowEvent::Moved(_) | WindowEvent::Resized(_)) {
      // An open export options window is a child of this one, so it follows
      // the editor's frame rather than keeping the place it was opened at.
      crate::editor::export_window::recenter_for_editor_label(&app, export.label());
    }
    // A panel attached to this editor follows it around the screen by itself.
    // It is placed against the preview area rather than the window frame, so a
    // resize leaves it in the wrong corner - the editor answers that by
    // re-placing it, which is why a resize no longer puts it away. Minimising
    // is the resize Tauri reports, and that one still takes the panel with it.
    if matches!(event, WindowEvent::Resized(_)) && export.is_minimized().unwrap_or(false) {
      super::options::close_standalone_listbox_for_parent(&app, export.label());
    }
  });

  Ok(())
}

pub fn initialize_normal_window(window: &WebviewWindow) -> tauri::Result<()> {
  platform::initialize_editor(window)?;
  super::hide(window)
}

/// Recover a window only when it no longer overlaps any connected display.
/// Partially off-screen positions remain under the user's control.
pub fn recover_window_position(app: &AppHandle, window: &WebviewWindow) -> tauri::Result<()> {
  keep_window_on_a_monitor(app, window)
}
