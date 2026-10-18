// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

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
  screenwide_capture::cursor::set_cursor_visibility(false, None);
  Ok(())
}

pub(super) fn release_cursor(anchor: CGPoint, revealed: bool) {
  crate::cursor_scrub::restore_cursor_at(anchor);
  PINNED.store(false, std::sync::atomic::Ordering::Release);
  if revealed {
    crate::cursor_scrub::show_cursor();
    screenwide_capture::cursor::set_cursor_visibility(true, Some((anchor.x, anchor.y)));
  }
}

/// Warps the pointer onto a window that moved out from under it, leaving the
/// hide and pin state a gesture owns untouched: a centering never hid the
/// cursor, so there is nothing to reveal. The recording stream is told where
/// the pointer went, since a warp emits no mouse event of its own.
pub(super) fn carry_cursor(landing: CGPoint) {
  crate::cursor_scrub::restore_cursor_at(landing);
  screenwide_capture::cursor::set_cursor_visibility(true, Some((landing.x, landing.y)));
}
