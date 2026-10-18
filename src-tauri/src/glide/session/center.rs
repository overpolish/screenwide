// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The double tap that centers a window on its monitor's work area.

use super::{GlidePlatform, Point};
use crate::glide::core::tween::{animate_to, GlideWindow};
use crate::glide::core::tween_completion::after_current;
use crate::glide::core::{frames_match, landing_point, GlideFrame};
use crate::glide::BusyLease;

/// Centers `target`, read at `frame`, in `work`, keeping its size. Centering
/// commits a move like every other glide, so when `cursor_follows` the
/// pointer keeps its grip on the window: the warp waits for the tween and
/// reads the frame the window actually reached, and a window that never
/// travelled - already centered, or one its application refused to move -
/// leaves the pointer where the hand put it.
pub(crate) fn center<P: GlidePlatform>(
  target: &P::Window,
  frame: GlideFrame,
  work: GlideFrame,
  anchor: Point,
  cursor_follows: bool,
) {
  animate_to(target.duplicate(), centered(frame, work), None);
  if !cursor_follows {
    return;
  }
  let grip = target.duplicate();
  let busy = BusyLease::acquire();
  after_current(Box::new(move || {
    let _busy = busy;
    let Some(achieved) = grip.frame() else {
      return;
    };
    if !frames_match(frame, achieved, 1.0) {
      P::carry_pointer(landing_point(anchor, frame, achieved));
    }
  }));
}

/// `frame` centered in `work`. A window larger than the work area pins to its
/// origin on that axis.
fn centered(frame: GlideFrame, work: GlideFrame) -> GlideFrame {
  GlideFrame {
    x: work.x + (work.width - frame.width).max(0.0) / 2.0,
    y: work.y + (work.height - frame.height).max(0.0) / 2.0,
    ..frame
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn rect(x: f64, y: f64, width: f64, height: f64) -> GlideFrame {
    GlideFrame {
      x,
      y,
      width,
      height,
    }
  }

  #[test]
  fn centers_a_window_in_the_work_area_of_its_monitor() {
    let centered = centered(
      rect(0.0, 0.0, 800.0, 475.0),
      rect(-1_440.0, 25.0, 1_440.0, 875.0),
    );
    assert_eq!((centered.x, centered.y), (-1_120.0, 225.0));
  }

  #[test]
  fn pins_a_window_larger_than_the_work_area_to_its_origin() {
    let centered = centered(
      rect(0.0, 0.0, 1_800.0, 1_000.0),
      rect(0.0, 25.0, 1_440.0, 875.0),
    );
    assert_eq!((centered.x, centered.y), (0.0, 25.0));
  }
}
