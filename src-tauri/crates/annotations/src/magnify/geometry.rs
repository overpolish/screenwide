// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A lens's prepared record, and the distance that picks it.
//!
//! The record carries, in the pixels it was prepared in: the lens's centre in
//! `a`, its half size in `b` and its corner radius in `rounding`, as this
//! frame draws it; how many times it enlarges in `high`, as this frame draws
//! it; the lens's width, which its hover halo reaches by, in `width`; and how
//! present it is in `low`, which its shadow and its glass come with.
//!
//! As a clip arrives, the lens settles onto the picture: it starts smaller
//! and at no zoom at all, showing exactly what it covers, and grows to its
//! size as its zoom rises to its own, so the enlargement arrives with it and
//! the lens lifts off the picture rather than fading in over it.
//! `reveal.scale` is how far it has come.

use crate::geometry::{ArrowGeometry, ArrowTriangle};
use crate::reveal::AnnotationReveal;
use crate::text::geometry::rounded_box_distance;

/// How big a lens sets off, as a share of its size.
const SETTING_OFF: f32 = 0.6;

/// The lens over the box from `start` to `end`, in the space both are given
/// in, enlarging `zoom` times, rounded by `radius` percent of its shorter
/// side, `width` wide, at `reveal`.
pub fn prepare_magnify(
  start: [f32; 2],
  end: [f32; 2],
  zoom: f32,
  radius: f32,
  width: f32,
  reveal: AnnotationReveal,
) -> ArrowGeometry {
  let travel = if reveal.scale.is_finite() {
    reveal.scale.clamp(0.0, 1.0)
  } else {
    1.0
  };
  let grown = SETTING_OFF + (1.0 - SETTING_OFF) * travel;
  let half = [
    (end[0] - start[0]).abs() * 0.5 * grown,
    (end[1] - start[1]).abs() * 0.5 * grown,
  ];
  let zoom = if zoom.is_finite() && zoom > 0.0 {
    zoom
  } else {
    1.0
  };
  let share = if radius.is_finite() {
    radius.clamp(0.0, 50.0) / 100.0
  } else {
    0.0
  };
  ArrowGeometry {
    a: [(start[0] + end[0]) * 0.5, (start[1] + end[1]) * 0.5],
    b: half,
    c: [0.0, 0.0],
    width: width.max(0.0),
    low: reveal.opacity.clamp(0.0, 1.0),
    high: 1.0 + (zoom - 1.0) * travel,
    start_head: ArrowTriangle::default(),
    end_head: ArrowTriangle::default(),
    rounding: 2.0 * half[0].min(half[1]).max(0.0) * share,
    head: 0,
  }
}

/// How far `point` falls outside the lens: zero or less anywhere on it, so a
/// press anywhere on the lens picks it.
pub fn magnify_distance(point: [f32; 2], geometry: &ArrowGeometry) -> f32 {
  let [x, y] = geometry.a;
  let [w, h] = geometry.b;
  rounded_box_distance(point, [x - w, y - h], [x + w, y + h], geometry.rounding)
}
