// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What happens once a move has settled: the frame the window actually took is
//! read back, a window its application refused to resize is pulled onto the
//! edge its region was aimed at, and the result is reported to the preview.

use cidre::cg;
use tauri::AppHandle;

use super::{is_current, WindowTarget};
use crate::glide::fit::{emit_fit, FitRect, GlideFitEvent};
#[cfg(test)]
use crate::glide::region_rect::Gravity;
use crate::glide::region_rect::RegionGravity;

/// How far the achieved size may miss the destination and still count as a
/// fill. Applications round their size to a character cell or a device pixel,
/// and a point or two of that is not a constraint worth showing.
const FIT_EPSILON: f64 = 2.0;

/// Everything the settling step needs to place a constrained window and report
/// it. A move with no region behind it - an Esc restore, a double-tap centering
/// - carries none of this and settles exactly as it always did.
pub(in crate::glide) struct FitContext {
  pub(in crate::glide) app: AppHandle,
  pub(in crate::glide) session_id: u64,
  pub(in crate::glide) gravity: RegionGravity,
  pub(in crate::glide) work_origin: (f64, f64),
  pub(in crate::glide) work_size: (f64, f64),
}

/// Reads the settled frame, corrects its origin if the window came out a
/// different size than asked for, and reports the result to the preview. The
/// judgement is always against `requested` - the rectangle the placement aimed
/// at - even when the tween itself was reduced to a move because the window
/// refuses to be resized: the preview's outline is only truthful if it shows
/// the size the window really has inside the region it was asked to fill. The
/// generation is re-checked around every Accessibility call: a retarget that
/// landed while the frame was being read owns the window now, and a superseded
/// settle must neither move it nor describe it.
pub(super) fn settle(
  target: &mut WindowTarget,
  requested: cg::Rect,
  generation: u64,
  context: &FitContext,
) {
  let Some(achieved) = target.frame() else {
    return;
  };
  if !is_current(generation) {
    return;
  }

  let fits = !deviates(achieved.size, requested.size);
  let mut frame = achieved;
  if !fits {
    frame.origin = corrected_origin(requested, achieved.size, context.gravity);
    if !target.set_pos(&frame.origin) || !is_current(generation) {
      return;
    }
  }

  let Some(actual) = fractions(frame, context.work_origin, context.work_size) else {
    return;
  };
  if let Err(error) = emit_fit(
    &context.app,
    GlideFitEvent {
      session_id: context.session_id,
      fits,
      actual,
    },
  ) {
    eprintln!("Could not report the Glide placement: {error}");
  }
}

fn deviates(achieved: cg::Size, destination: cg::Size) -> bool {
  !crate::glide::core::frame_fits(size_frame(achieved), size_frame(destination), FIT_EPSILON)
}

/// Where a window of `achieved` size belongs inside the destination rectangle.
/// A window larger than the region on an axis overhangs it the same way it
/// would underfill it, so an `End` pull still lines up its far edge. The tween
/// reuses this to aim a fixed-size window's move-only travel, which is why the
/// settle that follows finds the origin already where it wants it.
pub(super) fn corrected_origin(
  destination: cg::Rect,
  achieved: cg::Size,
  gravity: RegionGravity,
) -> cg::Point {
  let (x, y) =
    crate::glide::core::corrected_origin(frame(destination), size_frame(achieved), gravity);
  cg::Point { x, y }
}

/// The frame as fractions of the work area, which is the space the preview
/// draws in. A work area with no extent has no fractions to give.
fn fractions(rect: cg::Rect, work_origin: (f64, f64), work_size: (f64, f64)) -> Option<FitRect> {
  crate::glide::core::frame_fractions(frame(rect), work_origin, work_size).map(|frame| FitRect {
    x: frame.x,
    y: frame.y,
    width: frame.width,
    height: frame.height,
  })
}

fn frame(rect: cg::Rect) -> crate::glide::core::GlideFrame {
  crate::glide::core::GlideFrame {
    x: rect.origin.x,
    y: rect.origin.y,
    width: rect.size.width,
    height: rect.size.height,
  }
}

fn size_frame(size: cg::Size) -> crate::glide::core::GlideFrame {
  crate::glide::core::GlideFrame {
    x: 0.0,
    y: 0.0,
    width: size.width,
    height: size.height,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn rect(x: f64, y: f64, width: f64, height: f64) -> cg::Rect {
    cg::Rect {
      origin: cg::Point { x, y },
      size: cg::Size { width, height },
    }
  }

  fn pull(horizontal: Gravity, vertical: Gravity) -> RegionGravity {
    RegionGravity {
      horizontal,
      vertical,
    }
  }

  #[test]
  fn a_rounding_sized_window_still_counts_as_a_fill() {
    let destination = cg::Size {
      width: 720.0,
      height: 875.0,
    };
    assert!(!deviates(
      cg::Size {
        width: 718.5,
        height: 876.0
      },
      destination
    ));
    assert!(deviates(
      cg::Size {
        width: 600.0,
        height: 875.0
      },
      destination
    ));
  }

  #[test]
  fn an_end_pull_lines_up_the_far_edge() {
    assert_eq!(
      corrected_origin(
        rect(-720.0, 25.0, 720.0, 875.0),
        cg::Size {
          width: 500.0,
          height: 875.0
        },
        pull(Gravity::End, Gravity::Center),
      ),
      cg::Point { x: -500.0, y: 25.0 }
    );
  }

  #[test]
  fn a_start_pull_leaves_the_origin_alone() {
    assert_eq!(
      corrected_origin(
        rect(0.0, 25.0, 720.0, 437.0),
        cg::Size {
          width: 500.0,
          height: 300.0
        },
        pull(Gravity::Start, Gravity::Start),
      ),
      cg::Point { x: 0.0, y: 25.0 }
    );
  }

  #[test]
  fn a_window_wider_than_its_region_overhangs_the_far_edge() {
    assert_eq!(
      corrected_origin(
        rect(0.0, 25.0, 300.0, 875.0),
        cg::Size {
          width: 480.0,
          height: 120.0
        },
        pull(Gravity::End, Gravity::Start),
      ),
      cg::Point { x: -180.0, y: 25.0 }
    );
  }

  #[test]
  fn a_center_pull_splits_the_slack_and_rounds() {
    assert_eq!(
      corrected_origin(
        rect(0.0, 0.0, 333.0, 600.0),
        cg::Size {
          width: 300.0,
          height: 599.0
        },
        pull(Gravity::Center, Gravity::Center),
      ),
      cg::Point { x: 17.0, y: 1.0 }
    );
  }

  #[test]
  fn the_frame_reports_as_work_area_fractions() {
    let actual = fractions(
      rect(-1_240.0, 25.0, 500.0, 875.0),
      (-1_440.0, 25.0),
      (1_440.0, 875.0),
    )
    .unwrap();

    assert!((actual.x - 200.0 / 1_440.0).abs() < f64::EPSILON);
    assert_eq!(actual.y, 0.0);
    assert!((actual.width - 500.0 / 1_440.0).abs() < f64::EPSILON);
    assert_eq!(actual.height, 1.0);
  }

  #[test]
  fn an_empty_work_area_reports_nothing() {
    assert!(fractions(rect(0.0, 0.0, 10.0, 10.0), (0.0, 0.0), (0.0, 900.0)).is_none());
  }
}
