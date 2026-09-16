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
  /// Whether the next arrow drawn animates, remembered from the last one the
  /// Animate switch was set on. Animation is the mark's own property rather
  /// than part of its dress, so it travels beside the style defaults.
  animated: Option<bool>,
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
  pane_index: Option<u32>,
  selected_id: Option<String>,
  defaults: Option<AnnotationStyle>,
  animated: Option<bool>,
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

impl PreviewPlayerManager {
  /// Takes the arrow tool in hand, or puts it down, in the native
  /// `ScreenwideAnnotationMode` the chrome is published with: nothing (0),
  /// hit-test the arrows already there (1), or also draw a new one on empty
  /// picture (2). A gesture in flight keeps the mode it began under.
  pub(in crate::editor::recording_preview_player) fn set_annotation_tool(
    &mut self,
    tool: Option<&str>,
  ) {
    if self.annotation.gesture.is_some() {
      return;
    }
    self.annotation.mode = match tool {
      Some("arrow") => 2,
      Some("select") => 1,
      _ => 0,
    };
  }

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
            #[cfg(target_os = "macos")]
            surface.redraw_recording_workspace();
            #[cfg(not(target_os = "macos"))]
            let _ = surface;
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
      // Whether a mark animates is not part of its dress, so `new_arrow` has
      // no say in it: the switch's last setting is applied to the fresh arrow
      // here, where the recording's own timed marks are made.
      if target == AnnotationGestureTarget::NewArrow {
        if let Some(animated) = self.annotation.animated {
          if let Some(mark) = working.iter_mut().find(|m| m.id == edit.selected_id()) {
            mark.animated = animated;
          }
        }
      }
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
        // The provisional clip the gesture draws through reaches back a
        // draw-in, the way the editor's own placement does, so the mark is
        // finished drawing at the playhead and visible under the hand.
        next.push(RecordingAnnotationClip {
          annotation: annotation.clone(),
          track_id: track(gesture.pane),
          start_ms: gesture
            .position
            .saturating_sub(crate::editor::annotations::reveal::REVEAL_DRAW_IN_MS as u64),
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
    // The Metal workspace re-encodes from its retained scene on a restart,
    // which is cheap enough to do per pointer sample. On Windows a restart
    // seeks the still decoder, which puts a decode between the hand and the
    // arrow on every sample; the pane already holds the frame, so it is
    // re-presented with the new marks instead, and only the end of the
    // gesture restarts the worker to bring it back in step.
    if commit.is_some() || !self.redraw_annotation_frame(pane, position_ms) {
      let _ = self.restart(PlaybackMode::InteractiveStill);
    }
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
