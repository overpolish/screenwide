// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The export options window: one per editor workspace, presented centred on
//! the editor it belongs to and, on macOS, attached to it as a native child so
//! the pair moves and orders as a single unit.

use serde::Serialize;
use tauri::{
  AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, PhysicalSize, WebviewWindow,
};

use super::EditorKind;
use crate::windows::{self, WindowLabel};

mod presentation;

/// Told to an editor when its options window comes up, and again when it goes
/// away by any route: Escape, close, Cancel, Export, or a stood-down workspace.
const OPENED_EVENT: &str = "export-options://opened";
const CLOSED_EVENT: &str = "export-options://closed";

#[derive(Clone, Serialize)]
struct ExportOptionsVisibility {
  kind: EditorKind,
}

/// Tells a window what just happened to an options window. A failure to
/// deliver is not worth failing the show or hide over: a missing window has
/// nothing left to inform.
fn notify(app: &AppHandle, label: &str, kind: EditorKind, event: &str) {
  let _ = app.emit_to(label, event, ExportOptionsVisibility { kind });
}

/// An opening is told to the editor, which tracks it, and to the options
/// window itself, which is how it knows to measure and ask for its height
/// again. A reopening at an unchanged height has nothing for the frontend's
/// resize observer to report, and the window stays invisible until asked for.
fn notify_opened(app: &AppHandle, kind: EditorKind) {
  notify(app, kind.window_label().as_str(), kind, OPENED_EVENT);
  notify(app, options_label(kind).as_str(), kind, OPENED_EVENT);
}

/// Which options window belongs to a workspace. Kept beside the window code
/// rather than on `EditorKind`, because nothing else in the editor needs it.
const fn options_label(kind: EditorKind) -> WindowLabel {
  match kind {
    EditorKind::Recording => WindowLabel::ExportRecording,
    EditorKind::Screenshot => WindowLabel::ExportScreenshot,
  }
}

/// The workspace an editor or export window belongs to, by label. Used by the
/// editor's own move/resize watcher, which only has the label to go on.
pub(crate) fn kind_of_editor_label(label: &str) -> Option<EditorKind> {
  EditorKind::ALL
    .into_iter()
    .find(|kind| kind.window_label().as_str() == label)
}

/// The workspace an options window belongs to, by its own label: the mirror of
/// `kind_of_editor_label`, for the commands that window itself calls.
fn kind_of_options_label(label: &str) -> Option<EditorKind> {
  EditorKind::ALL
    .into_iter()
    .find(|kind| options_label(*kind).as_str() == label)
}

fn windows_of(app: &AppHandle, kind: EditorKind) -> tauri::Result<(WebviewWindow, WebviewWindow)> {
  let editor = app
    .get_webview_window(kind.window_label().as_str())
    .ok_or(tauri::Error::WindowNotFound)?;
  let options = app
    .get_webview_window(options_label(kind).as_str())
    .ok_or(tauri::Error::WindowNotFound)?;
  Ok((editor, options))
}

/// Places the options window in the middle of its editor's frame, at a size
/// the caller states rather than one read back.
///
/// Physical pixels throughout, so a window straddling two displays of
/// different scale factors cannot drift. A resize lands on the AppKit main
/// queue and is not observable the instant it is asked for, so the height
/// about to take effect is passed in: reading the window back would centre
/// the frame it is leaving.
fn center_on_editor_sized(
  app: &AppHandle,
  kind: EditorKind,
  size: PhysicalSize<u32>,
) -> tauri::Result<()> {
  let (editor, options) = windows_of(app, kind)?;
  let editor_position = editor.outer_position()?;
  let editor_size = editor.outer_size()?;
  options.set_position(PhysicalPosition::new(
    editor_position.x + (editor_size.width as i32 - size.width as i32) / 2,
    editor_position.y + (editor_size.height as i32 - size.height as i32) / 2,
  ))
}

/// Centres the options window at its current size.
fn center_on_editor(app: &AppHandle, kind: EditorKind) -> tauri::Result<()> {
  let (_, options) = windows_of(app, kind)?;
  let size = options.outer_size()?;
  center_on_editor_sized(app, kind, size)
}

/// Both options windows, for the boot passes that treat them as a pair.
pub(crate) const LABELS: [WindowLabel; 2] =
  [WindowLabel::ExportRecording, WindowLabel::ExportScreenshot];

/// Brings both options windows up hidden, and keeps closing one from
/// destroying it. Owned here rather than by the boot sequence, which would
/// otherwise have to know that these two windows exist at all.
pub fn initialize(app: &AppHandle) -> tauri::Result<()> {
  for label in LABELS {
    if let Some(window) = app.get_webview_window(label.as_str()) {
      windows::initialize_normal_window(&window)?;
    }
    windows::hide_instead_of_close(app, label);
  }
  Ok(())
}

