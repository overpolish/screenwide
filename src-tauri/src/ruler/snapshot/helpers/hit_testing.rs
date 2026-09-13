// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::*;
use super::*;

pub(in crate::ruler::snapshot) fn record_history(session: &mut Session) {
  session.undo.push(session.document.clone());
  trim_history(&mut session.undo);
  session.redo.clear();
}

pub(in crate::ruler::snapshot) fn trim_history(history: &mut Vec<Document>) {
  let excess = history.len().saturating_sub(HISTORY_LIMIT);
  if excess > 0 {
    history.drain(..excess);
  }
}

pub(in crate::ruler::snapshot) fn clear_transient_artifact_state(session: &mut Session) {
  session.drag = Drag::default();
  session.hovered_target = None;
  session.label_drag = None;
  session.range = None;
  session.guide = None;
  session.guide_drag = None;
  session.settle = None;
}

pub(in crate::ruler::snapshot) fn hit_test_measurement(
  measurements: &[Measurement],
  point: Point,
  hit_slop: f64,
) -> Option<u64> {
  measurements.iter().rev().find_map(|measurement| {
    let bounds = measurement.bounds;
    let outer = Rect::from_xywh(
      bounds.origin.x - hit_slop,
      bounds.origin.y - hit_slop,
      bounds.size.width + hit_slop * 2.0,
      bounds.size.height + hit_slop * 2.0,
    );
    let inner_width = (bounds.size.width - hit_slop * 2.0).max(0.0);
    let inner_height = (bounds.size.height - hit_slop * 2.0).max(0.0);
    let inner = Rect::from_xywh(
      bounds.origin.x + hit_slop,
      bounds.origin.y + hit_slop,
      inner_width,
      inner_height,
    );
    (rect_contains(outer, point) && !rect_contains(inner, point)).then_some(measurement.id)
  })
}

pub(in crate::ruler::snapshot) fn hit_test_probe(
  probes: &[ProbeArtifact],
  point: Point,
  hit_slop: f64,
) -> Option<u64> {
  probes.iter().rev().find_map(|probe| {
    let start = probe.start.min(probe.end) - hit_slop;
    let end = probe.start.max(probe.end) + hit_slop;
    let hit = match probe.axis {
      ProbeAxis::Horizontal => {
        point.x >= start && point.x <= end && (point.y - probe.position).abs() <= hit_slop
      }
      ProbeAxis::Vertical => {
        point.y >= start && point.y <= end && (point.x - probe.position).abs() <= hit_slop
      }
    };
    hit.then_some(probe.id)
  })
}

pub(in crate::ruler::snapshot) fn hit_test_guide(
  session: &Session,
  point: Point,
  hit_slop: f64,
) -> Option<u64> {
  session.document.guides.iter().rev().find_map(|guide| {
    let display = session
      .displays
      .iter()
      .find(|snapshot| snapshot.display.id == guide.display_id)?
      .display;
    let within_display = point.x >= display.origin.x
      && point.y >= display.origin.y
      && point.x <= display.origin.x + display.size.width
      && point.y <= display.origin.y + display.size.height;
    let hit = match guide.axis {
      GuideAxis::Vertical => (point.x - guide.position).abs() <= hit_slop,
      GuideAxis::Horizontal => (point.y - guide.position).abs() <= hit_slop,
    };
    (within_display && hit).then_some(guide.id)
  })
}

pub(in crate::ruler::snapshot) fn hit_test_guide_gap(
  session: &Session,
  point: Point,
  hit_slop: f64,
) -> Option<u64> {
  guide_gap_visuals(session)
    .into_iter()
    .rev()
    .filter(|gap| !gap.label_hidden)
    .find_map(|gap| {
      let start = gap.start.min(gap.end) - hit_slop;
      let end = gap.start.max(gap.end) + hit_slop;
      let hit = match gap.axis {
        ProbeAxis::Horizontal => {
          point.x >= start && point.x <= end && (point.y - gap.position).abs() <= hit_slop
        }
        ProbeAxis::Vertical => {
          point.y >= start && point.y <= end && (point.x - gap.position).abs() <= hit_slop
        }
      };
      hit.then_some(gap.id)
    })
}

pub(in crate::ruler::snapshot) fn target_id(target: HoverTarget) -> u64 {
  match target {
    HoverTarget::Measurement(id)
    | HoverTarget::Probe(id)
    | HoverTarget::Guide(id)
    | HoverTarget::GuideGap(id)
    | HoverTarget::Radius(id) => id,
  }
}

