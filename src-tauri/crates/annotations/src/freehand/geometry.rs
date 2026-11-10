// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A stroke's prepared record, and the distance that picks it.
//!
//! A stroke has as many curves as it bends, which no fixed record holds, so
//! its fitted chain rides in the side buffer, in source pixels from the
//! corner of its box, and the shader places each point itself. The prepared
//! record says how: `a` is where the box's top-left corner lands in the
//! pixels it is drawn in, `b` how far one source pixel reaches across and
//! down, and `c` where the box's bottom-right corner lands, which is what a
//! pixel far from the stroke is turned away by. `width` is the pen, and
//! `low` and `high` the stretch of the line showing.

use crate::geometry::ArrowGeometry;
use crate::reveal::AnnotationReveal;

/// The record for a stroke whose box runs from `low` to `high` in the
/// caller's pixels, and whose top-left corner moved one source pixel across
/// and down lands at `unit`.
pub fn prepare_freehand(
  low: [f32; 2],
  unit: [f32; 2],
  high: [f32; 2],
  width: f32,
  reveal: AnnotationReveal,
) -> ArrowGeometry {
  ArrowGeometry {
    a: low,
    b: [unit[0] - low[0], unit[1] - low[1]],
    c: high,
    width,
    low: reveal.low.clamp(0.0, 1.0),
    high: reveal.high.clamp(0.0, 1.0),
    ..ArrowGeometry::default()
  }
}

/// How far `point` falls outside the drawn stroke whose fitted chain is
/// `chain`, in the same pixels: zero on its edge and negative inside it.
pub fn freehand_distance(point: [f32; 2], chain: &[[f32; 2]], width: f32) -> f32 {
  super::path::chain_distance(point, chain) - width.max(0.0) * 0.5
}
