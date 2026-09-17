// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The recording document's clip commit bridge: the chrome's annotation
//! state as the native surface needs it.

use super::*;
use crate::editor::annotations::edit::AnnotationEdit;
use crate::editor::annotations::gesture::{annotation_mode, AnnotationGestureTarget, NewMarkKind};
use crate::editor::annotations::handles::{annotation_handles, source_point};
use crate::editor::annotations::timing::{
  active_annotations, validate_clips, AnnotationTrack, RecordingAnnotationClip,
};
use crate::editor::annotations::{Annotation, AnnotationStyle};
use crate::editor::preview_platform::SelectionGesturePhase;
use gesture::Gesture;
use tauri::Emitter;

#[derive(Default)]
pub(super) struct AnnotationState {
  pane: Option<u32>,
  mode: u32,
  selected: Option<String>,
  defaults: Option<AnnotationStyle>,
  /// Whether the next arrow animates and where the next counter's tail
  /// points. Both are the mark's own rather than part of its dress, so they
  /// travel beside the style defaults.
  animated: Option<bool>,
  counter_angle: Option<f64>,
  gesture: Option<Gesture>,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Hover {
  session_id: u64,
  annotation_id: Option<String>,
}

// The command's arguments are its wire format: the chrome sends the clips and
// the next mark's dress as one flat payload.
#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub async fn set_recording_preview_annotations(
  state: tauri::State<'_, RecordingPreviewPlayerState>,
  session_id: u64,
  clips: Vec<RecordingAnnotationClip>,
  pane_index: Option<u32>,
  selected_id: Option<String>,
  defaults: Option<AnnotationStyle>,
  animated: Option<bool>,
  counter_angle: Option<f64>,
) -> Result<(), String> {
  validate_clips(&clips)?;
  let mut manager = state
    .0
    .lock()
    .map_err(|_| "The recording preview is unavailable")?;
  manager.require_session(session_id)?;
  if manager.annotation.gesture.is_some() {
    return Ok(());
  }
  let sources = manager
    .sources
    .as_ref()
    .ok_or("The recording preview is not open")?;
  let mut current = sources
    .annotation_clips
    .write()
    .map_err(|_| "The annotations are unavailable")?;
  let changed = *current != clips;
  *current = clips;
  drop(current);
  manager.annotation.pane = pane_index.filter(|pane| *pane <= 1);
  manager.annotation.selected = selected_id;
  manager.annotation.defaults = defaults;
  manager.annotation.animated = animated;
  manager.annotation.counter_angle = counter_angle;
  manager.publish_annotation_handles();
  if !manager.is_playing {
    if changed {
      manager.restart(PlaybackMode::InteractiveStill)?;
    } else if let Some(surface) = manager
      .sources
      .as_ref()
      .and_then(|s| s.preview_surface.as_ref())
      .filter(|_| manager.annotation.mode != 0)
    {
      // Only the retained Metal workspace re-presents itself from marks
      // alone. The D3D11 backend has no retained scene to repaint, and
      // nothing to repaint yet either: its arrow overlay is not drawn.
      #[cfg(target_os = "macos")]
      surface.redraw_recording_workspace();
      #[cfg(not(target_os = "macos"))]
      let _ = surface;
    }
  }
  Ok(())
}

#[path = "annotation_gesture.rs"]
mod gesture;

#[path = "annotation_callbacks.rs"]
mod callbacks;
pub(super) use callbacks::install;

#[cfg(test)]
#[path = "annotation_bridge_tests.rs"]
mod tests;

#[path = "annotation_handles.rs"]
mod handles;
