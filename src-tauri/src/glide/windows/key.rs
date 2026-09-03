// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use keyboard_types::Code;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VIRTUAL_KEY};

#[derive(Clone, Copy, Debug)]
pub(super) struct NativeKey {
  first: VIRTUAL_KEY,
  second: Option<VIRTUAL_KEY>,
}

impl NativeKey {
  pub(super) fn from_code(code: Code) -> Option<Self> {
    let name = code.to_string();
    if let Some(letter) = name.strip_prefix("Key").filter(|value| value.len() == 1) {
      return Self::one(u16::from(letter.as_bytes()[0].to_ascii_uppercase()));
    }
    if let Some(digit) = name.strip_prefix("Digit").filter(|value| value.len() == 1) {
      return Self::one(u16::from(digit.as_bytes()[0]));
    }
    if let Some(function) = name
      .strip_prefix('F')
      .and_then(|value| value.parse::<u16>().ok())
    {
      if (1..=24).contains(&function) {
        return Self::one(0x6f + function);
      }
    }
    if let Some(number) = name
      .strip_prefix("Numpad")
      .and_then(|value| value.parse::<u16>().ok())
    {
      if number <= 9 {
        return Self::one(0x60 + number);
      }
    }
    let (first, second) = match code {
      Code::Backspace => (0x08, None),
      Code::Tab => (0x09, None),
      Code::Enter | Code::NumpadEnter => (0x0d, None),
      Code::ShiftLeft => (0xa0, Some(0xa1)),
      Code::ShiftRight => (0xa1, None),
      Code::ControlLeft => (0xa2, Some(0xa3)),
      Code::ControlRight => (0xa3, None),
      Code::AltLeft => (0xa4, Some(0xa5)),
      Code::AltRight => (0xa5, None),
      Code::Pause => (0x13, None),
      Code::CapsLock => (0x14, None),
      Code::Escape => (0x1b, None),
      Code::Space => (0x20, None),
      Code::PageUp => (0x21, None),
      Code::PageDown => (0x22, None),
      Code::End => (0x23, None),
      Code::Home => (0x24, None),
      Code::ArrowLeft => (0x25, None),
      Code::ArrowUp => (0x26, None),
      Code::ArrowRight => (0x27, None),
      Code::ArrowDown => (0x28, None),
      Code::PrintScreen => (0x2c, None),
      Code::Insert => (0x2d, None),
      Code::Delete => (0x2e, None),
      Code::MetaLeft => (0x5b, Some(0x5c)),
      Code::MetaRight => (0x5c, None),
      Code::ContextMenu => (0x5d, None),
      Code::NumpadMultiply => (0x6a, None),
      Code::NumpadAdd => (0x6b, None),
      Code::NumpadSubtract => (0x6d, None),
      Code::NumpadDecimal => (0x6e, None),
      Code::NumpadDivide => (0x6f, None),
      Code::NumLock => (0x90, None),
      Code::ScrollLock => (0x91, None),
      Code::Semicolon => (0xba, None),
      Code::Equal | Code::NumpadEqual => (0xbb, None),
      Code::Comma => (0xbc, None),
      Code::Minus => (0xbd, None),
      Code::Period => (0xbe, None),
      Code::Slash => (0xbf, None),
      Code::Backquote => (0xc0, None),
      Code::BracketLeft => (0xdb, None),
      Code::Backslash => (0xdc, None),
      Code::BracketRight => (0xdd, None),
      Code::Quote => (0xde, None),
      Code::IntlBackslash => (0xe2, None),
      _ => return None,
    };
    Some(Self {
      first: VIRTUAL_KEY(first),
      second: second.map(VIRTUAL_KEY),
    })
  }

  fn one(value: u16) -> Option<Self> {
    Some(Self {
      first: VIRTUAL_KEY(value),
      second: None,
    })
  }

  pub(super) fn is_down(self) -> bool {
    (unsafe { GetAsyncKeyState(self.first.0 as i32) }) < 0
      || self
        .second
        .is_some_and(|key| (unsafe { GetAsyncKeyState(key.0 as i32) }) < 0)
  }

  pub(super) fn matches(self, key: u32) -> bool {
    u32::from(self.first.0) == key || self.second.is_some_and(|value| u32::from(value.0) == key)
  }

  pub(super) fn is_modifier(self) -> bool {
    matches!(self.first.0, 0x5b | 0x5c | 0xa0..=0xa5)
  }
}
