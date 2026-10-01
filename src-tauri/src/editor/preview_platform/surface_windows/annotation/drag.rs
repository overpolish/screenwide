// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A press the annotation chrome has taken, before and after it becomes a
//! drag.

/// A press has to travel this far before it draws an arrow rather than
/// clearing the choice: a click and a very short drag are the same gesture to
/// a hand, and neither should leave a stub behind.
pub(super) const DRAG_SLOP: f64 = 3.0;

/// What a press does if it never travels, and what it becomes if it does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Press {
  Ordinary,
  /// A press with the toggle modifier on an annotation: a click adds it to
  /// the choice or takes it out, and a drag moves it as an ordinary press.
  Toggle,
  /// A press on one of several chosen together, or inside their box: a drag
  /// carries them all, measured on `layer`.
  Group,
  /// A marquee band over `layer`; `additive` when the toggle modifier was held
  /// at the press.
  Marquee {
    additive: bool,
  },
}

#[derive(Clone, Copy)]
pub(super) struct Drag {
  pub(super) target_kind: u32,
  pub(super) index: u32,
  pub(super) handle: u32,
  pub(super) origin: (f64, f64),
  pub(super) press: Press,
  /// The layer a group carry or a marquee band is measured on.
  pub(super) layer: i32,
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
      press: Press::Ordinary,
      layer: -1,
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
      press: Press::Ordinary,
      layer: -1,
      pending: false,
      begun: true,
    }
  }

  /// The same press, doing what `press` says over `layer`.
  pub(super) fn with_press(self, press: Press, layer: i32) -> Self {
    Self {
      press,
      layer,
      ..self
    }
  }

  /// The same press, now acting on `target_kind` through `handle`.
  pub(super) fn retargeted(self, target_kind: u32, handle: u32) -> Self {
    Self {
      target_kind,
      handle,
      ..self
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
mod tests;
