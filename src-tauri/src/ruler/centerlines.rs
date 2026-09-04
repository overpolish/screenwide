// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::osc::geometry::{Point, Rect};

const SELF_SLACK: f64 = 3.0;
const MINIMUM_OBJECT_SIZE: f64 = 3.0;
const CLUSTER_GAP: f64 = 6.0;
const MAXIMUM_PARTS: usize = 128;
const MAXIMUM_OBJECTS: usize = 12;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct InnerObject {
  pub bounds: Rect,
  pub aligned_x: bool,
  pub aligned_y: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CenterlineAnalysis {
  pub objects: Vec<InnerObject>,
  pub x_accent: bool,
  pub y_accent: bool,
}

fn center(bounds: Rect) -> Point {
  Point {
    x: bounds.origin.x + bounds.size.width * 0.5,
    y: bounds.origin.y + bounds.size.height * 0.5,
  }
}

fn shares_rendered_center(left: f64, right: f64, scale: f64) -> bool {
  (left * scale).round() == (right * scale).round()
}

fn contains(outer: Rect, inner: Rect) -> bool {
  inner.origin.x >= outer.origin.x
    && inner.origin.y >= outer.origin.y
    && inner.right() <= outer.right()
    && inner.bottom() <= outer.bottom()
}

fn is_self(outer: Rect, inner: Rect, slack: f64) -> bool {
  (outer.origin.x - inner.origin.x).abs() <= slack
    && (outer.origin.y - inner.origin.y).abs() <= slack
    && (outer.right() - inner.right()).abs() <= slack
    && (outer.bottom() - inner.bottom()).abs() <= slack
}

fn near(left: Rect, right: Rect, gap: f64) -> bool {
  left.origin.x - gap <= right.right()
    && right.origin.x - gap <= left.right()
    && left.origin.y - gap <= right.bottom()
    && right.origin.y - gap <= left.bottom()
}

fn union(left: Rect, right: Rect) -> Rect {
  let x = left.origin.x.min(right.origin.x);
  let y = left.origin.y.min(right.origin.y);
  Rect::from_xywh(
    x,
    y,
    left.right().max(right.right()) - x,
    left.bottom().max(right.bottom()) - y,
  )
}

fn clustered(mut parts: Vec<Rect>, gap: f64) -> Vec<Rect> {
  let mut changed = true;
  while changed {
    changed = false;
    'outer: for left in 0..parts.len() {
      for right in left + 1..parts.len() {
        if near(parts[left], parts[right], gap) {
          parts[left] = union(parts[left], parts[right]);
          parts.remove(right);
          changed = true;
          break 'outer;
        }
      }
    }
  }
  parts
}

pub(crate) fn analyze(
  bounds: Rect,
  boxes: &[Rect],
  peers: &[Rect],
  device_scale: f64,
) -> CenterlineAnalysis {
  let scale = device_scale.max(f64::EPSILON);
  let self_slack = SELF_SLACK / scale;
  let minimum = MINIMUM_OBJECT_SIZE / scale;
  let mut parts = boxes
    .iter()
    .copied()
    .filter(|candidate| {
      contains(bounds, *candidate)
        && !is_self(bounds, *candidate, self_slack)
        && candidate.size.width >= minimum
        && candidate.size.height >= minimum
    })
    .collect::<Vec<_>>();
  parts.sort_by(|left, right| {
    let left_area = left.size.width * left.size.height;
    let right_area = right.size.width * right.size.height;
    right_area.total_cmp(&left_area)
  });
  parts.truncate(MAXIMUM_PARTS);
  let mut objects = clustered(parts, CLUSTER_GAP / scale)
    .into_iter()
    .filter(|candidate| !is_self(bounds, *candidate, self_slack))
    .collect::<Vec<_>>();
  objects.sort_by(|left, right| {
    let left_area = left.size.width * left.size.height;
    let right_area = right.size.width * right.size.height;
    right_area.total_cmp(&left_area)
  });
  objects.truncate(MAXIMUM_OBJECTS);

  let outer_center = center(bounds);
  let union = objects.iter().copied().reduce(union);
  let x_accent = peers
    .iter()
    .any(|peer| shares_rendered_center(center(*peer).x, outer_center.x, scale))
    || union.is_some_and(|inner| shares_rendered_center(center(inner).x, outer_center.x, scale));
  let y_accent = peers
    .iter()
    .any(|peer| shares_rendered_center(center(*peer).y, outer_center.y, scale))
    || union.is_some_and(|inner| shares_rendered_center(center(inner).y, outer_center.y, scale));
  CenterlineAnalysis {
    objects: objects
      .into_iter()
      .map(|inner| {
        let inner_center = center(inner);
        InnerObject {
          bounds: inner,
          aligned_x: shares_rendered_center(inner_center.x, outer_center.x, scale),
          aligned_y: shares_rendered_center(inner_center.y, outer_center.y, scale),
        }
      })
      .collect(),
    x_accent,
    y_accent,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn clusters_inner_parts_and_reports_content_alignment() {
    let bounds = Rect::from_xywh(10.0, 10.0, 80.0, 60.0);
    let result = analyze(
      bounds,
      &[
        bounds,
        Rect::from_xywh(35.0, 25.0, 12.0, 30.0),
        Rect::from_xywh(52.0, 25.0, 13.0, 30.0),
      ],
      &[],
      1.0,
    );
    assert_eq!(result.objects.len(), 1);
    assert!(result.x_accent);
    assert!(result.y_accent);
    assert!(result.objects[0].aligned_x);
    assert!(result.objects[0].aligned_y);
  }

  #[test]
  fn sibling_centres_accent_only_the_shared_axis() {
    let bounds = Rect::from_xywh(10.0, 10.0, 40.0, 40.0);
    let result = analyze(bounds, &[], &[Rect::from_xywh(10.0, 80.0, 40.0, 20.0)], 2.0);
    assert!(result.x_accent);
    assert!(!result.y_accent);
  }

  #[test]
  fn a_one_device_pixel_offset_is_not_reported_as_centered() {
    let bounds = Rect::from_xywh(10.0, 10.0, 40.0, 40.0);
    let result = analyze(bounds, &[Rect::from_xywh(11.0, 15.0, 39.0, 30.0)], &[], 2.0);
    assert!(!result.x_accent);
    assert!(result.y_accent);
    assert!(!result.objects[0].aligned_x);
    assert!(result.objects[0].aligned_y);
  }
}
