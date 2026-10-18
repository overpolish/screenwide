// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A captured HWND as the shared Glide tween drives it, in physical pixels.

use super::target::WindowTarget;
use crate::glide::core::{tween::GlideWindow, GlideFrame};

pub(super) use crate::glide::core::tween::{animate_to, cancel, start};

impl GlideWindow for WindowTarget {
  fn duplicate(&self) -> Self {
    *self
  }

  fn is_movable(&self) -> bool {
    WindowTarget::is_movable(*self)
  }

  fn is_resizable(&self) -> bool {
    WindowTarget::is_resizable(*self)
  }

  fn frame(&self) -> Option<GlideFrame> {
    WindowTarget::frame(*self).ok()
  }

  fn prepare_for_move(&mut self) -> Option<GlideFrame> {
    WindowTarget::prepare_for_move(*self)
      .map_err(|error| eprintln!("Could not move the Glide window: {error}"))
      .ok()
  }

  /// `SetWindowPos` writes the origin and size together, so the settling
  /// frame needs no second write.
  fn place(&mut self, frame: GlideFrame, resizes: bool, _settling: bool) -> bool {
    if resizes {
      self.set_frame(frame).is_ok()
    } else {
      self.set_origin(frame).is_ok()
    }
  }

  fn move_to(&mut self, frame: GlideFrame) -> bool {
    self.set_origin(frame).is_ok()
  }

  fn raise(&self) {
    WindowTarget::raise(*self);
  }

  fn minimize(&mut self) {
    WindowTarget::minimize(*self);
  }

  fn restore(&self, _original: GlideFrame) {
    WindowTarget::restore(*self);
  }
}
