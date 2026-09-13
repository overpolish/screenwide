// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::*;
use super::*;

pub(in crate::ruler::snapshot) fn guide_pointer_position(axis: GuideAxis, point: Point) -> f64 {
  match axis {
    GuideAxis::Vertical => point.x,
    GuideAxis::Horizontal => point.y,
  }
}

pub(in crate::ruler::snapshot) fn guide_cross_axis_position(axis: GuideAxis, point: Point) -> f64 {
  match axis {
    GuideAxis::Vertical => point.y,
    GuideAxis::Horizontal => point.x,
  }
}

pub(in crate::ruler::snapshot) fn update_guide(session: &mut Session, pointer: RulerPointer) {
  let Some(mut gesture) = session.guide else {
    return;
  };
  let raw = guide_pointer_position(gesture.visual.axis, pointer.world);
  let retain_snap = gesture.snapped
    && gesture.visual.display_id == pointer.display_id
    && (gesture.visual.position - raw).abs() * pointer.zoom <= GUIDE_RELEASE_RADIUS;
  if !retain_snap {
    let snapped = session
      .displays
      .iter()
      .find(|snapshot| snapshot.display.id == pointer.display_id)
      .and_then(|snapshot| snap_guide(snapshot, gesture.visual.axis, pointer, session.tolerance));
    gesture.visual.display_id = pointer.display_id;
    gesture.visual.position = snapped.unwrap_or(raw);
    gesture.snapped = snapped.is_some();
  }
  session.guide = Some(gesture);
}

pub(in crate::ruler::snapshot) fn update_radius(session: &mut Session, pointer: RulerPointer) {
  if session.radius.is_none() {
    return;
  }
  let visual = session
    .displays
    .iter()
    .find(|snapshot| snapshot.display.id == pointer.display_id)
    .and_then(|snapshot| {
      let display = snapshot.display;
      let scale_x = display.size.width / f64::from(snapshot.image.width.max(1));
      let scale_y = display.size.height / f64::from(snapshot.image.height.max(1));
      let cursor = Point {
        x: (pointer.world.x - display.origin.x) / scale_x,
        y: (pointer.world.y - display.origin.y) / scale_y,
      };
      corner_radius_at(
        &snapshot.boxes_by_tolerance[session.tolerance.index()],
        cursor,
        &snapshot.gradients,
        session.tolerance.threshold(),
        scale_x,
        scale_y,
      )
      .map(|estimate| RulerRadiusVisual {
        id: 0,
        display_id: display.id,
        bounds: Rect::from_xywh(
          display.origin.x + f64::from(estimate.bounds.x) * scale_x,
          display.origin.y + f64::from(estimate.bounds.y) * scale_y,
          f64::from(estimate.bounds.width) * scale_x,
          f64::from(estimate.bounds.height) * scale_y,
        ),
        corner: estimate.corner,
        radius: f64::from(estimate.radius) * (scale_x + scale_y) * 0.5,
        low_confidence: estimate.low_confidence,
        draft: true,
        hovered: false,
        hover_alpha: 0.0,
        label_anchor: None,
        label_hidden: false,
      })
    });
  session.radius = Some(RadiusGesture { visual });
}

