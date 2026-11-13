// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Export island's finish: the file an export wrote, revealed, copied
//! or dragged out to another app, and the island put away. Each acts on the
//! file Rust kept from the export (`EditorWorkspaceSlot::exported`), never a
//! path the webview names.

use super::*;

#[cfg(target_os = "macos")]
mod macos;

fn exported(app: &AppHandle, workspace: EditorKind) -> Result<PathBuf, String> {
  app
    .state::<EditorState>()
    .slot(workspace)
    .exported
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .clone()
    .filter(|path| path.exists())
    .ok_or_else(|| "The exported file is no longer there".to_owned())
}

fn is_occupied(app: &AppHandle, kind: EditorKind) -> bool {
  app
    .state::<EditorState>()
    .slot(kind)
    .artifact
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .is_some()
}

/// The finished Export island put away. The editor stays open on the
/// project it exported, or on a recording opened while it exported; only a
/// project moved to Recently Deleted with the export takes the editor with
/// it, its native player retired.
#[tauri::command]
pub fn put_away_export(app: AppHandle, window: tauri::WebviewWindow) -> Result<(), String> {
  let kind = kind_of_window(&window)?;
  app
    .state::<EditorState>()
    .slot(kind)
    .exported
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .take();
  if is_occupied(&app, kind) {
    return Ok(());
  }
  if kind == EditorKind::Recording {
    artifact::clear_recording_preview(&app);
  }
  window::close(&app, kind);
  emit_snapshot(&app, kind);
  Ok(())
}

/// Shows the exported file in Finder or File Explorer.
#[tauri::command]
pub fn reveal_export_file(app: AppHandle, workspace: EditorKind) -> Result<(), String> {
  crate::file_browser::reveal(&exported(&app, workspace)?)
}

/// Puts the exported file on the clipboard, as Copy in Finder does, so it
/// pastes into a message or a folder as the file itself.
#[tauri::command]
pub fn copy_export_file(app: AppHandle, workspace: EditorKind) -> Result<(), String> {
  let path = exported(&app, workspace)?;
  #[cfg(target_os = "macos")]
  return macos::copy(&path);
  #[cfg(not(target_os = "macos"))]
  return Err(format!("Copying {} is not available here", path.display()));
}

/// Starts dragging the exported file out of the calling window, from the
/// press the page is handling now, so it drops into another app as the file.
#[tauri::command]
pub fn drag_export_file(
  app: AppHandle,
  window: tauri::WebviewWindow,
  workspace: EditorKind,
) -> Result<(), String> {
  drag_out(&window, &exported(&app, workspace)?, &DragLook::Icon)
}

/// What a file dragged out looks like under the pointer, in points.
pub(crate) enum DragLook {
  /// The file's own icon, as the Finder draws it.
  Icon,
  /// The file itself, a picture.
  File { height: f64, width: f64 },
  /// A picture of what is dragged, as PNG data.
  Png {
    height: f64,
    png: Vec<u8>,
    width: f64,
  },
}

/// Starts dragging the file at `path` out of `window`, drawn as `look`, from
/// the press its page is handling now, so it drops wherever files are taken
/// as the file.
pub(crate) fn drag_out(
  window: &tauri::WebviewWindow,
  path: &Path,
  look: &DragLook,
) -> Result<(), String> {
  #[cfg(target_os = "macos")]
  return macos::drag(window, path, look);
  #[cfg(not(target_os = "macos"))]
  {
    let _ = (window, look);
    Err(format!(
      "Dragging {} out is not available here",
      path.display()
    ))
  }
}
