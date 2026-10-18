// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// Port of `hovered_artifact_key` (`:851-896`): hover opacity is smuggled in
/// `padding[0]` as 0-255, and the key is `(id << 3) | kindTag`.
pub(crate) fn hovered_artifact_key(data: &RulerData) -> (u64, f64) {
  for item in &data.measurements {
    if item.padding[0] != 0 {
      return ((item.id << 3) | 1, f64::from(item.padding[0]) / 255.0);
    }
  }
  for item in &data.probes {
    if item.padding[0] != 0 {
      return ((item.id << 3) | 2, f64::from(item.padding[0]) / 255.0);
    }
  }
  for item in &data.guides {
    if item.padding[0] != 0 {
      return ((item.id << 3) | 3, f64::from(item.padding[0]) / 255.0);
    }
  }
  for item in &data.guide_gaps {
    if item.padding[0] != 0 {
      return ((item.id << 3) | 4, f64::from(item.padding[0]) / 255.0);
    }
  }
  for item in &data.radii {
    if item.padding[0] != 0 {
      return ((item.id << 3) | 5, f64::from(item.padding[0]) / 255.0);
    }
  }
  (0, 0.0)
}

/// True while any measurement is mid-animation or the result asked for one
/// (`ruler_flags` bit 7). Drives the settle frame (phase 15).
pub(crate) fn animation_active(data: &RulerData, ruler_flags: u8) -> bool {
  data.measurements.iter().any(|item| item.flags & 2 != 0) || ruler_flags & 128 != 0
}

pub(super) fn axis_point(axis: u8, along: f64, across: f64) -> Point {
  if axis == 1 {
    Point {
      x: along,
      y: across,
    }
  } else {
    Point {
      x: across,
      y: along,
    }
  }
}

pub(crate) fn project_world_rect(
  x: f64,
  y: f64,
  width: f64,
  height: f64,
  offset: Point,
  origin: Point,
  zoom: f64,
) -> Rect {
  Rect::from_xywh(
    (x - offset.x - origin.x) * zoom,
    (y - offset.y - origin.y) * zoom,
    width * zoom,
    height * zoom,
  )
}

/// Port of `add_center_object_outline` (`:523-554`): a one-pixel rectangle at
/// kind 44, built from four snapped quads.
pub(super) fn add_center_object_outline(
  out: &mut Vec<Vertex>,
  view: Size,
  frame: Rect,
  scale: f64,
) {
  let min_x = renderer::snap(frame.origin.x, scale);
  let max_x = renderer::snap(frame.right(), scale);
  let min_y = renderer::snap(frame.origin.y, scale);
  let max_y = renderer::snap(frame.bottom(), scale);
  let half = 0.5 / scale;
  renderer::add_quad(
    out,
    view,
    Rect::from_xywh(
      min_x - half,
      min_y - half,
      max_x - min_x + half * 2.0,
      half * 2.0,
    ),
    44,
  );
  renderer::add_quad(
    out,
    view,
    Rect::from_xywh(
      min_x - half,
      max_y - half,
      max_x - min_x + half * 2.0,
      half * 2.0,
    ),
    44,
  );
  let vertical = (max_y - min_y - half * 2.0).max(0.0);
  if vertical > 0.0 {
    renderer::add_quad(
      out,
      view,
      Rect::from_xywh(min_x - half, min_y + half, half * 2.0, vertical),
      44,
    );
    renderer::add_quad(
      out,
      view,
      Rect::from_xywh(max_x - half, min_y + half, half * 2.0, vertical),
      44,
    );
  }
}

pub(crate) fn radius_center(radius: RadiusPacket) -> Point {
  let right = radius.corner == 2 || radius.corner == 4;
  let bottom = radius.corner == 3 || radius.corner == 4;
  let corner_x = radius.x + if right { radius.width } else { 0.0 };
  let corner_y = radius.y + if bottom { radius.height } else { 0.0 };
  Point {
    x: corner_x + if right { -radius.radius } else { radius.radius },
    y: corner_y
      + if bottom {
        -radius.radius
      } else {
        radius.radius
      },
  }
}

pub(super) fn radius_arc_midpoint(radius: RadiusPacket) -> Point {
  let right = radius.corner == 2 || radius.corner == 4;
  let bottom = radius.corner == 3 || radius.corner == 4;
  let center = radius_center(radius);
  let diagonal = std::f64::consts::FRAC_1_SQRT_2 * radius.radius;
  Point {
    x: center.x + if right { diagonal } else { -diagonal },
    y: center.y + if bottom { diagonal } else { -diagonal },
  }
}

pub(crate) fn guide_gap_probe(gap: GuideGapPacket) -> ProbePacket {
  ProbePacket {
    id: gap.id,
    display_id: gap.display_id,
    axis: gap.axis,
    flags: if gap.flags & 2 != 0 { 8 } else { 0 },
    padding: [0; 2],
    start: gap.start,
    end: gap.end,
    position: gap.position,
    label_anchor_x: gap.label_anchor_x,
    label_anchor_y: gap.label_anchor_y,
  }
}

pub(crate) fn radius_label_probe(radius: RadiusPacket) -> ProbePacket {
  let midpoint = radius_arc_midpoint(radius);
  ProbePacket {
    id: radius.id,
    display_id: radius.display_id,
    axis: 1,
    flags: if radius.flags & 8 != 0 { 8 } else { 0 },
    padding: [0; 2],
    start: midpoint.x - radius.radius * 0.5,
    end: midpoint.x + radius.radius * 0.5,
    position: midpoint.y,
    label_anchor_x: radius.label_anchor_x,
    label_anchor_y: radius.label_anchor_y,
  }
}
