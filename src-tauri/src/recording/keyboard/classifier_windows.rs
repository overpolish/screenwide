// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Windows key classification, focus inspection and keycode translation.
//!
//! Every recorded keycode is a macOS virtual keycode: the sidecar format, the
//! export geometry table and the renderer label tables are all keyed on those,
//! so this is the one place where the platform difference is normalised away.

use std::collections::HashSet;
use std::time::Instant;

use windows::Win32::System::Com::{
  CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_MULTITHREADED,
};
use windows::Win32::UI::Accessibility::{
  CUIAutomation, IUIAutomation, UIA_ComboBoxControlTypeId, UIA_DocumentControlTypeId,
  UIA_EditControlTypeId,
};

use super::super::{FocusContext, KeyboardModifier, RawKeyboardEventKind};

/// What the low-level hook forwards to the worker. Deliberately plain data:
/// the callback runs inside the system input path and may not block.
#[derive(Clone, Copy)]
pub(super) struct PendingEvent {
  pub at: Instant,
  pub extended: bool,
  pub focus: u8,
  pub is_down: bool,
  pub virtual_key: u32,
}

pub(super) struct ClassifiedEvent {
  pub key_code: u16,
  pub kind: RawKeyboardEventKind,
  pub modifiers: Vec<KeyboardModifier>,
}

pub(super) fn encode_focus(focus: FocusContext) -> u8 {
  match focus {
    FocusContext::Unknown => 0,
    FocusContext::NonText => 1,
    FocusContext::Text => 2,
    FocusContext::Secure => 3,
  }
}

pub(super) fn decode_focus(value: u8) -> FocusContext {
  match value {
    1 => FocusContext::NonText,
    2 => FocusContext::Text,
    3 => FocusContext::Secure,
    _ => FocusContext::Unknown,
  }
}

/// COM for the calling thread, tolerant of a thread that already holds it in
/// another apartment.
struct Com {
  owned: bool,
}

impl Com {
  fn enter() -> Self {
    let result = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
    Self {
      owned: result.is_ok(),
    }
  }
}

impl Drop for Com {
  fn drop(&mut self) {
    if self.owned {
      unsafe { CoUninitialize() };
    }
  }
}

/// The focused-control probe. UI Automation is a cross-process call, so it is
/// only ever made from the worker thread, never from the hook callback.
pub(super) struct Focus {
  automation: Option<IUIAutomation>,
  _com: Com,
}

impl Focus {
  pub(super) fn enter() -> Self {
    let com = Com::enter();
    let automation = unsafe { CoCreateInstance(&CUIAutomation, None, CLSCTX_ALL) }.ok();
    Self {
      automation,
      _com: com,
    }
  }

  pub(super) fn context(&self) -> FocusContext {
    let Some(automation) = self.automation.as_ref() else {
      return FocusContext::Unknown;
    };
    let Ok(element) = (unsafe { automation.GetFocusedElement() }) else {
      return FocusContext::Unknown;
    };
    if unsafe { element.CurrentIsPassword() }.is_ok_and(|password| password.as_bool()) {
      return FocusContext::Secure;
    }
    match unsafe { element.CurrentControlType() } {
      Ok(control)
        if control == UIA_EditControlTypeId
          || control == UIA_DocumentControlTypeId
          || control == UIA_ComboBoxControlTypeId =>
      {
        FocusContext::Text
      }
      Ok(_) => FocusContext::NonText,
      Err(_) => FocusContext::Unknown,
    }
  }
}

/// Held keys, in macOS keycodes. The low-level hook reports neither auto-repeat
/// nor an aggregate modifier state, so both are derived from this set.
#[derive(Default)]
pub(super) struct KeyTracker {
  down: HashSet<u16>,
}

impl KeyTracker {
  pub(super) fn classify(&mut self, event: &PendingEvent) -> Option<ClassifiedEvent> {
    let key_code = mac_key_code(event.virtual_key, event.extended);
    let is_repeat = if event.is_down {
      !self.down.insert(key_code)
    } else {
      self.down.remove(&key_code);
      false
    };
    let kind = if let Some(modifier) = modifier_for_key_code(key_code) {
      // Windows auto-repeats a held modifier at the typematic rate. The shared
      // writer resolves aggregate modifier flags against its own held set, so
      // it reads a repeated down as a release; FlagsChanged must therefore
      // only ever report real transitions, as the macOS event tap does.
      if event.is_down && is_repeat {
        return None;
      }
      RawKeyboardEventKind::FlagsChanged {
        is_down: event.is_down,
        modifier,
      }
    } else if event.is_down {
      RawKeyboardEventKind::KeyDown {
        is_printable: is_printable(key_code),
        is_repeat,
      }
    } else {
      RawKeyboardEventKind::KeyUp
    };
    Some(ClassifiedEvent {
      key_code,
      kind,
      modifiers: self.modifiers(),
    })
  }

