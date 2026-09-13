// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// Translates a Windows virtual-key code into the macOS virtual keycode for the
/// same physical key. `extended` is the LLKHF_EXTENDED flag, the only thing
/// that separates the navigation cluster from the numeric keypad, or Return
/// from keypad Enter.
pub(super) fn mac_key_code(virtual_key: u32, extended: bool) -> u16 {
  match virtual_key {
    0x41..=0x5a => LETTERS[(virtual_key - 0x41) as usize],
    0x30..=0x39 => DIGITS[(virtual_key - 0x30) as usize],
    0x60..=0x69 => KEYPAD_DIGITS[(virtual_key - 0x60) as usize],
    0x70..=0x7f => FUNCTION_KEYS[(virtual_key - 0x70) as usize],
    // VK_BACK, VK_TAB, VK_ESCAPE, VK_SPACE.
    0x08 => 51,
    0x09 => 48,
    0x1b => 53,
    0x20 => 49,
    // VK_RETURN; the keypad reports the same key with the extended flag set.
    0x0d if extended => 76,
    0x0d => 36,
    // The navigation cluster is extended; the same virtual keys arrive without
    // the flag from the numeric keypad while Num Lock is off.
    0x0c if extended => 71,
    0x0c => 87,
    0x21 if extended => 116,
    0x21 => 92,
    0x22 if extended => 121,
    0x22 => 85,
    0x23 if extended => 119,
    0x23 => 83,
    0x24 if extended => 115,
    0x24 => 89,
    0x25 if extended => 123,
    0x25 => 86,
    0x26 if extended => 126,
    0x26 => 91,
    0x27 if extended => 124,
    0x27 => 88,
    0x28 if extended => 125,
    0x28 => 84,
    0x2d if extended => 114,
    0x2d => 82,
    0x2e if extended => 117,
    0x2e => 65,
    // VK_MULTIPLY, VK_ADD, VK_SUBTRACT, VK_DECIMAL, VK_DIVIDE.
    0x6a => 67,
    0x6b => 69,
    0x6d => 78,
    0x6e => 65,
    0x6f => 75,
    // VK_CAPITAL, and VK_NUMLOCK where macOS keyboards carry Clear.
    0x14 => 57,
    0x90 => 71,
    // Modifiers. The unsided VK_SHIFT/VK_CONTROL/VK_MENU never reach a
    // low-level hook, but map them to the left key rather than pass them on.
    0x10 | 0xa0 => 56,
    0xa1 => 60,
    0x11 | 0xa2 => 59,
    0xa3 => 62,
    0x12 | 0xa4 => 58,
    0xa5 => 61,
    0x5b => 55,
    0x5c => 54,
    // OEM punctuation, on a US layout.
    0xba => 41,
    0xbb => 24,
    0xbc => 43,
    0xbd => 27,
    0xbe => 47,
    0xbf => 44,
    0xc0 => 50,
    0xdb => 33,
    0xdc => 42,
    0xdd => 30,
    0xde => 39,
    // VK_OEM_102, the extra ISO key beside the left Shift.
    0xe2 => 10,
    _ => passthrough(virtual_key),
  }
}

pub(super) fn modifier_for_key_code(key_code: u16) -> Option<KeyboardModifier> {
  match key_code {
    54 | 55 => Some(KeyboardModifier::Command),
    58 | 61 => Some(KeyboardModifier::Option),
    56 | 60 => Some(KeyboardModifier::Shift),
    59 | 62 => Some(KeyboardModifier::Control),
    _ => None,
  }
}

/// The macOS printable set, expressed on translated keycodes. Return, Tab,
/// Backspace and keypad Enter are excluded exactly as they are there.
pub(super) fn is_printable(key_code: u16) -> bool {
  matches!(
    key_code,
    0..=35 | 37..=47 | 49 | 50 | 65 | 67 | 69 | 75 | 78 | 81..=89 | 91..=95
  )
}
