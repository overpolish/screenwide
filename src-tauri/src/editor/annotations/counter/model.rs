// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Making a counter, and the dress a fresh one wears.
//!
//! The disc is what carries the number, so its diameter is the annotation's
//! `style.width` - a counter has no stroke of its own to weigh. The tail is
//! the part that points: it is always there, and only its direction is
//! editable, so a counter can be aimed without being reshaped. The shape it
//! is drawn and picked as lives in [`super::silhouette`].

use crate::editor::annotations::{
  Annotation, AnnotationHead, AnnotationPoint, AnnotationShape, AnnotationStyle,
};

/// The disc diameters the counter control offers, in output pixels. Five
/// steps, keeping the three earlier sizes at the first, middle and last step
/// so an older document keeps its discs. The twin of
/// `ANNOTATION_COUNTER_SIZES` in
/// `src/components/shared/annotation-style/widths.ts`.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) const COUNTER_SIZES: [f64; 5] = [56.0, 72.0, 96.0, 128.0, 160.0];

/// The disc a fresh counter is drawn at, in output pixels: the smallest,
/// which is the size counters were drawn at before the sizes widened.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) const NEW_COUNTER_WIDTH: f64 = COUNTER_SIZES[0];

/// A fresh counter points its tail right, so a row of them is dropped with
/// the same aim and only the ones that need turning are turned.
pub(crate) const NEW_COUNTER_ANGLE: f64 = 0.0;

/// The dress a fresh counter is drawn in before anything has been chosen:
/// the same first colour an arrow takes, at the middle disc.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn default_counter_style() -> AnnotationStyle {
  AnnotationStyle {
    align: Default::default(),
    color: crate::editor::annotations::model::NEW_ANNOTATION_COLOR.to_owned(),
    head: AnnotationHead::None,
    width: NEW_COUNTER_WIDTH,
  }
}

/// A counter numbered `value` at `center`, in `style` or in the tool's own
/// first dress where the editor has not settled on one yet, aimed at `angle`
/// or east where no counter has been turned yet.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn new_counter(
  id: String,
  center: AnnotationPoint,
  value: u32,
  style: Option<&AnnotationStyle>,
  angle: Option<f64>,
) -> Annotation {
  Annotation {
    above_camera: false,
    animated: true,
    id,
    reveal: crate::editor::annotations::reveal::AnnotationReveal::default(),
    shape: AnnotationShape::Counter {
      center,
      value,
      angle: angle
        .filter(|angle| angle.is_finite())
        .unwrap_or(NEW_COUNTER_ANGLE),
    },
    style: style.cloned().unwrap_or_else(default_counter_style),
  }
}

/// The number the next counter dropped on this list takes. The editor keeps
/// the numbers contiguous, so the count is the highest number there is. The
/// list is taken as an iterator because the live overlay holds its
/// annotations inside a wrapper rather than as a slice of its own.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn next_counter_value<'a>(annotations: impl IntoIterator<Item = &'a Annotation>) -> u32 {
  annotations
    .into_iter()
    .filter(|annotation| matches!(annotation.shape, AnnotationShape::Counter { .. }))
    .count() as u32
    + 1
}

/// Whether a counter is somewhere it can be drawn. A document read from disk
/// carries whatever it was written with, so every space-changing path tests
/// this before it works with one.
pub(crate) fn placed(center: AnnotationPoint, angle: f64) -> bool {
  center.x.is_finite() && center.y.is_finite() && angle.is_finite()
}

/// How far apart the angles Shift snaps to are: eight of them, the quarter
/// turns and the diagonals between. The twin of `ANNOTATION_COUNTER_SNAP` in
/// `src/features/editor/annotations.ts`, which the angle control steps by.
pub(crate) const COUNTER_SNAP_RADIANS: f64 = std::f64::consts::FRAC_PI_4;

/// The angle a tail tip dragged to `point` aims the counter at. A press
/// exactly on the centre keeps the aim it had: there is no direction in a
/// zero-length vector to take one from.
///
/// `snap` holds the aim to the nearest eighth of a turn - measured from east
/// rather than from wherever the drag began, so the same eight angles are
/// always the ones on offer however the tail was taken hold of.
pub(crate) fn counter_tail_angle(
  center: AnnotationPoint,
  point: AnnotationPoint,
  current: f64,
  snap: bool,
) -> f64 {
  let dx = point.x - center.x;
  let dy = point.y - center.y;
  if dx == 0.0 && dy == 0.0 {
    return current;
  }
  let angle = dy.atan2(dx);
  if !snap {
    return angle;
  }
  (angle / COUNTER_SNAP_RADIANS).round() * COUNTER_SNAP_RADIANS
}

#[cfg(test)]
mod tests {
  use super::*;

  fn center() -> AnnotationPoint {
    AnnotationPoint { x: 100.0, y: 100.0 }
  }

  #[test]
  fn a_dragged_tail_takes_the_angle_of_the_drag() {
    let at = |x: f64, y: f64, snap: bool| {
      counter_tail_angle(center(), AnnotationPoint { x, y }, 0.0, snap)
    };
    assert!((at(100.0, 140.0, false) - std::f64::consts::FRAC_PI_2).abs() < 1e-9);
    // A drag onto the centre itself keeps the aim: there is no direction in
    // it to take one from.
    assert_eq!(counter_tail_angle(center(), center(), 1.25, false), 1.25);
  }

  #[test]
  fn a_snapped_tail_lands_on_the_absolute_eighths() {
    let at = |x: f64, y: f64| counter_tail_angle(center(), AnnotationPoint { x, y }, 0.0, true);
    // Just off east and just off the diagonals: each lands on the nearest
    // eighth of a turn, measured from east rather than from the drag.
    assert_eq!(at(180.0, 104.0), 0.0);
    assert!((at(150.0, 140.0) - COUNTER_SNAP_RADIANS).abs() < 1e-9);
    assert!((at(100.0, 150.0) - std::f64::consts::FRAC_PI_2).abs() < 1e-9);
    assert!((at(60.0, 145.0) - 3.0 * COUNTER_SNAP_RADIANS).abs() < 1e-9);
    // And above the disc it snaps to the turns on the other side.
    assert!((at(150.0, 60.0) + COUNTER_SNAP_RADIANS).abs() < 1e-9);
  }

  #[test]
  fn numbering_follows_the_counters_already_there() {
    let counter = |value: u32| {
      new_counter(
        format!("c{value}"),
        AnnotationPoint { x: 0.0, y: 0.0 },
        value,
        None,
        None,
      )
    };
    assert_eq!(next_counter_value(&[]), 1);
    assert_eq!(next_counter_value(&[counter(1), counter(2)]), 3);
  }
}
