// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Making a redaction, and the dress a fresh one wears.
//!
//! The box is held by two corners in source pixels, `start` the top-left and
//! `end` the bottom-right; every edit writes them back in that order. The
//! style's width is the pixelation's block size in output pixels, so blocks
//! keep their weight on the canvas the way a stroke does.

#[cfg(any(target_os = "macos", target_os = "windows", test))]
use crate::editor::annotations::snap::SnapBox;
use crate::editor::annotations::AnnotationPoint;
#[cfg(any(target_os = "macos", target_os = "windows", test))]
use crate::editor::annotations::{
  Annotation, AnnotationHead, AnnotationRedaction, AnnotationShape, AnnotationStyle,
};

/// The block sizes pixelation offers, in output pixels. The twin of
/// `ANNOTATION_REDACT_SIZES` in
/// `src/components/shared/annotation-style/widths.ts`.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) const REDACT_BLOCK_SIZES: [f64; 5] = [8.0, 12.0, 16.0, 24.0, 32.0];

/// The block a fresh redaction pixelates with, in output pixels.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) const NEW_REDACT_WIDTH: f64 = REDACT_BLOCK_SIZES[1];

/// The strength a fresh blur takes: the middle of its five steps.
pub(crate) const NEW_BLUR_STRENGTH: f64 = 3.0;

/// The colour mode's first colour: black, the conventional redaction bar.
/// A redaction keeps a dress of its own rather than the colour the arrows and
/// counters share, which would fill a box in whatever an arrow was last drawn
/// in.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
const NEW_REDACT_COLOR: &str = "#000000";

#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn default_redact_style() -> AnnotationStyle {
  AnnotationStyle {
    align: Default::default(),
    color: NEW_REDACT_COLOR.to_owned(),
    head: AnnotationHead::None,
    radius: 0.0,
    redaction: AnnotationRedaction::Erase,
    strength: NEW_BLUR_STRENGTH,
    width: NEW_REDACT_WIDTH,
  }
}

/// A redaction with both corners at `point`, which the drag that follows
/// pulls out. It starts without animating: a box that ramped in would show
/// what it hides for the frames before it arrived, so that is left to be
/// chosen.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn new_redact(
  id: String,
  point: AnnotationPoint,
  style: Option<&AnnotationStyle>,
) -> Annotation {
  Annotation {
    above_camera: false,
    animated: false,
    id,
    held: None,
    reveal: crate::editor::annotations::reveal::AnnotationReveal::default(),
    shape: AnnotationShape::Redact {
      start: point,
      end: point,
      seed: fresh_seed(),
    },
    style: style.cloned().unwrap_or_else(default_redact_style),
  }
}

/// A seed for the pixelation pattern, from the operating system's secure
/// generator. The blocks never depend on the covered pixels, so hiding them
/// does not rest on the seed; drawing it from the secure generator keeps one
/// box's blocks unrelated to any other's, including across documents.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
fn fresh_seed() -> u32 {
  // The generator only fails where the platform has none at all. Any fixed
  // seed still yields blocks that carry nothing of the picture.
  getrandom::u32().unwrap_or(0x9e37_79b9)
}

/// Whether a redaction is somewhere it can be drawn.
pub(crate) fn placed(start: AnnotationPoint, end: AnnotationPoint) -> bool {
  start.x.is_finite() && start.y.is_finite() && end.x.is_finite() && end.y.is_finite()
}

/// The box between two corners, whichever way round they are given.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn redact_box(start: AnnotationPoint, end: AnnotationPoint) -> SnapBox {
  SnapBox {
    x: start.x.min(end.x),
    y: start.y.min(end.y),
    width: (end.x - start.x).abs(),
    height: (end.y - start.y).abs(),
  }
}

/// The corners of `bounds`, top-left first.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn corners(bounds: SnapBox) -> (AnnotationPoint, AnnotationPoint) {
  (
    AnnotationPoint {
      x: bounds.x,
      y: bounds.y,
    },
    AnnotationPoint {
      x: bounds.x + bounds.width,
      y: bounds.y + bounds.height,
    },
  )
}
