// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A focus's retained draw record, and the strength its blur carries.
//!
//! `p0` and `p2` are the box's corners, placed like any point. `p1` is not
//! a point: it carries the corner radius as a percentage of the box's
//! shorter side, which no placement touches. `params[0]` carries the
//! strength of the blur, a step from 1 to 5, which the compositor's focus
//! pass reads with the reveal's presence. Nothing is drawn from the record
//! on the canvas: it exists so the editor chrome, the picking and the blur
//! pass share one placed box.

use crate::AnnotationPoint;
use crate::AnnotationStyle;

/// The three points a focus's record carries: the box's corners, and the
/// corner radius beside zero in the slot a spotlight keeps its softness.
pub(crate) fn draw_points(
  start: AnnotationPoint,
  end: AnnotationPoint,
  style: &AnnotationStyle,
) -> [[f32; 2]; 3] {
  [
    [start.x as f32, start.y as f32],
    [super::model::share(style.radius) as f32, 0.0],
    [end.x as f32, end.y as f32],
  ]
}

/// Writes the blur's strength into the record's parameters.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn fill(record: &mut super::super::native::NativeAnnotation, style: &AnnotationStyle) {
  record.params[0] = style.strength.clamp(0.0, 5.0) as f32;
}
