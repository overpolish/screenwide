// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::key::NativeKey;
use crate::glide::settings::GlideControl;

pub(super) const MOUSE_STATE_BASE: i64 = 0x100;

#[derive(Clone, Copy)]
pub(super) enum NativeControl {
  Key(NativeKey),
  Mouse(i64),
}

impl NativeControl {
  pub(super) fn from_control(control: GlideControl) -> Option<Self> {
    Some(match control {
      GlideControl::Key(code) => Self::Key(NativeKey::from_code(code)?),
      GlideControl::MouseMiddle => Self::Mouse(2),
      GlideControl::MouseBack => Self::Mouse(3),
      GlideControl::MouseForward => Self::Mouse(4),
    })
  }

  pub(super) fn matches_key(self, code: i64) -> bool {
    matches!(self, Self::Key(key) if key.matches(code))
  }

  pub(super) fn matches_mouse(self, button: i64) -> bool {
    matches!(self, Self::Mouse(configured) if configured == button)
  }

  pub(super) fn matches_state(self, code: i64) -> bool {
    match self {
      Self::Key(key) => key.matches(code),
      Self::Mouse(button) => MOUSE_STATE_BASE + button == code,
    }
  }
}
