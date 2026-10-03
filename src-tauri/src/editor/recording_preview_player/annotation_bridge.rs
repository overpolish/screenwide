// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The recording document's clip commit bridge: the chrome's annotation
//! state as the native surface needs it.

use super::*;
use crate::editor::annotations::edit::AnnotationEdit;
use crate::editor::annotations::gesture::{annotation_mode, drawing_kind, AnnotationGestureTarget};
use crate::editor::annotations::handles::{
  annotation_handles, annotation_snap, source_point, HANDLE_FLAG_GROUPED,
};
use crate::editor::annotations::snap::{
  detect_anchors, request_anchors, source_per_point, source_per_size, threshold_source_px,
  AnchorBoxes, AnchorCache, SnapField, SnapModifiers, SnapRequest, SnapResult,
};
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
  /// Every annotation chosen, in the order it was; one on its own is the one
  /// in hand, whose grips the chrome draws.
  selected: Vec<String>,
  defaults: Option<AnnotationStyle>,
  /// Whether the next arrow animates and where the next counter's tail points.
  /// Both are the annotation's own rather than part of its dress, so they
  /// travel beside the style defaults.
  animated: Option<bool>,
  counter_angle: Option<f64>,
  gesture: Option<Gesture>,
  /// The chosen group being carried by a press on the picture.
  group: Option<group::GroupDrag>,
  /// A marquee band being drawn over the picture.
  band: Option<choice::Band>,
  /// The text box being typed into, which owns the pane's annotations the way
  /// a gesture does until the typing ends.
  text: Option<TextSession>,
  /// The latest frame's detected UI elements, for an arrow's tip to land on.
  /// Behind its own lock because the detection that fills it runs on a
  /// blocking thread and must never wait for the manager.
  anchors: Arc<AnchorCache>,
  /// The frame under the pointer, for a highlight to select from. Decoded on
  /// a blocking thread for the same reason.
  pictures: Arc<crate::editor::annotations::highlight::picture::PictureCache>,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Hover {
  session_id: u64,
  annotation_id: Option<String>,
}

// The command's arguments are its wire format: the chrome sends the clips and
// the next annotation's dress as one flat payload.
#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub async fn set_recording_preview_annotations(
  state: tauri::State<'_, RecordingPreviewPlayerState>,
  session_id: u64,
  clips: Vec<RecordingAnnotationClip>,
  pane_index: Option<u32>,
  selected_ids: Vec<String>,
  defaults: Option<AnnotationStyle>,
  animated: Option<bool>,
  counter_angle: Option<f64>,
) -> Result<(), String> {
  let mut clips = clips;
  validate_clips(&clips)?;
  let mut manager = state
    .0
    .lock()
    .map_err(|_| "The recording preview is unavailable")?;
  manager.require_session(session_id)?;
  if manager.annotation.gesture.is_some() || manager.annotation.group.is_some() {
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
  // A text box being typed into takes React's clips - the panel may be
  // dressing it - with the text typed so far laid over them.
  manager.merge_text_session(&mut clips, &current);
  sources.attach_pins(&mut clips, manager.position_ms);
  let changed = *current != clips;
  // A list that grew or shrank has moved what the halo's index names, so the
  // halo goes rather than pass to whatever took its place.
  let recounted = current.len() != clips.len();
  *current = clips;
  drop(current);
  manager.annotation.pane = pane_index.filter(|pane| *pane <= 1);
  manager.annotation.selected = selected_ids;
  manager.annotation.defaults = defaults;
  manager.annotation.animated = animated;
  manager.annotation.counter_angle = counter_angle;
  manager.publish_annotation_handles();
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  if recounted {
    if let Some(surface) = manager
      .sources
      .as_ref()
      .and_then(|sources| sources.preview_surface.as_ref())
    {
      surface.redraw_annotation_hover(None);
    }
  }
  if !manager.is_playing {
    if changed {
      manager.restart(PlaybackMode::InteractiveStill)?;
    } else if let Some(surface) = manager
      .sources
      .as_ref()
      .and_then(|s| s.preview_surface.as_ref())
      .filter(|_| manager.annotation.mode != 0)
    {
      // Only the retained macOS workspace re-presents itself from annotations
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

/// Every pinned annotation's latest status, for a timeline that starts
/// listening after the paths first landed.
#[tauri::command]
pub async fn recording_preview_pin_statuses(
  state: tauri::State<'_, RecordingPreviewPlayerState>,
  session_id: u64,
) -> Result<Vec<super::pin_paths::PinStatus>, String> {
  let manager = state
    .0
    .lock()
    .map_err(|_| "The recording preview is unavailable")?;
  manager.require_session(session_id)?;
  Ok(
    manager
      .sources
      .as_ref()
      .and_then(|sources| sources.pins.as_ref())
      .map(|pins| pins.statuses())
      .unwrap_or_default(),
  )
}

/// Where a pinned annotation hidden at `source_ms` goes when the hand says
/// its content is back in view there, as a movement from where it was drawn:
/// found again on that frame, or where it was last shown. Nothing when it is
/// not hidden there. The frames are read off the preview's lock.
#[tauri::command]
pub async fn recording_preview_pin_back_in_view(
  state: tauri::State<'_, RecordingPreviewPlayerState>,
  session_id: u64,
  annotation_id: String,
  source_ms: u64,
) -> Result<Option<[f64; 2]>, String> {
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  {
    let (pins, clip, image_width) = {
      let manager = state
        .0
        .lock()
        .map_err(|_| "The recording preview is unavailable")?;
      manager.require_session(session_id)?;
      let Some(sources) = manager.sources.as_ref() else {
        return Ok(None);
      };
      let Some(pins) = sources.pins.clone() else {
        return Ok(None);
      };
      let clip = sources
        .annotation_clips
        .read()
        .map_err(|_| "The annotations are unavailable")?
        .iter()
        .find(|clip| clip.annotation.id == annotation_id)
        .cloned();
      let Some(clip) = clip else {
        return Ok(None);
      };
      (pins, clip, sources.screen_size_width())
    };
    tauri::async_runtime::spawn_blocking(move || pins.back_in_view(&clip, source_ms, image_width))
      .await
      .map_err(|error| error.to_string())
  }
  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  {
    let _ = (state, session_id, annotation_id, source_ms);
    Ok(None)
  }
}

mod gesture;

mod commit;

mod group;

mod choice;

mod text;
use text::TextSession;

mod callbacks;
pub(super) use callbacks::install;

mod hold;

#[cfg(test)]
mod tests;

mod handles;

mod redraw;
