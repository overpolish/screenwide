// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::assignment::{contains, has_label_anchor};
use super::*;

/// Port of `layout` (`:624-652`): the loupe follows the pointer, flips to the
/// other side when it would leave the surface and is clamped inside the inset.
pub(crate) fn loupe_origin(point: Point, width: f64, height: f64, view: Size, inset: f64) -> Point {
  let mut left = point.x + inset;
  let mut top = point.y + inset;
  if left + width > view.width - inset {
    left = point.x - width - inset;
  }
  if top + height > view.height - inset {
    top = point.y - height - inset;
  }
  Point {
    x: left.clamp(inset, (view.width - width - inset).max(inset)),
    y: top.clamp(inset, (view.height - height - inset).max(inset)),
  }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn measurement_label_rect(
  ruler: &Ruler,
  measurement: MeasurementPacket,
  text: &str,
  cell: f64,
  value: ControlMetrics,
  control: f64,
  inset: f64,
  view: Size,
  offset: Point,
) -> Rect {
  let global = Rect::from_xywh(
    measurement.x,
    measurement.y,
    measurement.width,
    measurement.height,
  );
  let frame = ruler.project_world_rect(
    measurement.x,
    measurement.y,
    measurement.width,
    measurement.height,
    offset,
  );
  let width = value.padding_x * 2.0 + cell * text.chars().count() as f64;
  let height = value.height;
  let horizontal = global.size.height < inset;
  let vertical = global.size.width < inset;
  let mut left = frame.origin.x + frame.size.width * 0.5 - width * 0.5;
  let mut top = frame.origin.y + frame.size.height * 0.5 - height * 0.5;
  if has_label_anchor(measurement.label_anchor_x, measurement.label_anchor_y) {
    let anchor = ruler.project_point(
      Point {
        x: measurement.label_anchor_x,
        y: measurement.label_anchor_y,
      },
      offset,
    );
    left = anchor.x - width * 0.5;
    top = anchor.y - height * 0.5;
  } else if !horizontal
    && !vertical
    && (frame.size.width < width + inset * 2.0 || frame.size.height < height + inset * 2.0)
  {
    top = frame.origin.y - height - control;
  } else if horizontal {
    top = frame.bottom() + control;
  } else if vertical {
    left = frame.right() + control;
  }
  clamp_label(left, top, width, height, view, inset)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn probe_label_rect(
  ruler: &Ruler,
  probe: ProbePacket,
  radius: Option<RadiusPacket>,
  text: &str,
  cell: f64,
  value: ControlMetrics,
  control: f64,
  inset: f64,
  view: Size,
  offset: Point,
) -> Rect {
  let width = value.padding_x * 2.0 + cell * text.chars().count() as f64;
  let height = value.height;
  let (start, end, position) = ruler.project_probe(probe, offset);
  let mut left = if probe.axis == 1 {
    (start + end - width) * 0.5
  } else {
    position - width * 0.5
  };
  let mut top = if probe.axis == 1 {
    position - height * 0.5
  } else {
    (start + end - height) * 0.5
  };
  let anchored = has_label_anchor(probe.label_anchor_x, probe.label_anchor_y);
  if let (Some(radius), false) = (radius, anchored) {
    let right = radius.corner == 2 || radius.corner == 4;
    let bottom = radius.corner == 3 || radius.corner == 4;
    let corner = ruler.project_point(
      Point {
        x: radius.x + if right { radius.width } else { 0.0 },
        y: radius.y + if bottom { radius.height } else { 0.0 },
      },
      offset,
    );
    left = corner.x + if right { control } else { -width - control };
    top = corner.y + if bottom { control } else { -height - control };
  } else if anchored {
    let anchor = ruler.project_point(
      Point {
        x: probe.label_anchor_x,
        y: probe.label_anchor_y,
      },
      offset,
    );
    left = anchor.x - width * 0.5;
    top = anchor.y - height * 0.5;
  }
  clamp_label(left, top, width, height, view, inset)
}

pub(super) fn clamp_label(
  left: f64,
  top: f64,
  width: f64,
  height: f64,
  view: Size,
  inset: f64,
) -> Rect {
  Rect::from_xywh(
    left.clamp(inset, (view.width - width - inset).max(inset)),
    top.clamp(inset, (view.height - height - inset).max(inset)),
    width,
    height,
  )
}

/// Measurement, probe, guide gap then radius — the order the macOS hit test
/// walked its four label arrays in.
pub(crate) fn label_hit(rects: &[LabelRect], point: Point) -> Option<LabelHit> {
  for kind in 1..=4_u8 {
    if let Some(label) = rects
      .iter()
      .find(|label| label.kind == kind && label.id != 0 && contains(label.rect, point))
    {
      return Some(LabelHit {
        id: label.id,
        kind: label.kind,
        center: Point {
          x: label.rect.origin.x + label.rect.size.width * 0.5,
          y: label.rect.origin.y + label.rect.size.height * 0.5,
        },
      });
    }
  }
  None
}

pub(super) fn push_segment(
  segments: &mut Vec<Segment>,
  out: &[Vertex],
  start: usize,
  action_fills: [[f32; 4]; 2],
  radius: f64,
  label: Option<ID3D11ShaderResourceView>,
  secondary: Option<ID3D11ShaderResourceView>,
) {
  if out.len() <= start {
    return;
  }
  segments.push(Segment {
    start: start as u32,
    count: (out.len() - start) as u32,
    action_fills,
    chrome: [radius as f32, 0.0, 0.0, 0.0],
    chrome_outline: [0.0; 4],
    label,
    secondary,
  });
}
