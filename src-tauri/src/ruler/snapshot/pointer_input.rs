// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl RulerState {
  pub(crate) fn pointer_up(&self, pointer: RulerPointer) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if !session.active {
      return None;
    }
    if session.guide.is_some() {
      update_guide(&mut session, pointer);
      session.drag = Drag::default();
      return refresh_visual(&mut session, pointer);
    }
    if session.radius.is_some() {
      update_radius(&mut session, pointer);
      session.drag = Drag::default();
      return refresh_visual(&mut session, pointer);
    }
    if session.guide_drag.is_some() {
      update_guide_drag_session(&mut session, pointer)?;
      let id = session.guide_drag.take()?.id;
      let visual = refresh_visual(&mut session, pointer)?;
      session.hovered_target = Some(HoverTarget::Guide(id));
      return Some(visual);
    }
    if let Some(start) = session.drag.start {
      let raw = ordered_rect(start.world, pointer.world);
      if raw.size.width >= 2.0 || raw.size.height >= 2.0 {
        let snapped = snap_bounds(&session.boxes, raw);
        record_history(&mut session);
        session.document.next_id = session.document.next_id.wrapping_add(1).max(1);
        let id = session.document.next_id;
        session.document.measurements.push(Measurement {
          id,
          bounds: snapped,
          label: ArtifactLabel::default(),
        });
        session.hovered_target = Some(HoverTarget::Measurement(id));
        session.settle = settle_worthwhile(raw, snapped).then(|| Settle {
          id,
          from: raw,
          to: snapped,
          started: Instant::now(),
        });
      }
    }
    session.drag = Drag::default();
    refresh_visual(&mut session, pointer)
  }

  pub(crate) fn cancel_pointer(&self) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    session.drag = Drag::default();
    session.guide_drag = None;
    session.radius = None;
    let pointer = pointer_from_visual(session.visual?);
    refresh_visual(&mut session, pointer)
  }

  pub(crate) fn animation_frame(&self) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let pointer = pointer_from_visual(session.visual?);
    refresh_visual(&mut session, pointer)
  }

  pub(crate) fn toggle_crosshair(&self) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut visual = session.visual?;
    visual.crosshair = !visual.crosshair;
    visual.copied = session
      .copied_until
      .is_some_and(|deadline| deadline > Instant::now());
    session.visual = Some(visual);
    Some(visual)
  }

  pub(crate) fn toggle_centerlines(&self) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    session.centerlines_visible = !session.centerlines_visible;
    session.visual
  }

  pub(crate) fn cycle_tolerance(&self) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let pointer = pointer_from_visual(session.visual?);
    session.tolerance = session.tolerance.next();
    let tolerance = session.tolerance;
    session.boxes = session
      .displays
      .iter()
      .flat_map(|snapshot| detected_boxes(snapshot, tolerance))
      .collect();
    session.center_aid_cache = None;
    session.tolerance_until = Some(Instant::now() + TOLERANCE_FEEDBACK_DURATION);
    if let Some(guide) = &mut session.guide {
      guide.snapped = false;
    }
    refresh_visual(&mut session, pointer)
  }

  pub(crate) fn tolerance_notice(&self) -> Option<Tolerance> {
    let session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    session
      .tolerance_until
      .is_some_and(|deadline| deadline > Instant::now())
      .then_some(session.tolerance)
  }
}
