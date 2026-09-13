// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::*;
use super::*;

pub(in crate::ruler::snapshot) fn label_target(kind: LabelKind, id: u64) -> HoverTarget {
  match kind {
    LabelKind::Measurement => HoverTarget::Measurement(id),
    LabelKind::Probe => HoverTarget::Probe(id),
    LabelKind::GuideGap => HoverTarget::GuideGap(id),
    LabelKind::Radius => HoverTarget::Radius(id),
  }
}

pub(in crate::ruler::snapshot) fn label_state(
  document: &Document,
  target: HoverTarget,
) -> Option<&ArtifactLabel> {
  match target {
    HoverTarget::Measurement(id) => document
      .measurements
      .iter()
      .find(|measurement| measurement.id == id)
      .map(|measurement| &measurement.label),
    HoverTarget::Probe(id) => document
      .probes
      .iter()
      .find(|probe| probe.id == id)
      .map(|probe| &probe.label),
    HoverTarget::GuideGap(id) => document
      .guide_gaps
      .iter()
      .find(|gap| gap.id == id)
      .map(|gap| &gap.label),
    HoverTarget::Radius(id) => document
      .radii
      .iter()
      .find(|radius| radius.id == id)
      .map(|radius| &radius.label),
    HoverTarget::Guide(_) => None,
  }
}

pub(in crate::ruler::snapshot) fn label_state_mut(
  document: &mut Document,
  target: HoverTarget,
) -> Option<&mut ArtifactLabel> {
  match target {
    HoverTarget::Measurement(id) => document
      .measurements
      .iter_mut()
      .find(|measurement| measurement.id == id)
      .map(|measurement| &mut measurement.label),
    HoverTarget::Probe(id) => document
      .probes
      .iter_mut()
      .find(|probe| probe.id == id)
      .map(|probe| &mut probe.label),
    HoverTarget::GuideGap(id) => document
      .guide_gaps
      .iter_mut()
      .find(|gap| gap.id == id)
      .map(|gap| &mut gap.label),
    HoverTarget::Radius(id) => document
      .radii
      .iter_mut()
      .find(|radius| radius.id == id)
      .map(|radius| &mut radius.label),
    HoverTarget::Guide(_) => None,
  }
}

pub(in crate::ruler::snapshot) fn update_label_drag_session(
  session: &mut Session,
  pointer: RulerPointer,
) -> Option<()> {
  let mut drag = session.label_drag?;
  if !drag.changed {
    let distance =
      (pointer.screen.x - drag.start_screen.x).hypot(pointer.screen.y - drag.start_screen.y);
    if distance < DRAG_THRESHOLD {
      session.hovered_target = Some(drag.target);
      return Some(());
    }
    record_history(session);
    drag.changed = true;
    session.label_drag = Some(drag);
  }
  let anchor = Point {
    x: pointer.world.x + drag.grab_offset.x,
    y: pointer.world.y + drag.grab_offset.y,
  };
  let label = label_state_mut(&mut session.document, drag.target)?;
  label.anchor = Some(anchor);
  label.hidden = false;
  session.label_drag = Some(drag);
  session.hovered_target = Some(drag.target);
  Some(())
}

pub(in crate::ruler::snapshot) fn rect_contains(rect: Rect, point: Point) -> bool {
  point.x >= rect.origin.x
    && point.y >= rect.origin.y
    && point.x <= rect.origin.x + rect.size.width
    && point.y <= rect.origin.y + rect.size.height
}

pub(in crate::ruler::snapshot) fn measurement_text(bounds: Rect) -> String {
  let width = bounds.size.width.round().max(0.0) as u64;
  let height = bounds.size.height.round().max(0.0) as u64;
  if bounds.size.height < 8.0 {
    format!("{width} px")
  } else if bounds.size.width < 8.0 {
    format!("{height} px")
  } else {
    format!("{width} × {height} px")
  }
}

pub(in crate::ruler::snapshot) fn probe_text(probe: ProbeArtifact) -> String {
  format!(
    "{} px",
    (probe.end - probe.start).abs().round().max(0.0) as u64
  )
}

pub(in crate::ruler::snapshot) fn mix_rect(from: Rect, to: Rect, amount: f64) -> Rect {
  Rect::from_xywh(
    from.origin.x + (to.origin.x - from.origin.x) * amount,
    from.origin.y + (to.origin.y - from.origin.y) * amount,
    from.size.width + (to.size.width - from.size.width) * amount,
    from.size.height + (to.size.height - from.size.height) * amount,
  )
}

pub(in crate::ruler::snapshot) fn ordered_rect(start: Point, end: Point) -> Rect {
  Rect::from_xywh(
    start.x.min(end.x),
    start.y.min(end.y),
    (end.x - start.x).abs(),
    (end.y - start.y).abs(),
  )
}