  fn modifiers(&self) -> Vec<KeyboardModifier> {
    [
      (KeyboardModifier::Command, [54, 55]),
      (KeyboardModifier::Control, [59, 62]),
      (KeyboardModifier::Option, [58, 61]),
      (KeyboardModifier::Shift, [56, 60]),
    ]
    .into_iter()
    .filter_map(|(modifier, codes)| {
      codes
        .iter()
        .any(|code| self.down.contains(code))
        .then_some(modifier)
    })
    .collect()
  }
}

fn modifier_for_key_code(key_code: u16) -> Option<KeyboardModifier> {
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
fn is_printable(key_code: u16) -> bool {
  matches!(
    key_code,
    0..=35 | 37..=47 | 49 | 50 | 65 | 67 | 69 | 75 | 78 | 81..=89 | 91..=95
  )
}

/// Alphabetic order, VK_A..VK_Z.
const LETTERS: [u16; 26] = [
  0, 11, 8, 2, 14, 3, 5, 4, 34, 38, 40, 37, 46, 45, 31, 35, 12, 15, 1, 17, 32, 9, 13, 7, 16, 6,
];
/// VK_0..VK_9, the number row.
const DIGITS: [u16; 10] = [29, 18, 19, 20, 21, 23, 22, 26, 28, 25];
/// VK_NUMPAD0..VK_NUMPAD9.
const KEYPAD_DIGITS: [u16; 10] = [82, 83, 84, 85, 86, 87, 88, 89, 91, 92];
/// VK_F1..VK_F16. macOS numbers the function row out of order.
const FUNCTION_KEYS: [u16; 16] = [
  122, 120, 99, 118, 96, 97, 98, 100, 101, 109, 103, 111, 105, 107, 113, 106,
];

/// A Windows virtual key with no macOS equivalent. The offset keeps it clear of
/// the macOS range so downstream width and label tables fall back cleanly.
fn passthrough(virtual_key: u32) -> u16 {
  0x0200 | (virtual_key & 0x00ff) as u16
}

/// Translates a Windows virtual-key code into the macOS virtual keycode for the
/// same physical key. `extended` is the LLKHF_EXTENDED flag, the only thing
/// that separates the navigation cluster from the numeric keypad, or Return
/// from keypad Enter.
fn mac_key_code(virtual_key: u32, extended: bool) -> u16 {
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

#[cfg(test)]
mod tests {
  use super::*;

  fn press(virtual_key: u32, extended: bool) -> PendingEvent {
    PendingEvent {
      at: Instant::now(),
      extended,
      focus: encode_focus(FocusContext::NonText),
      is_down: true,
      virtual_key,
    }
  }

  fn release(virtual_key: u32, extended: bool) -> PendingEvent {
    PendingEvent {
      is_down: false,
      ..press(virtual_key, extended)
    }
  }

  #[test]
  fn translates_letters_digits_and_punctuation_to_mac_keycodes() {
    assert_eq!(mac_key_code(0x41, false), 0);
    assert_eq!(mac_key_code(0x5a, false), 6);
    assert_eq!(mac_key_code(0x30, false), 29);
    assert_eq!(mac_key_code(0x35, false), 23);
    assert_eq!(mac_key_code(0xbd, false), 27);
    assert_eq!(mac_key_code(0x20, false), 49);
    assert_eq!(mac_key_code(0x70, false), 122);
    assert_eq!(mac_key_code(0x7f, false), 106);
  }

  #[test]
  fn the_extended_flag_separates_navigation_from_the_keypad() {
    assert_eq!(mac_key_code(0x25, true), 123);
    assert_eq!(mac_key_code(0x25, false), 86);
    assert_eq!(mac_key_code(0x2e, true), 117);
    assert_eq!(mac_key_code(0x2e, false), 65);
    assert_eq!(mac_key_code(0x0d, true), 76);
    assert_eq!(mac_key_code(0x0d, false), 36);
  }

  #[test]
  fn unmapped_virtual_keys_pass_through_outside_the_mac_range() {
    // VK_BROWSER_BACK has no macOS counterpart.
    assert_eq!(mac_key_code(0xa6, false), 0x02a6);
    assert!(mac_key_code(0xa6, false) > 126);
  }

  #[test]
  fn classifies_modifier_keys_as_flag_changes_on_both_sides() {
    let mut tracker = KeyTracker::default();
    let left = tracker
      .classify(&press(0xa2, false))
      .expect("a transition is classified");
    assert_eq!(left.key_code, 59);
    assert_eq!(
      left.kind,
      RawKeyboardEventKind::FlagsChanged {
        is_down: true,
        modifier: KeyboardModifier::Control
      }
    );
    let right = tracker
      .classify(&release(0x5c, false))
      .expect("a transition is classified");
    assert_eq!(right.key_code, 54);
    assert_eq!(
      right.kind,
      RawKeyboardEventKind::FlagsChanged {
        is_down: false,
        modifier: KeyboardModifier::Command
      }
    );
  }

  #[test]
  fn a_held_modifier_auto_repeat_is_suppressed() {
    let mut tracker = KeyTracker::default();
    assert!(tracker.classify(&press(0xa0, false)).is_some());
    // Windows re-sends key downs for a held modifier at the typematic rate;
    // forwarded, the shared writer would read every other one as a release.
    assert!(tracker.classify(&press(0xa0, false)).is_none());
    assert!(tracker.classify(&press(0xa0, false)).is_none());
    let released = tracker
      .classify(&release(0xa0, false))
      .expect("a transition is classified");
    assert_eq!(
      released.kind,
      RawKeyboardEventKind::FlagsChanged {
        is_down: false,
        modifier: KeyboardModifier::Shift
      }
    );
  }

  #[test]
  fn reports_held_modifiers_with_every_key() {
    let mut tracker = KeyTracker::default();
    tracker.classify(&press(0xa2, false));
    tracker.classify(&press(0xa1, false));
    let event = tracker
      .classify(&press(0x43, false))
      .expect("a key down is classified");
    assert_eq!(event.key_code, 8);
    assert_eq!(
      event.modifiers,
      vec![KeyboardModifier::Control, KeyboardModifier::Shift]
    );
    tracker.classify(&release(0xa2, false));
    let event = tracker
      .classify(&press(0x56, false))
      .expect("a key down is classified");
    assert_eq!(event.modifiers, vec![KeyboardModifier::Shift]);
  }

  #[test]
  fn a_second_key_down_without_a_release_is_a_repeat() {
    let mut tracker = KeyTracker::default();
    assert_eq!(
      tracker.classify(&press(0x41, false)).unwrap().kind,
      RawKeyboardEventKind::KeyDown {
        is_printable: true,
        is_repeat: false
      }
    );
    assert_eq!(
      tracker.classify(&press(0x41, false)).unwrap().kind,
      RawKeyboardEventKind::KeyDown {
        is_printable: true,
        is_repeat: true
      }
    );
    assert_eq!(
      tracker.classify(&release(0x41, false)).unwrap().kind,
      RawKeyboardEventKind::KeyUp
    );
    assert_eq!(
      tracker.classify(&press(0x41, false)).unwrap().kind,
      RawKeyboardEventKind::KeyDown {
        is_printable: true,
        is_repeat: false
      }
    );
  }

  #[test]
  fn only_character_producing_keys_are_printable() {
    assert!(is_printable(mac_key_code(0x41, false)));
    assert!(is_printable(mac_key_code(0x20, false)));
    assert!(is_printable(mac_key_code(0x61, false)));
    assert!(!is_printable(mac_key_code(0x0d, false)));
    assert!(!is_printable(mac_key_code(0x09, false)));
    assert!(!is_printable(mac_key_code(0x08, false)));
    assert!(!is_printable(mac_key_code(0x1b, false)));
    assert!(!is_printable(mac_key_code(0x25, true)));
    assert!(!is_printable(mac_key_code(0x70, false)));
    assert!(!is_printable(mac_key_code(0xa6, false)));
  }

  #[test]
  fn focus_round_trips_through_the_shared_atomic() {
    for focus in [
      FocusContext::Unknown,
      FocusContext::NonText,
      FocusContext::Text,
      FocusContext::Secure,
    ] {
      assert_eq!(decode_focus(encode_focus(focus)), focus);
    }
  }
}
