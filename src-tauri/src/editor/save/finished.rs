// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Export island's finish: the file an export wrote, revealed, copied
//! or dragged out to another app, and the editor put away once the
//! island is. Each acts on the file Rust kept from the export
//! (`EditorWorkspaceSlot::exported`), never a path the webview names.

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

/// The finished Export island put away: the export's editor goes, its
/// native player retired, as the export used to do the moment it was done.
/// A recording opened while the island showed owns the window by then, and
/// stays in it.
#[tauri::command]
pub fn close_exported_editor(app: AppHandle, window: tauri::WebviewWindow) -> Result<(), String> {
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
  window::hide(&app, kind).map_err(|error| error.to_string())?;
  emit_snapshot(&app, kind);
  Ok(())
}

/// Shows the exported file in Finder or File Explorer.
#[tauri::command]
pub fn reveal_export_file(app: AppHandle, workspace: EditorKind) -> Result<(), String> {
  location::reveal(&exported(&app, workspace)?)
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
  let path = exported(&app, workspace)?;
  #[cfg(target_os = "macos")]
  return macos::drag(&window, &path);
  #[cfg(not(target_os = "macos"))]
  {
    let _ = window;
    Err(format!(
      "Dragging {} out is not available here",
      path.display()
    ))
  }
}
