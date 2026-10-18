// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A window as the shared Glide tween drives it on macOS, in logical points.
//! The Accessibility API writes origin and size separately, which is why the
//! settling frame repeats the origin.

use cidre::cg;

use crate::glide::core::{tween::GlideWindow, GlideFrame};

pub(super) use crate::glide::core::tween::start;

mod target;

pub(super) use target::WindowTarget;

pub(super) fn glide_frame(rect: cg::Rect) -> GlideFrame {
  GlideFrame {
    x: rect.origin.x,
    y: rect.origin.y,
    width: rect.size.width,
    height: rect.size.height,
  }
}

pub(super) fn cg_rect(frame: GlideFrame) -> cg::Rect {
  cg::Rect {
    origin: cg::Point {
      x: frame.x,
      y: frame.y,
    },
    size: cg::Size {
      width: frame.width,
      height: frame.height,
    },
  }
}

impl GlideWindow for WindowTarget {
  fn duplicate(&self) -> Self {
    WindowTarget::duplicate(self)
  }

  fn is_movable(&self) -> bool {
    WindowTarget::is_movable(self)
  }

  fn is_resizable(&self) -> bool {
    WindowTarget::is_resizable(self)
  }

  fn frame(&self) -> Option<GlideFrame> {
    WindowTarget::frame(self).map(glide_frame)
  }

  /// An application clamps a move against the size it still has, so a window
  /// travelling to a smaller region stops short of the destination, and the
  /// resize that follows can shift the origin again as the window is pulled
  /// back on screen. Repeating the position once the window is the right size
  /// settles it exactly where it belongs.
  fn place(&mut self, frame: GlideFrame, resizes: bool, settling: bool) -> bool {
    let rect = cg_rect(frame);
    if !self.set_pos(&rect.origin) {
      return false;
    }
    if !resizes {
      return true;
    }
    if !self.set_size(&rect.size) {
      return false;
    }
    !settling || self.set_pos(&rect.origin)
  }

  fn move_to(&mut self, frame: GlideFrame) -> bool {
    self.set_pos(&cg_rect(frame).origin)
  }

  fn raise(&self) {
    WindowTarget::raise(self);
  }

  fn minimize(&mut self) {
    WindowTarget::minimize(self);
  }
}
