// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

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
