// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What happens once a move has settled: the frame the window actually took
//! is read back, a window its application refused to resize is pulled onto
//! the edge its region was aimed at, and the result is reported to the
//! preview.

use tauri::AppHandle;

use crate::glide::{
  core::{corrected_origin, frame_fits, frame_fractions, tween::GlideWindow, GlideFrame},
  fit::{emit_fit, FitRect, GlideFitEvent},
  region_rect::RegionGravity,
};

/// How far the achieved size may miss the destination and still count as a
/// fill. Applications round their size to a character cell or a device pixel,
/// and a point or two of that is not a constraint worth showing.
const FIT_EPSILON: f64 = 2.0;

/// Everything the settling step needs to place a constrained window and
/// report it. A move with no region behind it - an Esc restore, a double-tap
/// centering - carries none of this and settles exactly where it was sent.
pub(crate) struct FitContext {
  pub(crate) app: AppHandle,
  pub(crate) session_id: u64,
  pub(crate) gravity: RegionGravity,
  /// The work area the region was laid out in, which the preview draws in.
  pub(crate) work: GlideFrame,
}

/// Reads the settled frame, corrects its origin if the window came out a
/// different size than asked for, and reports the result to the preview. The
/// judgement is always against `requested` - the rectangle the placement
/// aimed at - even when the tween itself was reduced to a move because the
/// window refuses to be resized: the preview's outline is only truthful if it
/// shows the size the window really has inside the region it was asked to
/// fill. `is_current` is asked again after every window call: a retarget that
/// landed meanwhile owns the window now, and a superseded settle must neither
/// move it nor describe it.
pub(crate) fn settle(
  target: &mut dyn GlideWindow,
  requested: GlideFrame,
  is_current: impl Fn() -> bool,
  context: &FitContext,
) {
  let Some(achieved) = target.frame() else {
    return;
  };
  if !is_current() {
    return;
  }
  let fits = fills(achieved, requested);
  let mut frame = achieved;
  if !fits {
    (frame.x, frame.y) = corrected_origin(requested, achieved, context.gravity);
    if !target.move_to(frame) || !is_current() {
      return;
    }
  }
  let work = context.work;
  let Some(actual) = frame_fractions(frame, (work.x, work.y), (work.width, work.height)) else {
    return;
  };
  if let Err(error) = emit_fit(
    &context.app,
    GlideFitEvent {
      session_id: context.session_id,
      fits,
      actual: FitRect {
        x: actual.x,
        y: actual.y,
        width: actual.width,
        height: actual.height,
      },
    },
  ) {
    eprintln!("Could not report the Glide placement: {error}");
  }
}

fn fills(achieved: GlideFrame, requested: GlideFrame) -> bool {
  frame_fits(achieved, requested, FIT_EPSILON)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn size(width: f64, height: f64) -> GlideFrame {
    GlideFrame {
      x: 0.0,
      y: 0.0,
      width,
      height,
    }
  }

  #[test]
  fn a_rounding_sized_window_still_counts_as_a_fill() {
    assert!(fills(size(498.5, 801.0), size(500.0, 800.0)));
    assert!(!fills(size(480.0, 800.0), size(500.0, 800.0)));
  }
}
