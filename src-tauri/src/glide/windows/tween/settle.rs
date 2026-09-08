// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// Reads the settled frame, corrects its origin if the window came out a
/// different size than asked for, and reports the result to the preview. The
/// generation is re-checked around every window call: a retarget that landed
/// while the frame was being read owns the window now.
pub(super) fn settle(
  target: &WindowTarget,
  requested: GlideFrame,
  generation: u64,
  context: &FitContext,
) {
  let Ok(achieved) = target.frame() else {
    return;
  };
  if !is_current(generation) {
    return;
  }
  let fits = frame_fits(achieved, requested, FIT_EPSILON);
  let mut frame = achieved;
  if !fits {
    let (x, y) = corrected_origin(requested, achieved, context.gravity);
    frame.x = x;
    frame.y = y;
    if target.set_origin(frame).is_err() || !is_current(generation) {
      return;
    }
  }
  let Some(actual) = frame_fractions(
    frame,
    (context.work.x, context.work.y),
    (context.work.width, context.work.height),
  ) else {
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
