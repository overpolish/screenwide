// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The keyboard, while a box's typing holds it, as the engine's typing
//! input. Editing keys are read by their codes; text goes through AppKit's
//! text input, so the keyboard layout, dead keys and input methods have had
//! their say before it arrives.

use objc2::runtime::{AnyObject, Sel};
use objc2::sel;
use objc2_app_kit::{NSEvent, NSEventModifierFlags};
use objc2_foundation::{NSArray, NSAttributedString, NSString};

use objc2::DefinedClass;

use super::EditorView;
use screenwide_workspace::editing::{TypingInput, TypingKey};

/// The key a Mac virtual key code names, as a box being typed into knows it.
fn typing_key(code: u16) -> Option<TypingKey> {
  Some(match code {
    53 => TypingKey::Escape,
    51 => TypingKey::Backspace,
    117 => TypingKey::Delete,
    123 => TypingKey::Left,
    124 => TypingKey::Right,
    125 => TypingKey::Down,
    126 => TypingKey::Up,
    115 => TypingKey::Home,
    119 => TypingKey::End,
    _ => return None,
  })
}

fn modifiers(event: &NSEvent) -> (bool, bool) {
  let flags = event.modifierFlags();
  (
    flags.contains(NSEventModifierFlags::Command),
    flags.contains(NSEventModifierFlags::Shift),
  )
}

/// Hands a key going down to the typing. Answers whether it was taken.
pub(super) fn key_down(view: &EditorView, event: &NSEvent) -> bool {
  if !view.ivars().typing.get() {
    return false;
  }
  let (shortcut, shift) = modifiers(event);
  if let Some(key) = typing_key(event.keyCode()) {
    view.typing(TypingInput::Key {
      key,
      shortcut,
      shift,
    });
    return true;
  }
  if shortcut {
    return letter(view, event, shift);
  }
  view.typing(TypingInput::Key {
    key: TypingKey::Other,
    shortcut,
    shift,
  });
  view.interpretKeyEvents(&NSArray::from_slice(&[event]));
  true
}

/// Text the keyboard or an input method committed, NSString or
/// NSAttributedString as AppKit pleases.
pub(super) fn insert(view: &EditorView, string: &AnyObject) {
  let text = if let Some(text) = string.downcast_ref::<NSString>() {
    text.to_string()
  } else if let Some(text) = string.downcast_ref::<NSAttributedString>() {
    text.string().to_string()
  } else {
    return;
  };
  for unit in text.encode_utf16() {
    // The arrows and function keys arrive as private-use characters, which
    // type nothing.
    if !(0xF700..=0xF8FF).contains(&unit) {
      view.typing(TypingInput::Char(unit));
    }
  }
}

/// A key AppKit turned into an editing command rather than text. The ones
/// read by their codes never reach here.
pub(super) fn command(view: &EditorView, selector: Sel) {
  let input = if selector == sel!(insertNewline:) {
    TypingInput::Char(u16::from(b'\r'))
  } else if selector == sel!(insertTab:) {
    TypingInput::Char(u16::from(b'\t'))
  } else if selector == sel!(cancelOperation:) {
    TypingInput::Key {
      key: TypingKey::Escape,
      shortcut: false,
      shift: false,
    }
  } else {
    return;
  };
  view.typing(input);
}

/// Command with a letter reaches the menu before `keyDown:`; while typing
/// it is the box's copy, cut, paste or select-all instead.
pub(super) fn key_equivalent(view: &EditorView, event: &NSEvent) -> bool {
  if !view.ivars().typing.get() {
    return false;
  }
  let (shortcut, shift) = modifiers(event);
  shortcut && letter(view, event, shift)
}

fn letter(view: &EditorView, event: &NSEvent, shift: bool) -> bool {
  let letter = event
    .charactersIgnoringModifiers()
    .and_then(|text| text.to_string().chars().next())
    .filter(char::is_ascii_alphabetic)
    .map(|letter| letter.to_ascii_uppercase());
  let Some(letter) = letter else {
    return false;
  };
  view.typing(TypingInput::Key {
    key: TypingKey::Letter(letter),
    shortcut: true,
    shift,
  });
  true
}
