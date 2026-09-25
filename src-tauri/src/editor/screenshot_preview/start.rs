// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use tauri::{AppHandle, Manager};

use super::super::preview_platform::RecordingPreviewSurface;
use super::super::{EditorArtifact, EditorKind, EditorState};
use super::state::{PreviewSource, ScreenshotPreviewState};

#[tauri::command]
pub fn start_screenshot_preview(
  app: AppHandle,
  state: tauri::State<'_, ScreenshotPreviewState>,
  artifact_id: u64,
  session_id: u64,
) -> Result<(), String> {
  let sources = {
    let editor_state = app.state::<EditorState>();
    let artifact = editor_state
      .screenshot
      .artifact
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let Some(EditorArtifact::Screenshot { id, items, .. }) = artifact.as_ref() else {
      return Err("There is no screenshot to preview".to_owned());
    };
    if *id != artifact_id {
      return Err("That screenshot is no longer available in the editor".to_owned());
    }
    items
      .iter()
      .map(|item| PreviewSource {
        id: item.id,
        image: Arc::new(item.image.clone()),
        capture_width_points: item.capture_width_points(),
      })
      .collect::<Vec<_>>()
  };
  // Obtaining the surface is idempotent - on Windows it is the editor
  // window's one compositor, opened once and kept - so it happens outside the
  // manager lock, where it may round-trip to the event-loop thread.
  let window = app.get_webview_window(EditorKind::Screenshot.window_label().as_str());
  let mut surface = window
    .as_ref()
    .map(RecordingPreviewSurface::from_window)
    .transpose()?;
  let mut manager = state
    .0
    .lock()
    .map_err(|_| "The screenshot preview is unavailable".to_owned())?;
  if session_id < manager.latest_session_id {
    return Ok(());
  }
  manager.stop();
  manager.latest_session_id = session_id;
  manager.session_id = Some(session_id);
  manager.sources = sources;
  // The callbacks name this session, so they are installed only once it is
  // the one the manager holds, and under the same lock that adopts it. A
  // superseded start returns above without touching the surface every
  // session shares.
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  if let (Some(window), Some(surface)) = (window.as_ref(), surface.as_mut()) {
    super::start_callbacks::install(&app, window, surface, session_id);
  }
  manager.surface = surface.map(Arc::new);
  Ok(())
}