pub fn show(app: &AppHandle, kind: EditorKind) -> tauri::Result<()> {
  let (editor, options) = windows_of(app, kind)?;
  center_on_editor(app, kind)?;
  presentation::attach(app, &editor, &options)?;
  // Presented invisible: the configured height is only a starting point, and
  // `resize_export_options` is what both fits the window and reveals it. Focus
  // is still taken, so the form's autofocus is in place by the time it is seen.
  presentation::conceal(app, &options)?;
  windows::show(&options, true)?;
  notify_opened(app, kind);
  Ok(())
}

pub fn hide(app: &AppHandle, kind: EditorKind) -> tauri::Result<()> {
  let Some(options) = app.get_webview_window(options_label(kind).as_str()) else {
    notify(app, kind.window_label().as_str(), kind, CLOSED_EVENT);
    return Ok(());
  };
  if let Some(editor) = app.get_webview_window(kind.window_label().as_str()) {
    presentation::detach(app, &editor, &options)?;
  }
  let hidden = windows::hide_without_focus_transfer(&options);
  // Every hide route ends here, so a close the editor did not initiate
  // reaches it just as readily as one it did.
  notify(app, kind.window_label().as_str(), kind, CLOSED_EVENT);
  hidden
}

/// Keeps a visible options window in the middle of its editor while that
/// editor is moved or resized.
///
/// Called from the editor's own move/resize handler, so it stays to cheap
/// geometry reads and one `set_position`, and does nothing at all in the usual
/// case where the options window is closed.
pub(crate) fn recenter_for_editor_label(app: &AppHandle, label: &str) {
  let Some(kind) = kind_of_editor_label(label) else {
    return;
  };
  let Some(options) = app.get_webview_window(options_label(kind).as_str()) else {
    return;
  };
  if !options.is_visible().unwrap_or(false) {
    return;
  }
  let _ = center_on_editor(app, kind);
}

/// Opens the calling editor's export options.
#[tauri::command]
pub fn show_export_options(app: AppHandle, window: WebviewWindow) -> Result<(), String> {
  let kind = super::kind_of_window(&window)?;
  show(&app, kind).map_err(|error| error.to_string())
}

/// Closes the export options window the call came from, and returns to its
/// editor. The workspace is read off the caller, which is the options window
/// itself rather than the editor.
#[tauri::command]
pub fn hide_export_options(app: AppHandle, window: WebviewWindow) -> Result<(), String> {
  let kind = kind_of_options_label(window.label())
    .ok_or_else(|| "That window has no editor workspace".to_owned())?;
  hide(&app, kind).map_err(|error| error.to_string())?;
  // The editor is the window the user came from, so it gets the focus back
  // rather than whichever window AppKit would have promoted.
  super::workspace::focus_pending(&app, kind);
  Ok(())
}

/// Fits the calling options window to the height its content asks for.
///
/// The width stays as configured: only the form's own height varies, with the
/// rows a workspace happens to show. Being non-resizable is a style mask on
/// macOS and no constraint on `setContentSize:`, so this still applies.
#[tauri::command]
pub fn resize_export_options(
  app: AppHandle,
  window: WebviewWindow,
  height: f64,
) -> Result<(), String> {
  let kind = kind_of_options_label(window.label())
    .ok_or_else(|| "That window has no editor workspace".to_owned())?;
  if !height.is_finite() || height <= 0.0 {
    return Err("That is not a height".to_owned());
  }
  let resize = || -> tauri::Result<()> {
    let scale = window.scale_factor()?;
    let current = window.inner_size()?;
    let width = f64::from(current.width) / scale;
    window.set_size(LogicalSize::new(width, height))?;
    // Centred against the height that is on its way, not the one still shown.
    center_on_editor_sized(
      &app,
      kind,
      PhysicalSize::new(current.width, (height * scale).round() as u32),
    )?;
    // The first fit after an opening is also the reveal, and every later one
    // asserts a visibility the window already has.
    presentation::reveal_after_resize(&window)
  };
  resize().map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn every_workspace_has_its_own_options_window() {
    assert_eq!(
      options_label(EditorKind::Recording).as_str(),
      "export-recording"
    );
    assert_eq!(
      options_label(EditorKind::Screenshot).as_str(),
      "export-screenshot"
    );
  }

  #[test]
  fn an_editor_label_names_its_workspace() {
    assert_eq!(
      kind_of_editor_label("editor-screenshot"),
      Some(EditorKind::Screenshot)
    );
    assert_eq!(kind_of_editor_label("export-screenshot"), None);
  }

  #[test]
  fn an_options_label_names_its_workspace() {
    let recording = kind_of_options_label("export-recording");
    assert_eq!(recording, Some(EditorKind::Recording));
    assert_eq!(kind_of_options_label("editor-recording"), None);
  }
}
