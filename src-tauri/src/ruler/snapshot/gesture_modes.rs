// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl RulerState {
  pub(crate) fn finish_range(&self) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let range = session.range.take()?;
    record_history(&mut session);
    session.document.next_id = session.document.next_id.wrapping_add(1).max(1);
    let id = session.document.next_id;
    session.document.probes.push(ProbeArtifact {
      id,
      axis: range.axis,
      start: range.draft.start,
      end: range.draft.end,
      position: range.draft.position,
      label: ArtifactLabel::default(),
    });
    session.hovered_target = Some(HoverTarget::Probe(id));
    session.visual
  }

  pub(crate) fn cancel_range(&self) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    session.range.take()?;
    session.visual
  }

  pub(crate) fn hover_probe_label(&self, id: u64) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if !session.document.probes.iter().any(|probe| probe.id == id) {
      return None;
    }
    session.hovered_target = Some(HoverTarget::Probe(id));
    session.visual
  }

  pub(crate) fn hover_measurement_label(&self, id: u64) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if !session
      .document
      .measurements
      .iter()
      .any(|measurement| measurement.id == id)
    {
      return None;
    }
    session.hovered_target = Some(HoverTarget::Measurement(id));
    session.visual
  }

  pub(crate) fn hover_guide_gap_label(&self, id: u64) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if !session.document.guide_gaps.iter().any(|gap| gap.id == id) {
      return None;
    }
    session.hovered_target = Some(HoverTarget::GuideGap(id));
    session.visual
  }

  pub(crate) fn begin_label_drag(
    &self,
    kind: LabelKind,
    id: u64,
    pointer: RulerPointer,
    label_center: RulerPointer,
  ) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let target = label_target(kind, id);
    if label_state(&session.document, target)?.hidden {
      return None;
    }
    session.drag = Drag::default();
    session.range = None;
    session.guide = None;
    session.guide_drag = None;
    session.radius = None;
    session.settle = None;
    session.hovered_target = Some(target);
    session.label_drag = Some(LabelDrag {
      target,
      start_screen: pointer.screen,
      grab_offset: Point {
        x: label_center.world.x - pointer.world.x,
        y: label_center.world.y - pointer.world.y,
      },
      changed: false,
    });
    refresh_visual(&mut session, pointer)
  }

  pub(crate) fn update_label_drag(&self, pointer: RulerPointer) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    update_label_drag_session(&mut session, pointer)?;
    refresh_visual(&mut session, pointer)
  }
}
