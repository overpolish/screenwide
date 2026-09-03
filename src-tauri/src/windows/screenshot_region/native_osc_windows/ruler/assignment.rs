// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn has_label_anchor(x: f64, y: f64) -> bool {
  x.is_finite() && y.is_finite()
}

pub(super) fn point_in_surface(point: Point, view: Size) -> bool {
  point.x >= 0.0 && point.y >= 0.0 && point.x < view.width && point.y < view.height
}

pub(super) fn contains(rect: Rect, point: Point) -> bool {
  point.x >= rect.origin.x
    && point.y >= rect.origin.y
    && point.x < rect.right()
    && point.y < rect.bottom()
}

pub(super) fn overlap(left: Rect, right: Rect) -> f64 {
  let width = left.right().min(right.right()) - left.origin.x.max(right.origin.x);
  let height = left.bottom().min(right.bottom()) - left.origin.y.max(right.origin.y);
  if width <= 0.0 || height <= 0.0 {
    0.0
  } else {
    width * height
  }
}

/// Port of `label_anchor_surface` / `measurement_label_surface` (`:1219-1260`),
/// hoisted out of the surface because only the caller holding every surface can
/// answer "which display owns this label".
pub(crate) fn assign_labels(world: &[Rect], data: &RulerData) -> Vec<Vec<LabelItem>> {
  let mut owned = vec![Vec::new(); world.len()];
  for measurement in &data.measurements {
    if measurement.flags & 8 != 0 {
      continue;
    }
    let index = if has_label_anchor(measurement.label_anchor_x, measurement.label_anchor_y) {
      containing(
        world,
        Point {
          x: measurement.label_anchor_x,
          y: measurement.label_anchor_y,
        },
      )
    } else {
      largest_overlap(
        world,
        Rect::from_xywh(
          measurement.x,
          measurement.y,
          measurement.width,
          measurement.height,
        ),
      )
    };
    if let Some(index) = index {
      owned[index].push(LabelItem::Measurement(*measurement));
    }
  }
  for probe in &data.probes {
    let draft = probe.flags & 1 != 0;
    let labelled = probe.flags & 4 == 0 && probe.flags & 8 == 0 && (probe.id != 0 || draft);
    if !labelled {
      continue;
    }
    if let Some(index) = containing(world, probe_midpoint(*probe)) {
      owned[index].push(LabelItem::Probe(*probe));
    }
  }
  for gap in &data.guide_gaps {
    if gap.flags & 2 != 0 {
      continue;
    }
    let probe = guide_gap_probe(*gap);
    if let Some(index) = containing(world, probe_midpoint(probe)) {
      owned[index].push(LabelItem::GuideGap(probe));
    }
  }
  for radius in &data.radii {
    if radius.flags & 8 != 0 {
      continue;
    }
    let probe = radius_label_probe(*radius);
    if let Some(index) = containing(world, probe_midpoint(probe)) {
      owned[index].push(LabelItem::Radius(*radius));
    }
  }
  owned
}

pub(super) fn probe_midpoint(probe: ProbePacket) -> Point {
  if has_label_anchor(probe.label_anchor_x, probe.label_anchor_y) {
    return Point {
      x: probe.label_anchor_x,
      y: probe.label_anchor_y,
    };
  }
  if probe.axis == 1 {
    Point {
      x: (probe.start + probe.end) * 0.5,
      y: probe.position,
    }
  } else {
    Point {
      x: probe.position,
      y: (probe.start + probe.end) * 0.5,
    }
  }
}

pub(super) fn containing(world: &[Rect], point: Point) -> Option<usize> {
  world.iter().position(|rect| contains(*rect, point))
}

pub(super) fn largest_overlap(world: &[Rect], rect: Rect) -> Option<usize> {
  world
    .iter()
    .enumerate()
    .map(|(index, candidate)| (index, overlap(rect, *candidate)))
    .filter(|(_, area)| *area > 0.0)
    .max_by(|left, right| left.1.total_cmp(&right.1))
    .map(|(index, _)| index)
}
