// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A press the annotation chrome has taken, before and after it becomes a
//! drag.

/// A press has to travel this far before it draws an arrow rather than
/// clearing the choice: a click and a very short drag are the same gesture to
/// a hand, and neither should leave a stub behind.
pub(super) const DRAG_SLOP: f64 = 3.0;

#[derive(Clone, Copy)]
pub(super) struct Drag {
  pub(super) target_kind: u32,
  pub(super) index: u32,
  pub(super) handle: u32,
  pub(super) origin: (f64, f64),
  /// The press has not travelled far enough to be a drag yet, so no gesture
  /// has begun and nothing has been edited.
  pending: bool,
  pub(super) begun: bool,
}

impl Drag {
  /// A press the chrome has taken but which has not travelled yet.
  pub(super) fn pending(target_kind: u32, index: u32, handle: u32, origin: (f64, f64)) -> Self {
    Self {
      target_kind,
      index,
      handle,
      origin,
      pending: true,
      begun: false,
    }
  }

  /// A press that is a gesture from the moment it lands: the counter tool drops
  /// an annotation where it is pressed rather than drawing one out, so a click
  /// alone commits it.
  pub(super) fn begun(target_kind: u32, index: u32, handle: u32, origin: (f64, f64)) -> Self {
    Self {
      target_kind,
      index,
      handle,
      origin,
      pending: false,
      begun: true,
    }
  }

  /// Advances the drag by one pointer sample, reporting whether this sample
  /// is the one that turns the press into a gesture. A press that has not
  /// travelled past the slop yet leaves nothing behind: no phase is emitted
  /// and no edit is made, so a click never nudges the arrow.
  pub(super) fn sample(&mut self, point: (f64, f64)) -> bool {
    if !self.pending {
      return false;
    }
    if (point.0 - self.origin.0).hypot(point.1 - self.origin.1) < DRAG_SLOP {
      return false;
    }
    self.pending = false;
    self.begun = true;
    true
  }

  /// Whether this sample should report movement at all. A press still inside
  /// the slop reports nothing.
  pub(super) fn reports(&self) -> bool {
    !self.pending
  }
}

#[cfg(test)]
#[path = "drag_tests.rs"]
mod tests;