pub(in crate::ruler::snapshot) fn radius_geometry(radius: &RadiusArtifact) -> (Point, Point) {
  let sign_x = if radius.corner.right() { 1.0 } else { -1.0 };
  let sign_y = if radius.corner.bottom() { 1.0 } else { -1.0 };
  let corner = Point {
    x: radius.bounds.origin.x
      + if radius.corner.right() {
        radius.bounds.size.width
      } else {
        0.0
      },
    y: radius.bounds.origin.y
      + if radius.corner.bottom() {
        radius.bounds.size.height
      } else {
        0.0
      },
  };
  let center = Point {
    x: corner.x - sign_x * radius.radius,
    y: corner.y - sign_y * radius.radius,
  };
  let diagonal = std::f64::consts::FRAC_1_SQRT_2;
  let arc_midpoint = Point {
    x: center.x + sign_x * radius.radius * diagonal,
    y: center.y + sign_y * radius.radius * diagonal,
  };
  (center, arc_midpoint)
}

pub(in crate::ruler::snapshot) fn distance_to_segment(
  point: Point,
  start: Point,
  end: Point,
) -> f64 {
  let dx = end.x - start.x;
  let dy = end.y - start.y;
  let length_squared = dx * dx + dy * dy;
  if length_squared <= f64::EPSILON {
    return (point.x - start.x).hypot(point.y - start.y);
  }
  let t = (((point.x - start.x) * dx + (point.y - start.y) * dy) / length_squared).clamp(0.0, 1.0);
  (point.x - (start.x + dx * t)).hypot(point.y - (start.y + dy * t))
}

pub(in crate::ruler::snapshot) fn hit_test_radius(
  radii: &[RadiusArtifact],
  point: Point,
  hit_slop: f64,
) -> Option<u64> {
  radii.iter().rev().find_map(|radius| {
    let (center, arc_midpoint) = radius_geometry(radius);
    let radial_hit =
      ((point.x - center.x).hypot(point.y - center.y) - radius.radius).abs() <= hit_slop;
    let correct_quadrant = if radius.corner.right() {
      point.x >= center.x - hit_slop
    } else {
      point.x <= center.x + hit_slop
    } && if radius.corner.bottom() {
      point.y >= center.y - hit_slop
    } else {
      point.y <= center.y + hit_slop
    };
    let line_hit = distance_to_segment(point, center, arc_midpoint) <= hit_slop;
    (line_hit || (radial_hit && correct_quadrant)).then_some(radius.id)
  })
}

pub(in crate::ruler::snapshot) fn hit_test_artifact(
  session: &Session,
  pointer: RulerPointer,
) -> Option<HoverTarget> {
  let slop = ARTIFACT_HIT_SLOP / pointer.zoom;
  [
    hit_test_measurement(&session.document.measurements, pointer.world, slop)
      .map(HoverTarget::Measurement),
    hit_test_probe(&session.document.probes, pointer.world, slop).map(HoverTarget::Probe),
    hit_test_guide(session, pointer.world, slop).map(HoverTarget::Guide),
    hit_test_guide_gap(session, pointer.world, slop).map(HoverTarget::GuideGap),
    hit_test_radius(&session.document.radii, pointer.world, slop).map(HoverTarget::Radius),
  ]
  .into_iter()
  .flatten()
  .max_by_key(|target| target_id(*target))
}

pub(in crate::ruler::snapshot) fn latest_target(document: &Document) -> Option<HoverTarget> {
  [
    document
      .measurements
      .last()
      .map(|item| HoverTarget::Measurement(item.id)),
    document
      .probes
      .last()
      .map(|item| HoverTarget::Probe(item.id)),
    document
      .guides
      .last()
      .map(|item| HoverTarget::Guide(item.id)),
    document
      .guide_gaps
      .last()
      .map(|item| HoverTarget::GuideGap(item.id)),
    document
      .radii
      .last()
      .map(|item| HoverTarget::Radius(item.id)),
  ]
  .into_iter()
  .flatten()
  .max_by_key(|target| target_id(*target))
}

pub(in crate::ruler::snapshot) fn artifact_text(
  document: &Document,
  target: HoverTarget,
) -> Option<String> {
  match target {
    HoverTarget::Measurement(id) => document
      .measurements
      .iter()
      .find(|measurement| measurement.id == id)
      .map(|measurement| measurement_text(measurement.bounds)),
    HoverTarget::Probe(id) => document
      .probes
      .iter()
      .find(|probe| probe.id == id)
      .copied()
      .map(probe_text),
    HoverTarget::Guide(id) => document
      .guides
      .iter()
      .find(|guide| guide.id == id)
      .map(|guide| format!("{} px", guide.position.round() as i64)),
    HoverTarget::GuideGap(id) => document
      .guide_gaps
      .iter()
      .find(|gap| gap.id == id)
      .and_then(|gap| {
        let first = document
          .guides
          .iter()
          .find(|guide| guide.id == gap.first_id)?;
        let second = document
          .guides
          .iter()
          .find(|guide| guide.id == gap.second_id)?;
        Some(format!(
          "{} px",
          (second.position - first.position).abs().round() as u64
        ))
      }),
    HoverTarget::Radius(id) => document
      .radii
      .iter()
      .find(|radius| radius.id == id)
      .map(|radius| {
        format!(
          "{}{} px",
          if radius.low_confidence { "≈ " } else { "" },
          radius.radius.round() as u64
        )
      }),
  }
}
