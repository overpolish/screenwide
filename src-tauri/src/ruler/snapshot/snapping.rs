// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::osc::geometry::Rect;

#[cfg(not(test))]
mod trace;

const CONTAINMENT_SLACK: f64 = 12.0;
const MINIMUM_IOU: f64 = 0.25;
const EDGE_SEARCH: f64 = 20.0;

fn finish(_boxes: &[Rect], drag: Rect, selected: Rect, _branch: &str) -> Rect {
  let result = clamp_to_drawn_bounds(selected, drag);
  #[cfg(not(test))]
  trace::record(_boxes, drag, selected, result, _branch);
  result
}

fn rect_area(rect: Rect) -> f64 {
  rect.size.width.max(0.0) * rect.size.height.max(0.0)
}

fn contains(outer: Rect, inner: Rect) -> bool {
  inner.origin.x >= outer.origin.x
    && inner.origin.y >= outer.origin.y
    && inner.right() <= outer.right()
    && inner.bottom() <= outer.bottom()
}

fn intersection_over(a: Rect, b: Rect) -> f64 {
  let width = (a.right().min(b.right()) - a.origin.x.max(b.origin.x)).max(0.0);
  let height = (a.bottom().min(b.bottom()) - a.origin.y.max(b.origin.y)).max(0.0);
  let overlap = width * height;
  let union = rect_area(a) + rect_area(b) - overlap;
  if union > 0.0 {
    overlap / union
  } else {
    0.0
  }
}

pub(super) fn snap_bounds(boxes: &[Rect], drag: Rect) -> Rect {
  let grown = Rect::from_xywh(
    drag.origin.x - CONTAINMENT_SLACK,
    drag.origin.y - CONTAINMENT_SLACK,
    drag.size.width + CONTAINMENT_SLACK * 2.0,
    drag.size.height + CONTAINMENT_SLACK * 2.0,
  );
  let contained = boxes
    .iter()
    .copied()
    // Slack may recover a clipped edge, but must not admit a separate object
    // entirely outside the user's selection.
    .filter(|candidate| contains(grown, *candidate) && intersection_over(drag, *candidate) > 0.0)
    .reduce(union_rect);
  if let Some(bounds) = contained {
    return finish(boxes, drag, bounds, "contained-union");
  }
  if let Some((_, bounds)) = boxes
    .iter()
    .copied()
    .map(|candidate| (intersection_over(candidate, drag), candidate))
    .filter(|(score, _)| *score >= MINIMUM_IOU)
    .max_by(|(left, _), (right, _)| left.total_cmp(right))
  {
    return finish(boxes, drag, bounds, "overlap");
  }
  finish(
    boxes,
    drag,
    snap_to_nearby_box_edges(boxes, drag),
    "nearby-edges",
  )
}

/// The user's drag is a hard analysis boundary. Snapping may tighten a
/// measurement around detected content inside it, but content beside or around
/// the drag must never make the committed measurement larger than what the
/// user drew.
fn clamp_to_drawn_bounds(bounds: Rect, drag: Rect) -> Rect {
  let left = bounds.origin.x.max(drag.origin.x);
  let top = bounds.origin.y.max(drag.origin.y);
  let right = bounds.right().min(drag.right());
  let bottom = bounds.bottom().min(drag.bottom());
  if right <= left || bottom <= top {
    drag
  } else {
    Rect::from_xywh(left, top, right - left, bottom - top)
  }
}

fn union_rect(a: Rect, b: Rect) -> Rect {
  let left = a.origin.x.min(b.origin.x);
  let top = a.origin.y.min(b.origin.y);
  Rect::from_xywh(
    left,
    top,
    a.right().max(b.right()) - left,
    a.bottom().max(b.bottom()) - top,
  )
}

fn snap_to_nearby_box_edges(boxes: &[Rect], drag: Rect) -> Rect {
  let nearest = |target: f64, candidates: Vec<f64>| {
    candidates
      .into_iter()
      .filter(|edge| (edge - target).abs() <= EDGE_SEARCH)
      .min_by(|left, right| (left - target).abs().total_cmp(&(right - target).abs()))
      .unwrap_or(target)
  };
  let x_edges = boxes
    .iter()
    .filter(|item| item.bottom() >= drag.origin.y && item.origin.y <= drag.bottom())
    .flat_map(|item| [item.origin.x, item.right()])
    .collect::<Vec<_>>();
  let x0 = nearest(drag.origin.x, x_edges.clone());
  let x1 = nearest(drag.right(), x_edges);
  let y_edges = boxes
    .iter()
    .filter(|item| item.right() >= x0.min(x1) && item.origin.x <= x0.max(x1))
    .flat_map(|item| [item.origin.y, item.bottom()])
    .collect::<Vec<_>>();
  let y0 = nearest(drag.origin.y, y_edges.clone());
  let y1 = nearest(drag.bottom(), y_edges);
  Rect::from_xywh(x0.min(x1), y0.min(y1), (x1 - x0).abs(), (y1 - y0).abs())
}

#[cfg(test)]
mod tests;