pub(in crate::ruler::snapshot) fn update_guide_drag_session(
  session: &mut Session,
  pointer: RulerPointer,
) -> Option<()> {
  let mut drag = session.guide_drag?;
  if !drag.changed {
    let distance =
      (pointer.screen.x - drag.start_screen.x).hypot(pointer.screen.y - drag.start_screen.y);
    if distance < DRAG_THRESHOLD {
      session.hovered_target = Some(HoverTarget::Guide(drag.id));
      return Some(());
    }
    record_history(session);
    drag.changed = true;
  }

  let raw = guide_pointer_position(drag.original.axis, pointer.world);
  let current = session
    .document
    .guides
    .iter()
    .find(|guide| guide.id == drag.id)
    .copied()?;
  let retain_snap = drag.snapped
    && current.display_id == pointer.display_id
    && (current.position - raw).abs() * pointer.zoom <= GUIDE_RELEASE_RADIUS;
  let (position, snapped) = if retain_snap {
    (current.position, true)
  } else {
    let snapped = session
      .displays
      .iter()
      .find(|snapshot| snapshot.display.id == pointer.display_id)
      .and_then(|snapshot| snap_guide(snapshot, drag.original.axis, pointer, session.tolerance));
    (snapped.unwrap_or(raw), snapped.is_some())
  };
  let guide = session
    .document
    .guides
    .iter_mut()
    .find(|guide| guide.id == drag.id)?;
  guide.display_id = pointer.display_id;
  guide.position = position;
  if current.display_id != pointer.display_id {
    if let Some(display) = session
      .displays
      .iter()
      .find(|snapshot| snapshot.display.id == pointer.display_id)
      .map(|snapshot| snapshot.display)
    {
      guide.anchor = match guide.axis {
        GuideAxis::Vertical => guide
          .anchor
          .clamp(display.origin.y, display.origin.y + display.size.height),
        GuideAxis::Horizontal => guide
          .anchor
          .clamp(display.origin.x, display.origin.x + display.size.width),
      };
    }
  }
  drag.snapped = snapped;
  session.guide_drag = Some(drag);
  session.hovered_target = Some(HoverTarget::Guide(drag.id));
  reconcile_guide_gaps(&mut session.document);
  Some(())
}

pub(in crate::ruler::snapshot) fn reconcile_guide_gaps(document: &mut Document) {
  let mut pairs = Vec::new();
  for guide in &document.guides {
    let mut peers = document
      .guides
      .iter()
      .filter(|peer| peer.display_id == guide.display_id && peer.axis == guide.axis)
      .copied()
      .collect::<Vec<_>>();
    peers.sort_by(|left, right| {
      left
        .position
        .total_cmp(&right.position)
        .then_with(|| left.id.cmp(&right.id))
    });
    for pair in peers.windows(2) {
      let first_id = pair[0].id.min(pair[1].id);
      let second_id = pair[0].id.max(pair[1].id);
      if !pairs.contains(&(first_id, second_id)) {
        pairs.push((first_id, second_id));
      }
    }
  }

  document
    .guide_gaps
    .retain(|gap| pairs.contains(&(gap.first_id, gap.second_id)));
  for (first_id, second_id) in pairs {
    if document
      .guide_gaps
      .iter()
      .any(|gap| gap.first_id == first_id && gap.second_id == second_id)
    {
      continue;
    }
    document.next_id = document.next_id.wrapping_add(1).max(1);
    document.guide_gaps.push(GuideGapArtifact {
      id: document.next_id,
      first_id,
      second_id,
      label: ArtifactLabel::default(),
    });
  }
}

pub(in crate::ruler::snapshot) fn guide_gap_visuals(session: &Session) -> Vec<RulerGuideGapVisual> {
  let now = Instant::now();
  session
    .document
    .guide_gaps
    .iter()
    .filter_map(|gap| {
      let first = session
        .document
        .guides
        .iter()
        .find(|guide| guide.id == gap.first_id)?;
      let second = session
        .document
        .guides
        .iter()
        .find(|guide| guide.id == gap.second_id)?;
      if first.display_id != second.display_id || first.axis != second.axis {
        return None;
      }
      let owner = if first.id > second.id { first } else { second };
      let axis = match first.axis {
        GuideAxis::Vertical => ProbeAxis::Horizontal,
        GuideAxis::Horizontal => ProbeAxis::Vertical,
      };
      let default_position = (first.anchor + second.anchor) * 0.5;
      let position = gap
        .label
        .anchor
        .map_or(default_position, |anchor| match axis {
          ProbeAxis::Horizontal => anchor.y,
          ProbeAxis::Vertical => anchor.x,
        });
      Some(RulerGuideGapVisual {
        id: gap.id,
        owner_id: owner.id,
        display_id: first.display_id,
        axis,
        start: first.position.min(second.position),
        end: first.position.max(second.position),
        position,
        hovered: session.hovered_target == Some(HoverTarget::GuideGap(gap.id)),
        hover_alpha: hover_alpha(session, HoverTarget::GuideGap(gap.id), now),
        label_anchor: gap.label.anchor,
        label_hidden: gap.label.hidden,
      })
    })
    .collect()
}
