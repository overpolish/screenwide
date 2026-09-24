// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Typing into a screenshot's text box.
//!
//! Every change is presented natively at once and handed to React as it is
//! typed, inside one edit gesture React opens on the first commit and closes
//! on the last. React's own layouts keep arriving meanwhile - the panel may
//! dress the box as it is typed into - and the text typed so far is laid over
//! each, since React trails the typing by a round trip.

use super::super::ScreenshotWorkspaceOutputSettings;
use super::annotation_gesture::AnnotationCommit;
use super::state::PreviewManager;
use crate::editor::annotations::text::edit::{AnnotationTextEdit, TextEditPhase};
use crate::editor::annotations::AnnotationShape;
use crate::editor::preview_platform::AnnotationTextPhase;

pub(super) struct TextSession {
  pane_index: u32,
  edit: AnnotationTextEdit,
}

impl PreviewManager {
  pub(super) fn is_text(&self, pane_index: u32, id: &str) -> bool {
    self.annotations_for(pane_index).is_some_and(|annotations| {
      annotations.iter().any(|annotation| {
        annotation.id == id && matches!(annotation.shape, AnnotationShape::Text { .. })
      })
    })
  }

  /// Opens the box `id` for typing, and answers the commit that tells React
  /// the typing began: the pane's list with the box in it, the box chosen.
  pub(super) fn begin_text_session(
    &mut self,
    pane_index: u32,
    id: String,
  ) -> Option<AnnotationCommit> {
    let (index, annotation) = self
      .annotations_for(pane_index)?
      .iter()
      .enumerate()
      .find(|(_, annotation)| annotation.id == id)
      .map(|(index, annotation)| (index, annotation.clone()))?;
    let AnnotationShape::Text { text, .. } = &annotation.shape else {
      return None;
    };
    self.annotation_text = Some(TextSession {
      pane_index,
      edit: AnnotationTextEdit::begin(id.clone(), text.clone()),
    });
    self.present_annotation_gesture(pane_index, Some(id.as_str()));
    // The published grips are this pane's list in order, so the box's place
    // in it is its place in the grips.
    if let Some(surface) = self.surface.as_ref() {
      surface.begin_annotation_text(
        index,
        text,
        crate::editor::annotations::text::model::dark_ink(&annotation.style),
      );
    }
    let mut commit = self.commit_for(pane_index, Some(id))?;
    commit.text_edit = Some(TextEditPhase::Begin);
    Some(commit)
  }

  /// One report from the box being typed into, and what React is to commit
  /// for it.
  pub(crate) fn handle_annotation_text(
    &mut self,
    phase: AnnotationTextPhase,
    pane_index: u32,
    index: u32,
    text: &str,
    revision: u64,
  ) -> Option<AnnotationCommit> {
    match phase {
      AnnotationTextPhase::Open => {
        if self.annotation_gesture.is_some() || self.annotation_text.is_some() {
          return None;
        }
        // The box is opened on the list React last laid out, exactly as a
        // gesture begins on it.
        let base = self.react_output.clone().or_else(|| self.output.clone())?;
        self.output = Some(base);
        let annotation = self.annotations_for(pane_index)?.get(index as usize)?;
        if !matches!(annotation.shape, AnnotationShape::Text { .. }) {
          return None;
        }
        let id = annotation.id.clone();
        self.begin_text_session(pane_index, id)
      }
      AnnotationTextPhase::Change => {
        let session = self.annotation_text.as_mut()?;
        if !session.edit.take(text, revision) {
          return None;
        }
        let pane_index = session.pane_index;
        let id = session.edit.id().to_owned();
        let annotations = &mut self
          .output
          .as_mut()?
          .items
          .get_mut(pane_index as usize)?
          .output
          .annotations;
        self.annotation_text.as_ref()?.edit.apply(annotations);
        self.present_annotation_gesture(pane_index, Some(id.as_str()));
        let mut commit = self.commit_for(pane_index, Some(id))?;
        commit.text_edit = Some(TextEditPhase::Update);
        Some(commit)
      }
      AnnotationTextPhase::End => {
        let mut session = self.annotation_text.take()?;
        session.edit.take(text, revision);
        let pane_index = session.pane_index;
        let id = session.edit.id().to_owned();
        let annotations = &mut self
          .output
          .as_mut()?
          .items
          .get_mut(pane_index as usize)?
          .output
          .annotations;
        let selected = session.edit.finish(annotations).then_some(id);
        self.present_annotation_gesture(pane_index, selected.as_deref());
        let mut commit = self.commit_for(pane_index, selected)?;
        commit.text_edit = Some(TextEditPhase::End);
        Some(commit)
      }
    }
  }

  /// Lays the text typed so far over a layout React sent while the box is
  /// open. A box React has not taken in yet - the press that made it is still
  /// on its way - is kept from the list on screen.
  pub(super) fn merge_text_session(
    &self,
    mut output: ScreenshotWorkspaceOutputSettings,
  ) -> ScreenshotWorkspaceOutputSettings {
    let Some(session) = self.annotation_text.as_ref() else {
      return output;
    };
    let held = self
      .annotations_for(session.pane_index)
      .and_then(|annotations| annotations.iter().find(|a| a.id == session.edit.id()));
    if let Some(incoming) = output.items.get_mut(session.pane_index as usize) {
      let annotations = &mut incoming.output.annotations;
      if !session.edit.apply(annotations) {
        annotations.extend(held.cloned());
        session.edit.apply(annotations);
      }
    }
    output
  }
}
