// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;

use super::key::NativeKey;
use crate::glide::settings::GlideControl;

pub(super) const MOUSE_MIDDLE: u32 = 0x04;
pub(super) const MOUSE_BACK: u32 = 0x05;
pub(super) const MOUSE_FORWARD: u32 = 0x06;

#[derive(Clone, Copy, Debug)]
pub(super) enum NativeControl {
  Key(NativeKey),
  Mouse(u32),
}

impl NativeControl {
  pub(super) fn from_control(control: GlideControl) -> Option<Self> {
    Some(match control {
      GlideControl::Key(code) => Self::Key(NativeKey::from_code(code)?),
      GlideControl::MouseMiddle => Self::Mouse(MOUSE_MIDDLE),
      GlideControl::MouseBack => Self::Mouse(MOUSE_BACK),
      GlideControl::MouseForward => Self::Mouse(MOUSE_FORWARD),
    })
  }

  pub(super) fn is_down(self) -> bool {
    match self {
      Self::Key(key) => key.is_down(),
      Self::Mouse(button) => (unsafe { GetAsyncKeyState(button as i32) }) < 0,
    }
  }

  pub(super) fn matches(self, input: u32) -> bool {
    match self {
      Self::Key(key) => key.matches(input),
      Self::Mouse(button) => button == input,
    }
  }

  pub(super) fn is_keyboard_modifier(self) -> bool {
    matches!(self, Self::Key(key) if key.is_modifier())
  }

  pub(super) fn uses_observed_state(self) -> bool {
    !self.is_keyboard_modifier()
  }

  pub(super) fn is_mouse_button(self) -> bool {
    matches!(self, Self::Mouse(_))
  }
}

#[cfg(test)]
mod tests {
  use keyboard_types::Code;

  use super::*;

  #[test]
  fn only_non_modifier_controls_use_hook_state() {
    let control = NativeControl::from_control(GlideControl::Key(Code::ControlLeft)).unwrap();
    let letter = NativeControl::from_control(GlideControl::Key(Code::KeyM)).unwrap();
    let mouse = NativeControl::from_control(GlideControl::MouseForward).unwrap();

    assert!(!control.uses_observed_state());
    assert!(letter.uses_observed_state());
    assert!(mouse.uses_observed_state());
  }
}
