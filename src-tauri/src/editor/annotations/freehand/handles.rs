// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A stroke's grips, as the native chrome needs them.

use super::model::bounds;
use crate::editor::annotations::handles::{
  normalised_point, stroke_width, NativeAnnotationHandles,
};
use crate::editor::annotations::{AnnotationKind, AnnotationPoint, AnnotationStyle};

/// A stroke reads the slots as a box does: `start` is its box's top-left
/// corner and `end` its bottom-right, normalised over the source, and
/// `middle` its centre, from which the native side places the eight grips of
/// the layer selection's box. Its fitted chain is appended to `paths`,
/// normalised the same way, and the record says where: `start_head` is the
/// chain's first point in `paths` and `end_head` how many it has, each a whole
/// number. `width` is its pen as a share of the drawn width, so the chrome
/// picks the line as it is drawn.
pub(crate) fn grips(
  points: &[AnnotationPoint],
  smooth: bool,
  style: &AnnotationStyle,
  index: u32,
  source: (u32, u32),
  image_width: f64,
  paths: &mut Vec<[f32; 2]>,
) -> NativeAnnotationHandles {
  let (low, high) = bounds(points);
  let (start_x, start_y) = normalised_point(low, source);
  let (end_x, end_y) = normalised_point(high, source);
  let chain = super::path::fitted(points, smooth);
  let first = paths.len();
  paths.extend(chain.iter().map(|point| {
    let (x, y) = normalised_point(
      AnnotationPoint {
        x: f64::from(point[0]),
        y: f64::from(point[1]),
      },
      source,
    );
    [x as f32, y as f32]
  }));
  NativeAnnotationHandles {
    layer_id: -1,
    index,
    kind: AnnotationKind::Draw.raw(),
    padding: 0,
    start_x,
    start_y,
    middle_x: (start_x + end_x) / 2.0,
    middle_y: (start_y + end_y) / 2.0,
    end_x,
    end_y,
    start_head: first as f64,
    end_head: chain.len() as f64,
    width: stroke_width(style, image_width),
  }
}
