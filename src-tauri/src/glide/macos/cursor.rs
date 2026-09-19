// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use cidre::cg;
use core_graphics::geometry::CGPoint;

/// Whether the cursor is currently disassociated from the mouse. While it is,
/// event locations report the pinned point, not where the hand has moved - so
/// nothing that trusts an event's location may begin until the release.
static PINNED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub(super) fn is_cursor_pinned() -> bool {
  PINNED.load(std::sync::atomic::Ordering::Acquire)
}

pub(super) fn pin_cursor(anchor: CGPoint) -> Result<(), String> {
  crate::cursor_scrub::pin_cursor_at(anchor)?;
  PINNED.store(true, std::sync::atomic::Ordering::Release);
  Ok(())
}

pub(super) fn hide_cursor() -> Result<(), String> {
  crate::cursor_scrub::hide_cursor()?;
  crate::recording::cursor::set_cursor_visibility(false, None);
  Ok(())
}

/// Where the cursor lands after a commit: the same grip on the window it
/// carried, at the frame the window actually reached. The grip is proportional
/// along the titlebar (a resize keeps the relative hold) but an absolute offset
/// from the top (a proportional vertical would land in the content of a taller
/// window), clamped into the achieved frame.
pub(super) fn landing_point(anchor: CGPoint, original: cg::Rect, achieved: cg::Rect) -> CGPoint {
  let (x, y) =
    crate::glide::core::landing_point((anchor.x, anchor.y), frame(original), frame(achieved));
  CGPoint::new(x, y)
}

pub(super) fn returns_to_origin(original: cg::Rect, destination: cg::Rect) -> bool {
  crate::glide::core::frames_match(frame(original), frame(destination), 1.0)
}

fn frame(rect: cg::Rect) -> crate::glide::core::GlideFrame {
  crate::glide::core::GlideFrame {
    x: rect.origin.x,
    y: rect.origin.y,
    width: rect.size.width,
    height: rect.size.height,
  }
}

pub(super) fn release_cursor(anchor: CGPoint, revealed: bool) {
  crate::cursor_scrub::restore_cursor_at(anchor);
  PINNED.store(false, std::sync::atomic::Ordering::Release);
  if revealed {
    crate::cursor_scrub::show_cursor();
    crate::recording::cursor::set_cursor_visibility(true, Some((anchor.x, anchor.y)));
  }
}

/// Warps the pointer onto a window that moved out from under it, leaving the
/// hide and pin state a gesture owns untouched: a centering never hid the
/// cursor, so there is nothing to reveal. The recording stream is told where
/// the pointer went, since a warp emits no mouse event of its own.
pub(super) fn carry_cursor(landing: CGPoint) {
  crate::cursor_scrub::restore_cursor_at(landing);
  crate::recording::cursor::set_cursor_visibility(true, Some((landing.x, landing.y)));
}

#[cfg(test)]
mod tests {
  use super::*;

  const ORIGINAL: cg::Rect = rect(100.0, 50.0, 800.0, 600.0);

  const fn rect(x: f64, y: f64, width: f64, height: f64) -> cg::Rect {
    cg::Rect {
      origin: cg::Point { x, y },
      size: cg::Size { width, height },
    }
  }

  #[test]
  fn keeps_the_proportional_grip_across_a_move_and_resize() {
    // Grabbed a quarter along the titlebar, 10 down: same quarter of the new
    // width, same 10 from the new top.
    let landing = landing_point(
      CGPoint::new(300.0, 60.0),
      ORIGINAL,
      rect(960.0, 25.0, 400.0, 500.0),
    );
    assert_eq!((landing.x, landing.y), (1_060.0, 35.0));
  }

  #[test]
  fn clamps_a_grip_outside_the_achieved_frame() {
    let landing = landing_point(
      CGPoint::new(1_000.0, 700.0),
      ORIGINAL,
      rect(0.0, 0.0, 200.0, 100.0),
    );
    assert_eq!((landing.x, landing.y), (200.0, 100.0));
  }

  #[test]
  fn recognises_a_destination_that_returns_to_the_original_frame() {
    assert!(returns_to_origin(ORIGINAL, rect(100.5, 49.5, 800.5, 599.5)));
    assert!(!returns_to_origin(
      ORIGINAL,
      rect(102.0, 50.0, 800.0, 600.0)
    ));
  }
}
