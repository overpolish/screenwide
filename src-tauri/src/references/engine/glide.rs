// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

//! A reference gliding to a place and size it is put back to, by Back to
//! origin or Reset size, rather than jumping there: the picture, its window
//! and its island follow it frame by frame, as they follow a resize, over
//! the app's `reshape` timing with its ease out (`src/lib/motion.ts`). A
//! newer glide takes over from one still under way.

use std::time::{Duration, Instant};

use screenwide_osc::geometry::{Point, Rect, Size};

use super::super::fit::fit;
use super::hold::{hold_round, middle};
use super::place::{picture_on_desktop, place, release_hold};
use super::{with, ReferenceHost};

/// `reshape`: how long the glide takes.
const GLIDE: Duration = Duration::from_millis(280);
/// How often the glide is drawn.
const FRAME: Duration = Duration::from_millis(16);

/// Glides the reference `label`'s picture to `target` on the desktop, its
/// window held round both ends of the glide (`hold`).
pub(super) fn glide_to(label: &str, target: Rect) {
  let started = with(label, |native| {
    native.glide += 1;
    let ending = fit(
      target,
      native.stage.island,
      native.host.room_at(middle(target)),
    );
    hold_round(native, ending.window);
    (native.glide, picture_on_desktop(native), native.app.clone())
  });
  let Some((glide, from, app)) = started else {
    return;
  };
  let label = label.to_owned();
  let began = Instant::now();
  std::thread::spawn(move || loop {
    std::thread::sleep(FRAME);
    let progress = (began.elapsed().as_secs_f64() / GLIDE.as_secs_f64()).min(1.0);
    let step_label = label.clone();
    let stepped = app.run_on_main_thread(move || step(&step_label, glide, from, target, progress));
    if stepped.is_err() || progress >= 1.0 {
      break;
    }
  });
}

/// Draws the glide `glide` at `progress`, unless a newer one took over.
fn step(label: &str, glide: u64, from: Rect, target: Rect, progress: f64) {
  with(label, |native| {
    if native.glide != glide {
      return;
    }
    let t = if progress >= 1.0 {
      1.0
    } else {
      ease_out(progress)
    };
    let mix = |a: f64, b: f64| a + (b - a) * t;
    let size = Size {
      width: mix(from.size.width, target.size.width),
      height: mix(from.size.height, target.size.height),
    };
    native.stage.size = size;
    let origin = Point {
      x: mix(from.origin.x, target.origin.x),
      y: mix(from.origin.y, target.origin.y),
    };
    place(native, Rect { origin, size });
    // Once there, the window is fitted round the picture again, unless a
    // resize holding it began meanwhile.
    if progress >= 1.0 && !native.stage.resizing() {
      release_hold(native);
    }
  });
}

/// Motion's `easeOut`, the cubic Bézier through (0, 0), (0, 0), (0.58, 1)
/// and (1, 1), at `x` along it.
fn ease_out(x: f64) -> f64 {
  let curve = |t: f64, a: f64, b: f64| {
    3.0 * (1.0 - t).powi(2) * t * a + 3.0 * (1.0 - t) * t * t * b + t.powi(3)
  };
  // The curve's t for `x`, by bisection: its x runs one way from 0 to 1.
  let (mut low, mut high) = (0.0, 1.0);
  for _ in 0..30 {
    let middle = (low + high) / 2.0;
    if curve(middle, 0.0, 0.58) < x {
      low = middle;
    } else {
      high = middle;
    }
  }
  curve((low + high) / 2.0, 0.0, 1.0)
}

#[cfg(test)]
mod tests {
  use super::ease_out;

  #[test]
  fn the_ease_out_runs_from_still_to_done_and_leads_early() {
    assert!(ease_out(0.0).abs() < 1e-6);
    assert!((ease_out(1.0) - 1.0).abs() < 1e-6);
    assert!(ease_out(0.5) > 0.5);
  }
}
