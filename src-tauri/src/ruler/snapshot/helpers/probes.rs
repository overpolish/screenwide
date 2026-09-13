// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::*;
use super::*;

pub(in crate::ruler::snapshot) fn detected_boxes(
  snapshot: &DisplaySnapshot,
  tolerance: Tolerance,
) -> Vec<Rect> {
  snapshot.boxes_by_tolerance[tolerance.index()]
    .iter()
    .copied()
    .map(|item| {
      let display = snapshot.display;
      Rect::from_xywh(
        display.origin.x + f64::from(item.x) / f64::from(snapshot.image.width) * display.size.width,
        display.origin.y
          + f64::from(item.y) / f64::from(snapshot.image.height) * display.size.height,
        f64::from(item.width) / f64::from(snapshot.image.width) * display.size.width,
        f64::from(item.height) / f64::from(snapshot.image.height) * display.size.height,
      )
    })
    .collect()
}

pub(in crate::ruler::snapshot) fn probe_visuals(session: &Session) -> Vec<RulerProbeVisual> {
  let now = Instant::now();
  let mut visuals = session
    .document
    .probes
    .iter()
    .map(|probe| RulerProbeVisual {
      id: probe.id,
      display_id: 0,
      axis: probe.axis,
      start: probe.start,
      end: probe.end,
      position: probe.position,
      draft: false,
      hovered: session.hovered_target == Some(HoverTarget::Probe(probe.id)),
      hover_alpha: hover_alpha(session, HoverTarget::Probe(probe.id), now),
      label_anchor: probe.label.anchor,
      label_hidden: probe.label.hidden,
    })
    .collect::<Vec<_>>();
  if let Some(range) = session.range {
    visuals.push(range.draft);
    return visuals;
  }
  if session.drag.pending.is_some()
    || session.drag.start.is_some()
    || session.drag.draft.is_some()
    || session.hovered_target.is_some()
    || session.guide.is_some()
    || session.guide_drag.is_some()
    || session.radius.is_some()
  {
    return visuals;
  }
  let Some(visual) = session.visual else {
    return visuals;
  };
  if let Some(automatic) = automatic_probes(session, pointer_from_visual(visual)) {
    visuals.extend(automatic);
  }
  visuals
}

pub(in crate::ruler::snapshot) fn automatic_probes(
  session: &Session,
  pointer: RulerPointer,
) -> Option<[RulerProbeVisual; 2]> {
  let snapshot = session
    .displays
    .iter()
    .find(|snapshot| snapshot.display.id == pointer.display_id)?;
  let display = snapshot.display;
  let image_width = snapshot.image.width.max(1);
  let image_height = snapshot.image.height.max(1);
  let x = (((pointer.world.x - display.origin.x) / display.size.width) * f64::from(image_width))
    .floor()
    .clamp(0.0, f64::from(image_width.saturating_sub(1))) as u32;
  let y = (((pointer.world.y - display.origin.y) / display.size.height) * f64::from(image_height))
    .floor()
    .clamp(0.0, f64::from(image_height.saturating_sub(1))) as u32;
  let pixel_probes = if session.tolerance == Tolerance::Balanced {
    snapshot.probes.probes_at(x, y)
  } else {
    probes_at_threshold(&snapshot.gradients, x, y, session.tolerance.threshold())
  };
  let mut visuals: [RulerProbeVisual; 2] = pixel_probes
    .into_iter()
    .map(|probe| {
      let (axis_scale, axis_origin, position) = match probe.axis {
        ProbeAxis::Horizontal => (
          display.size.width / f64::from(image_width),
          display.origin.x,
          pointer.world.y,
        ),
        ProbeAxis::Vertical => (
          display.size.height / f64::from(image_height),
          display.origin.y,
          pointer.world.x,
        ),
      };
      RulerProbeVisual {
        id: 0,
        display_id: display.id,
        axis: probe.axis,
        start: axis_origin + f64::from(probe.start) * axis_scale,
        end: axis_origin + f64::from(probe.end) * axis_scale,
        position,
        draft: false,
        hovered: false,
        hover_alpha: 0.0,
        label_anchor: None,
        label_hidden: false,
      }
    })
    .collect::<Vec<_>>()
    .try_into()
    .ok()?;
  if session.option_active {
    clip_transient_probes_to_structural_edges(
      &session.document.guides,
      &session.document.measurements,
      pointer,
      &mut visuals,
    );
  }
  Some(visuals)
}

pub(in crate::ruler::snapshot) fn clip_transient_probes_to_structural_edges(
  guides: &[GuideArtifact],
  measurements: &[Measurement],
  pointer: RulerPointer,
  probes: &mut [RulerProbeVisual; 2],
) {
  for probe in probes {
    let (guide_axis, pointer_axis) = match probe.axis {
      ProbeAxis::Horizontal => (GuideAxis::Vertical, pointer.world.x),
      ProbeAxis::Vertical => (GuideAxis::Horizontal, pointer.world.y),
    };
    let mut positions = guides
      .iter()
      .filter(|guide| guide.display_id == pointer.display_id && guide.axis == guide_axis)
      .map(|guide| guide.position)
      .filter(|position| *position >= probe.start && *position <= probe.end)
      .collect::<Vec<_>>();
    positions.extend(measurements.iter().flat_map(|measurement| {
      let bounds = measurement.bounds;
      match probe.axis {
        ProbeAxis::Horizontal
          if pointer.world.y >= bounds.origin.y && pointer.world.y <= bounds.bottom() =>
        {
          [Some(bounds.origin.x), Some(bounds.right())]
        }
        ProbeAxis::Vertical
          if pointer.world.x >= bounds.origin.x && pointer.world.x <= bounds.right() =>
        {
          [Some(bounds.origin.y), Some(bounds.bottom())]
        }
        _ => [None, None],
      }
      .into_iter()
      .flatten()
      .filter(|position| *position >= probe.start && *position <= probe.end)
    }));
    probe.start = positions
      .iter()
      .copied()
      .filter(|position| *position <= pointer_axis)
      .max_by(|left, right| left.total_cmp(right))
      .map_or(probe.start, |position| probe.start.max(position));
    probe.end = positions
      .iter()
      .copied()
      .filter(|position| *position >= pointer_axis)
      .min_by(|left, right| left.total_cmp(right))
      .map_or(probe.end, |position| probe.end.min(position));
  }
}
