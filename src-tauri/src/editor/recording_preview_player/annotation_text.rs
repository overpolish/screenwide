// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Typing into a recording's text box.
//!
//! Every change is written into the clips, redrawn at once and handed to React
//! as it is typed, inside one edit gesture React opens on the first commit and
//! closes on the last. React's own clips keep arriving meanwhile - the panel
//! may dress the box as it is typed into - and the text typed so far is laid
//! over them, since React trails the typing by a round trip.

use super::gesture::Commit;
use super::*;
use crate::editor::annotations::text::edit::{AnnotationTextEdit, TextEditPhase};
use crate::editor::annotations::AnnotationShape;
use crate::editor::preview_platform::AnnotationTextPhase;

pub(super) struct TextSession {
  pane: u32,
  position: u64,
  edit: AnnotationTextEdit,
}

impl PreviewPlayerManager {
  /// Opens the box `id` on `pane` for typing, and answers the commit that
  /// tells React the typing began.
  pub(super) fn begin_text_session(
    &mut self,
    pane: u32,
    position: u64,
    id: String,
  ) -> Option<Commit> {
    self.session_id?;
    let annotations = self.pane_annotations(pane);
    let annotation = annotations.iter().find(|annotation| annotation.id == id)?;
    let AnnotationShape::Text { text, .. } = &annotation.shape else {
      return None;
    };
    let (text, dark_ink) = (
      text.clone(),
      crate::editor::annotations::text::model::dark_ink(&annotation.style),
    );
    self.annotation.text = Some(TextSession {
      pane,
      position,
      edit: AnnotationTextEdit::begin(id.clone(), text.clone()),
    });
    self.annotation.selected = Some(id.clone());
    self.publish_annotation_handles();
    let _ = self.restart(PlaybackMode::InteractiveStill);
    // The grips are published pane by pane, so the box's place in them is
    // counted across the panes before it.
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    if let (Some(index), Some(surface)) = (
      self
        .annotation_targets()
        .iter()
        .position(|(target, annotation)| *target == pane && annotation.id == id),
      self
        .sources
        .as_ref()
        .and_then(|s| s.preview_surface.as_ref()),
    ) {
      surface.begin_annotation_text(index, &text, dark_ink);
    }
    self.commit(
      pane,
      position,
      annotations,
      Some(id),
      Some(TextEditPhase::Begin),
    )
  }

  /// Lays the text typed so far over the box's clip, and redraws it.
  fn write_text_session(&mut self) {
    let Some(session) = self.annotation.text.as_ref() else {
      return;
    };
    let (pane, position) = (session.pane, session.position);
    if let Some(mut clips) = self
      .sources
      .as_ref()
      .and_then(|sources| sources.annotation_clips.write().ok())
    {
      for clip in clips.iter_mut() {
        session
          .edit
          .apply(std::slice::from_mut(&mut clip.annotation));
      }
    }
    self.publish_annotation_handles();
    if !self.redraw_annotation_frame(pane, position) {
      let _ = self.restart(PlaybackMode::InteractiveStill);
    }
  }

  /// One report from the box being typed into, and what React is to commit
  /// for it.
  pub(super) fn annotation_text(
    &mut self,
    phase: AnnotationTextPhase,
    pane: u32,
    index: u32,
    text: &str,
    revision: u64,
  ) -> Option<Commit> {
    self.session_id?;
    match phase {
      AnnotationTextPhase::Open => {
        if self.is_playing
          || self.annotation.gesture.is_some()
          || self.annotation.text.is_some()
          || pane > 1
        {
          return None;
        }
        let annotation = self
          .pane_annotations(pane)
          .into_iter()
          .nth(index as usize)?;
        if !matches!(annotation.shape, AnnotationShape::Text { .. }) {
          return None;
        }
        let sources = self.sources.as_ref()?;
        let position = self.position_ms.min(sources.duration_ms.saturating_sub(1));
        self.begin_text_session(pane, position, annotation.id)
      }
      AnnotationTextPhase::Change => {
        if !self.annotation.text.as_mut()?.edit.take(text, revision) {
          return None;
        }
        self.write_text_session();
        let session = self.annotation.text.as_ref()?;
        self.commit(
          session.pane,
          session.position,
          self.pane_annotations(session.pane),
          Some(session.edit.id().to_owned()),
          Some(TextEditPhase::Update),
        )
      }
      AnnotationTextPhase::End => {
        self.annotation.text.as_mut()?.edit.take(text, revision);
        self.write_text_session();
        let session = self.annotation.text.take()?;
        let (pane, position) = (session.pane, session.position);
        let mut annotations = self.pane_annotations(pane);
        let id = session.edit.id().to_owned();
        let kept = session.edit.finish(&mut annotations);
        if !kept {
          // A box left with nothing to read goes, clip and all.
          if let Some(mut clips) = self
            .sources
            .as_ref()
            .and_then(|sources| sources.annotation_clips.write().ok())
          {
            clips.retain(|clip| clip.annotation.id != id);
          }
        }
        self.annotation.selected = kept.then_some(id);
        self.publish_annotation_handles();
        let _ = self.restart(PlaybackMode::InteractiveStill);
        self.commit(
          pane,
          position,
          annotations,
          self.annotation.selected.clone(),
          Some(TextEditPhase::End),
        )
      }
    }
  }

  /// Lays the text typed so far over clips React sent while the box is open.
  /// A box React has not taken in yet - the press that made it is still on
  /// its way - keeps the clip it is drawn through.
  pub(super) fn merge_text_session(
    &self,
    incoming: &mut Vec<RecordingAnnotationClip>,
    previous: &[RecordingAnnotationClip],
  ) {
    let Some(session) = self.annotation.text.as_ref() else {
      return;
    };
    let id = session.edit.id();
    if !incoming.iter().any(|clip| clip.annotation.id == id) {
      if let Some(held) = previous.iter().find(|clip| clip.annotation.id == id) {
        incoming.push(held.clone());
      }
    }
    for clip in incoming.iter_mut() {
      session
        .edit
        .apply(std::slice::from_mut(&mut clip.annotation));
    }
  }
}
