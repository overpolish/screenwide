// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl RulerState {
  pub(crate) fn interaction_active(&self) -> bool {
    let session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    session.drag.pending.is_some()
      || session.drag.start.is_some()
      || session.label_drag.is_some()
      || session.range.is_some()
      || session.guide.is_some()
      || session.guide_drag.is_some()
      || session.radius.is_some()
  }

  pub(crate) fn hover_fade_active(&self) -> bool {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    expire_hover_exit(&mut session, Instant::now());
    session.hover_exit.is_some()
  }

  pub(crate) fn set_option_active(&self, active: bool) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    session.option_active = active;
    let pointer = pointer_from_visual(session.visual?);
    refresh_visual(&mut session, pointer)
  }

  pub(crate) fn copy_colour(&self) -> Option<(RulerVisual, String)> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut visual = session.visual?;
    let text = visual.hex();
    session.copied_until = Some(Instant::now() + COPIED_FEEDBACK_DURATION);
    visual.copied = true;
    session.visual = Some(visual);
    Some((visual, text))
  }

  pub(crate) fn copy_latest_artifact(&self) -> Option<(RulerVisual, String)> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let target = latest_target(&session.document)?;
    let text = artifact_text(&session.document, target)?;
    let mut visual = session.visual?;
    session.copied_until = Some(Instant::now() + COPIED_FEEDBACK_DURATION);
    visual.copied = true;
    session.visual = Some(visual);
    Some((visual, text))
  }

  pub(crate) fn delete_targeted_artifact(&self) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let target = session
      .hovered_target
      .or_else(|| latest_target(&session.document))?;
    record_history(&mut session);
    match target {
      HoverTarget::Measurement(id) => {
        let index = session
          .document
          .measurements
          .iter()
          .position(|item| item.id == id)?;
        session.document.measurements.remove(index);
      }
      HoverTarget::Probe(id) => {
        let index = session
          .document
          .probes
          .iter()
          .position(|item| item.id == id)?;
        session.document.probes.remove(index);
      }
      HoverTarget::Guide(id) => {
        let index = session
          .document
          .guides
          .iter()
          .position(|item| item.id == id)?;
        session.document.guides.remove(index);
        reconcile_guide_gaps(&mut session.document);
      }
      HoverTarget::GuideGap(id) => {
        let owner_id = session
          .document
          .guide_gaps
          .iter()
          .find(|item| item.id == id)?
          .second_id;
        let index = session
          .document
          .guides
          .iter()
          .position(|item| item.id == owner_id)?;
        session.document.guides.remove(index);
        reconcile_guide_gaps(&mut session.document);
      }
      HoverTarget::Radius(id) => {
        let index = session
          .document
          .radii
          .iter()
          .position(|item| item.id == id)?;
        session.document.radii.remove(index);
      }
    }
    session.hovered_target = None;
    session.hover_exit = None;
    session.label_drag = None;
    session.settle = None;
    let pointer = pointer_from_visual(session.visual?);
    refresh_visual(&mut session, pointer)
  }

  pub(crate) fn undo(&self) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous = session.undo.pop()?;
    let current = std::mem::replace(&mut session.document, previous);
    session.redo.push(current);
    clear_transient_artifact_state(&mut session);
    let pointer = pointer_from_visual(session.visual?);
    refresh_visual(&mut session, pointer)
  }
}
