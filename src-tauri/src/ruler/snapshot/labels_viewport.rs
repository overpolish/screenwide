// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl RulerState {
  pub(crate) fn finish_label_drag(&self, pointer: RulerPointer) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    update_label_drag_session(&mut session, pointer)?;
    let target = session.label_drag.take()?.target;
    let visual = refresh_visual(&mut session, pointer)?;
    session.hovered_target = Some(target);
    Some(visual)
  }

  pub(crate) fn hover_radius_label(&self, id: u64) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if !session.document.radii.iter().any(|radius| radius.id == id) {
      return None;
    }
    session.hovered_target = Some(HoverTarget::Radius(id));
    session.visual
  }

  pub(crate) fn cancel_label_drag(&self) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    session.label_drag.take()?;
    session.visual
  }

  pub(crate) fn hide_label(&self, kind: LabelKind, id: u64) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let target = label_target(kind, id);
    if label_state(&session.document, target)?.hidden {
      return None;
    }
    record_history(&mut session);
    label_state_mut(&mut session.document, target)?.hidden = true;
    session.hovered_target = None;
    session.label_drag = None;
    session.visual
  }

  pub(crate) fn toggle_label_at(&self, pointer: RulerPointer) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let target = hit_test_artifact(&session, pointer)?;
    if let HoverTarget::Guide(id) = target {
      let gap_ids = session
        .document
        .guide_gaps
        .iter()
        .filter(|gap| gap.second_id == id)
        .map(|gap| gap.id)
        .collect::<Vec<_>>();
      let hidden = gap_ids
        .first()
        .and_then(|gap_id| {
          session
            .document
            .guide_gaps
            .iter()
            .find(|gap| gap.id == *gap_id)
        })
        .map(|gap| gap.label.hidden)?;
      record_history(&mut session);
      for gap in &mut session.document.guide_gaps {
        if gap_ids.contains(&gap.id) {
          gap.label.hidden = !hidden;
        }
      }
      session.hovered_target = hidden.then_some(target);
      return refresh_visual(&mut session, pointer);
    }
    let hidden = label_state(&session.document, target)?.hidden;
    record_history(&mut session);
    label_state_mut(&mut session.document, target)?.hidden = !hidden;
    session.hovered_target = hidden.then_some(target);
    refresh_visual(&mut session, pointer)
  }

  pub(crate) fn update_viewport(
    &self,
    display_id: u32,
    action: ViewportAction,
  ) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if !session.active {
      return None;
    }
    let index = session
      .displays
      .iter()
      .position(|snapshot| snapshot.display.id == display_id)?;
    let snapshot = &mut session.displays[index];
    let display = snapshot.display;
    let local_screen = match action {
      ViewportAction::Zoom { anchor, factor } => {
        snapshot
          .viewport
          .zoom_at(snapshot.display.size, anchor, factor);
        anchor
      }
      ViewportAction::Pan { anchor, delta } => {
        snapshot.viewport.pan_content(snapshot.display.size, delta);
        anchor
      }
      ViewportAction::Reset { anchor } => {
        snapshot.viewport.reset();
        anchor
      }
    };
    let local_screen = Point {
      x: local_screen
        .x
        .clamp(0.0, (display.size.width - f64::EPSILON).max(0.0)),
      y: local_screen
        .y
        .clamp(0.0, (display.size.height - f64::EPSILON).max(0.0)),
    };
    let screen = Point {
      x: display.origin.x + local_screen.x,
      y: display.origin.y + local_screen.y,
    };
    let pointer = map_pointer(&session.displays, screen)?;
    refresh_visual(&mut session, pointer)
  }

  pub(in crate::ruler) fn active_generation(&self) -> Option<u64> {
    let session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    session.active.then_some(session.generation)
  }
}
