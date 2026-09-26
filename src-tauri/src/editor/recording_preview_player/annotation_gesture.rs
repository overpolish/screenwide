// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The native annotation gesture: the tool in hand, the annotations a pane is
//! showing, and the provisional clips a drag writes through until it commits.

use super::*;
use crate::editor::annotations::pin::fold;
use crate::editor::annotations::{AnnotationKind, AnnotationShape};

pub(super) struct Gesture {
  pane: u32,
  position: u64,
  edit: AnnotationEdit,
  working: Vec<Annotation>,
  before: Vec<RecordingAnnotationClip>,
  before_selected: Option<String>,
  /// What this gesture can snap to, built from the pane as it was when the
  /// press landed. Its element anchors are refreshed each sample, because
  /// detection may land part way through the drag.
  field: SnapField,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Commit {
  pub(super) session_id: u64,
  pub(super) pane_index: u32,
  pub(super) source_position_ms: u64,
  pub(super) annotations: Vec<Annotation>,
  pub(super) selected_annotation_id: Option<String>,
  /// Where in a text box's typing this commit falls; React groups a typing's
  /// commits into one edit.
  pub(super) text_edit: Option<crate::editor::annotations::text::edit::TextEditPhase>,
  /// The pins of the pinned annotations among `annotations`, which an edit
  /// on the picture may have given a keyframe.
  pub(super) pins: Vec<super::commit::CommitPin>,
}

pub(super) fn track(pane: u32) -> AnnotationTrack {
  if pane == 1 {
    AnnotationTrack::Camera
  } else {
    AnnotationTrack::Primary
  }
}

impl PreviewPlayerManager {
  pub(super) fn pane_annotations(&self, pane: u32) -> Vec<Annotation> {
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
  #[allow(clippy::too_many_arguments)]
  pub(super) fn annotation_gesture(
    &mut self,
    phase: SelectionGesturePhase,
    pane: u32,
    target: AnnotationGestureTarget,
    x: f64,
    y: f64,
    snap: u32,
    image_points: f64,
  ) -> Option<Commit> {
    if self.is_playing || self.annotation.mode == 0 || pane > 1 || self.annotation.text.is_some() {
      return None;
    }
    if matches!(phase, SelectionGesturePhase::Begin) {
      self.annotation.pane = Some(pane);
    }
    self.session_id?;
    let sources = self.sources.as_ref()?;
    let position_ms = self.position_ms.min(sources.duration_ms.saturating_sub(1));
    let source = sources.playback_layout.panes.get(pane as usize)?;
    let source_size = (source.source_width, source.source_height);
    let point = source_point(x, y, source_size);
    let clips = Arc::clone(&sources.annotation_clips);
    let modifiers = SnapModifiers::from_bits(snap);
    if matches!(phase, SelectionGesturePhase::Cancel) {
      let gesture = self.annotation.gesture.take()?;
      *clips.write().ok()? = gesture.before;
      self.annotation.selected = gesture.before_selected;
      self.publish_annotation_snap(source_size, &SnapResult::default());
      self.publish_annotation_handles();
      let _ = self.restart(PlaybackMode::InteractiveStill);
      return None;
    }
    if matches!(phase, SelectionGesturePhase::Begin) {
      let mut working = self.pane_annotations(pane);
      match target {
        AnnotationGestureTarget::None | AnnotationGestureTarget::Select { .. } => {
          self.annotation.selected = match target {
            AnnotationGestureTarget::Select { index } => working.get(index).map(|a| a.id.clone()),
            _ => None,
          };
          self.publish_annotation_snap(source_size, &SnapResult::default());
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
          return self.commit(
            pane,
            position_ms,
            working,
            self.annotation.selected.clone(),
            None,
          );
        }
        _ => {}
      }
      // A redaction is applied to the screen's frames alone.
      let redacting = drawing_kind(self.annotation.mode) == Some(AnnotationKind::Redact);
      if target == AnnotationGestureTarget::New && redacting && pane != 0 {
        return None;
      }
      let before = clips.read().ok()?.clone();
      // A disc's diameter and a text box's type size are in output pixels,
      // so the pane's drawn width is what turns them into source pixels.
      let image_width = self
        .selection_composition()
        .map(|composition| {
          if pane == 1 {
            composition.recording_output.camera.image_width
          } else {
            composition.recording_output.primary.image_width
          }
        })
        .unwrap_or_default();
      let edit = AnnotationEdit::begin(
        &mut working,
        target,
        point,
        self.annotation.defaults.as_ref(),
        drawing_kind(self.annotation.mode),
        self.annotation.counter_angle,
        source_per_output(source_size, image_width),
      )?;
      // Whether an annotation animates is not part of its dress, so the shape's
      // own constructor has no say in it: the switch's last setting is applied
      // to the fresh annotation here, where the recording's own timed
      // annotations are made. A redaction always starts still, however the
      // switch was last left, since animating shows what it hides.
      if target == AnnotationGestureTarget::New && !redacting {
        if let Some(animated) = self.annotation.animated {
          if let Some(annotation) = working.iter_mut().find(|m| m.id == edit.selected_id()) {
            annotation.animated = animated;
          }
        }
      }
      let before_selected = self.annotation.selected.clone();
      self.annotation.selected = Some(edit.selected_id().to_owned());
      // The field excludes the annotation the gesture holds, so a counter can
      // never snap back to the place it started from.
      let field = SnapField::new(source_size, &working, edit.selected_id(), image_width);
      self.annotation.gesture = Some(Gesture {
        pane,
        position: position_ms,
        edit,
        working,
        before,
        before_selected,
        field,
      });
      // The detector costs far too much to run between two pointer samples,
      // so the press starts it while the hand is still holding steady.
      if modifiers.position {
        let _ = self.annotation_anchors(pane, position_ms, source_size);
      }
      self.publish_annotation_snap(source_size, &SnapResult::default());
    } else {
      let threshold = threshold_source_px(source_size.0, image_points);
      let anchors = modifiers
        .position
        .then(|| self.annotation_anchors(pane, position_ms, source_size))
        .flatten();
      let Gesture {
        edit,
        working,
        field,
        ..
      } = self.annotation.gesture.as_mut()?;
      field.anchors = anchors;
      edit.set_source_per_point(source_per_point(source_size.0, image_points));
      let request = threshold.map(|threshold| SnapRequest { field, threshold });
      let result = edit.update(working, point, modifiers, request);
      // A gesture that has ended leaves nothing on screen to explain.
      let shown = if matches!(phase, SelectionGesturePhase::End) {
        SnapResult::default()
      } else {
        result
      };
      self.publish_annotation_snap(source_size, &shown);
    }
    let gesture = self.annotation.gesture.as_ref()?;
    *clips.write().ok()? = provisional_clips(
      &gesture.before,
      &gesture.working,
      gesture.pane,
      gesture.position,
    );
    let ended = matches!(phase, SelectionGesturePhase::End);
    if ended {
      let gesture = self.annotation.gesture.take()?;
      // A fresh text box goes straight on to being typed into, and React
      // groups the press that made it and the typing into one edit.
      let id = gesture.edit.selected_id().to_owned();
      let fresh_text = gesture.edit.is_new()
        && gesture.working.iter().any(|annotation| {
          annotation.id == id && matches!(annotation.shape, AnnotationShape::Text { .. })
        });
      if fresh_text {
        return self.begin_text_session(gesture.pane, gesture.position, id);
      }
      self.publish_annotation_handles();
      let _ = self.restart(PlaybackMode::InteractiveStill);
      return self.commit(
        pane,
        gesture.position,
        gesture.working,
        self.annotation.selected.clone(),
        None,
      );
    }
    self.publish_annotation_handles();
    // The Metal workspace re-encodes from its retained scene on a restart,
    // which is cheap enough to do per pointer sample. On Windows a restart
    // seeks the still decoder, which puts a decode between the hand and the
    // arrow on every sample; the pane already holds the frame, so it is
    // re-presented with the new annotations instead, and only the end of the
    // gesture restarts the worker to bring it back in step.
    if !self.redraw_annotation_frame(pane, position_ms) {
      let _ = self.restart(PlaybackMode::InteractiveStill);
    }
    None
  }
}

/// The clips a gesture or a typing session draws through: the clips it began
/// on, with every annotation it holds written into its own clip. One that has
/// no clip yet gets a provisional one reaching back a draw-in, the way the
/// editor's own placement does, so it is finished drawing at the playhead and
/// visible under the hand.
pub(super) fn provisional_clips(
  before: &[RecordingAnnotationClip],
  working: &[Annotation],
  pane: u32,
  position: u64,
) -> Vec<RecordingAnnotationClip> {
  let mut next = before.to_vec();
  for annotation in working {
    if let Some(clip) = next.iter_mut().find(|c| c.annotation.id == annotation.id) {
      // A pinned annotation was shown where its pin put it at `position`.
      fold(clip, annotation, position);
    } else {
      next.push(RecordingAnnotationClip {
        pin: None,
        annotation: annotation.clone(),
        track_id: track(pane),
        start_ms: position
          .saturating_sub(crate::editor::annotations::reveal::REVEAL_DRAW_IN_MS as u64),
        end_ms: position.saturating_add(3000),
      });
    }
  }
  next
}
