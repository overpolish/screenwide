// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use tauri::{AppHandle, Manager};

use super::super::{EditorArtifact, EditorState};
use super::state::{PreviewManager, PreviewSource, ScreenshotPreviewState};

/// Refreshes captured sources without replacing the live Metal surface.
/// Screenshot workspaces append items while the editor window stays open;
/// restarting the surface at that point can overlap an in-flight layout and
/// drawable presentation from the previous session.
#[tauri::command]
pub async fn refresh_screenshot_preview_sources(
  app: AppHandle,
  state: tauri::State<'_, ScreenshotPreviewState>,
  artifact_id: u64,
  session_id: u64,
) -> Result<(), String> {
  let existing = {
    let manager = state
      .0
      .lock()
      .map_err(|_| "The screenshot preview is unavailable".to_owned())?;
    manager.require_session(session_id)?;
    manager.sources.clone()
  };
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
        image: existing
          .iter()
          .find(|source| source.id == item.id)
          .map_or_else(
            || Arc::new(item.image.clone()),
            |source| Arc::clone(&source.image),
          ),
        capture_width_points: item.capture_width_points(),
      })
      .collect::<Vec<_>>()
  };
  let presentation = {
    let mut manager = state
      .0
      .lock()
      .map_err(|_| "The screenshot preview is unavailable".to_owned())?;
    manager.require_session(session_id)?;
    manager.sources = sources;
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    let hover = manager
      .annotation_hover
      .map(|hover| (hover.layer_id, hover.index, hover.width));
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let hover = None;
    manager.has_layout.then(|| {
      (
        manager.surface.clone(),
        manager.output.clone(),
        manager.sources.clone(),
        hover,
      )
    })
  };
  if let Some((Some(surface), Some(output), sources, hover)) = presentation {
    let batch = surface.present_batch();
    let staged = PreviewManager::present_snapshot(&surface, &output, &sources, hover)?;
    drop(batch);
    if !staged {
      PreviewManager::present_once_pane_exists(&app, session_id, 0);
    }
  }
  Ok(())
}
