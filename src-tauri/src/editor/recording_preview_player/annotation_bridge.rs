// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Native arrow gestures and the recording document's clip commit bridge.

use super::*;
use crate::editor::annotations::edit::AnnotationEdit;
use crate::editor::annotations::gesture::AnnotationGestureTarget;
use crate::editor::annotations::handles::{annotation_handles, source_point};
use crate::editor::annotations::timing::{
  active_annotations, validate_clips, AnnotationTrack, RecordingAnnotationClip,
};
use crate::editor::annotations::{Annotation, AnnotationStyle};
use crate::editor::preview_platform::SelectionGesturePhase;
use tauri::Emitter;

#[derive(Default)]
pub(super) struct AnnotationState {
  pane: Option<u32>,
  mode: u32,
  selected: Option<String>,
  defaults: Option<AnnotationStyle>,
  gesture: Option<Gesture>,
}
struct Gesture {
  pane: u32,
  position: u64,
  edit: AnnotationEdit,
  working: Vec<Annotation>,
  before: Vec<RecordingAnnotationClip>,
  before_selected: Option<String>,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Commit {
  session_id: u64,
  pane_index: u32,
  source_position_ms: u64,
  annotations: Vec<Annotation>,
  selected_annotation_id: Option<String>,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Hover {
  session_id: u64,
  annotation_id: Option<String>,
}
fn track(pane: u32) -> AnnotationTrack {
  if pane == 1 {
    AnnotationTrack::Camera
  } else {
    AnnotationTrack::Primary
  }
}

#[tauri::command]
pub async fn set_recording_preview_annotations(
  state: tauri::State<'_, RecordingPreviewPlayerState>,
  session_id: u64,
  clips: Vec<RecordingAnnotationClip>,
  tool: Option<String>,
  pane_index: Option<u32>,
  selected_id: Option<String>,
  defaults: Option<AnnotationStyle>,
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
  manager.annotation.mode = match tool.as_deref() {
    Some("arrow") => 2,
    Some("select") => 1,
    _ => 0,
  };
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
      surface.redraw_recording_workspace();
    }
  }
  Ok(())
}

impl PreviewPlayerManager {
  fn annotation_marks(&self, pane: u32) -> Vec<Annotation> {
    if let Some(gesture) = &self.annotation.gesture {
      if gesture.pane == pane {
        return gesture.working.clone();
      }
    }
    let Some(sources) = self.sources.as_ref() else {
      return Vec::new();
    };
    sources
      .annotation_clips
      .read()
      .map(|clips| {
        active_annotations(
          &clips,
          track(pane),
          self.position_ms.min(sources.duration_ms.saturating_sub(1)),
        )
      })
      .unwrap_or_default()
  }
  fn annotation_gesture(
    &mut self,
    phase: SelectionGesturePhase,
    pane: u32,
    target: AnnotationGestureTarget,
    x: f64,
    y: f64,
  ) -> Option<Commit> {
    if self.is_playing || self.annotation.mode == 0 || pane > 1 {
      return None;
    }
    if matches!(phase, SelectionGesturePhase::Begin) {
      self.annotation.pane = Some(pane);
    }
    let session_id = self.session_id?;
    let sources = self.sources.as_ref()?;
    let position_ms = self.position_ms.min(sources.duration_ms.saturating_sub(1));
    let source = sources.playback_layout.panes.get(pane as usize)?;
    let point = source_point(x, y, (source.source_width, source.source_height));
    let clips = Arc::clone(&sources.annotation_clips);
    if matches!(phase, SelectionGesturePhase::Cancel) {
      let gesture = self.annotation.gesture.take()?;
      *clips.write().ok()? = gesture.before;
      self.annotation.selected = gesture.before_selected;
      self.publish_annotation_handles();
      let _ = self.restart(PlaybackMode::InteractiveStill);
      return None;
    }
    if matches!(phase, SelectionGesturePhase::Begin) {
      let mut working = self.annotation_marks(pane);
      match target {
        AnnotationGestureTarget::None | AnnotationGestureTarget::Select { .. } => {
          self.annotation.selected = match target {
            AnnotationGestureTarget::Select { index } => working.get(index).map(|a| a.id.clone()),
            _ => None,
          };
          self.publish_annotation_handles();
          if let Some(surface) = self
            .sources
            .as_ref()
            .and_then(|s| s.preview_surface.as_ref())
          {
            surface.redraw_recording_workspace();
          }
          return Some(Commit {
            session_id,
            pane_index: pane,
            source_position_ms: position_ms,
            annotations: working,
            selected_annotation_id: self.annotation.selected.clone(),
          });
        }
        _ => {}
      }
      let before = clips.read().ok()?.clone();
      let edit = AnnotationEdit::begin(
        &mut working,
        target,
        point,
        self.annotation.defaults.as_ref(),
      )?;
      let before_selected = self.annotation.selected.clone();
      self.annotation.selected = Some(edit.selected_id().to_owned());
      self.annotation.gesture = Some(Gesture {
        pane,
        position: position_ms,
        edit,
        working,
        before,
        before_selected,
      });
    } else {
      let gesture = self.annotation.gesture.as_mut()?;
      gesture.edit.update(&mut gesture.working, point);
    }
    let gesture = self.annotation.gesture.as_ref()?;
    let mut next = gesture.before.clone();
    for annotation in &gesture.working {
      if let Some(clip) = next.iter_mut().find(|c| c.annotation.id == annotation.id) {
        clip.annotation = annotation.clone();
      } else {
        next.push(RecordingAnnotationClip {
          annotation: annotation.clone(),
          track_id: track(gesture.pane),
          start_ms: gesture.position,
          end_ms: gesture.position.saturating_add(3000),
        });
      }
    }
    *clips.write().ok()? = next;
    let commit = matches!(phase, SelectionGesturePhase::End).then(|| Commit {
      session_id,
      pane_index: pane,
      source_position_ms: gesture.position,
      annotations: gesture.working.clone(),
      selected_annotation_id: self.annotation.selected.clone(),
    });
    if commit.is_some() {
      self.annotation.gesture = None;
    }
    self.publish_annotation_handles();
    let _ = self.restart(PlaybackMode::InteractiveStill);
    commit
  }
}

#[path = "annotation_callbacks.rs"]
mod callbacks;
pub(super) use callbacks::install;

#[cfg(test)]
#[path = "annotation_bridge_tests.rs"]
mod tests;

#[path = "annotation_handles.rs"]
mod handles;
