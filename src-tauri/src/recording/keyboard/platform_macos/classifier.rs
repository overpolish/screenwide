// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! macOS key classification and modifier translation.

use cidre::ax;
use core_graphics::event::{CGEventFlags, KeyCode};

use super::super::{FocusContext, KeyboardModifier, RawKeyboardEventKind};

#[derive(Clone, Copy)]
pub(super) enum PendingKind {
  KeyDown,
  KeyUp,
  FlagsChanged,
}

pub(super) fn focus_context() -> FocusContext {
  let system = ax::UiElement::sys_wide();
  let _ = system.set_messaging_timeout_secs(0.05);
  let Ok(focused) = system.focused_ui_element() else {
    return FocusContext::Unknown;
  };
  let _ = focused.set_messaging_timeout_secs(0.05);
  let role = focused.role().ok().map(|value| value.to_string());
  let subrole = focused
    .attr_value(ax::attr::subrole())
    .ok()
    .and_then(|value| value.try_as_string().map(|value| value.to_string()));
  let accepts_text = focused.is_settable(ax::attr::value()).unwrap_or(false);
  if subrole.as_deref() == Some("AXSecureTextField") {
    FocusContext::Secure
  } else if matches!(
    role.as_deref(),
    Some("AXTextField" | "AXTextArea" | "AXComboBox")
  ) || subrole.as_deref() == Some("AXSearchField")
    || accepts_text
  {
    FocusContext::Text
  } else if role.is_some() {
    FocusContext::NonText
  } else {
    FocusContext::Unknown
  }
}

pub(super) fn modifiers(flags: CGEventFlags) -> Vec<KeyboardModifier> {
  [
    (CGEventFlags::CGEventFlagCommand, KeyboardModifier::Command),
    (CGEventFlags::CGEventFlagControl, KeyboardModifier::Control),
    (CGEventFlags::CGEventFlagAlternate, KeyboardModifier::Option),
    (CGEventFlags::CGEventFlagShift, KeyboardModifier::Shift),
    (
      CGEventFlags::CGEventFlagSecondaryFn,
      KeyboardModifier::Function,
    ),
  ]
  .into_iter()
  .filter_map(|(flag, modifier)| flags.contains(flag).then_some(modifier))
  .collect()
}

pub(super) fn event_kind(
  kind: PendingKind,
  key_code: u16,
  flags: CGEventFlags,
  is_repeat: bool,
) -> Option<RawKeyboardEventKind> {
  if matches!(kind, PendingKind::KeyUp) {
    return Some(RawKeyboardEventKind::KeyUp);
  }
  if let Some(modifier) = modifier_for_key_code(key_code) {
    return Some(RawKeyboardEventKind::FlagsChanged {
      is_down: flags_for_modifier(modifier, flags),
      modifier,
    });
  }
  match kind {
    PendingKind::FlagsChanged => None,
    PendingKind::KeyDown => Some(RawKeyboardEventKind::KeyDown {
      is_printable: is_printable(key_code),
      is_repeat,
    }),
    PendingKind::KeyUp => Some(RawKeyboardEventKind::KeyUp),
  }
}

fn modifier_for_key_code(key_code: u16) -> Option<KeyboardModifier> {
  match key_code {
    54 | 55 => Some(KeyboardModifier::Command),
    58 | 61 => Some(KeyboardModifier::Option),
    56 | 60 => Some(KeyboardModifier::Shift),
    59 | 62 => Some(KeyboardModifier::Control),
    63 => Some(KeyboardModifier::Function),
    _ => None,
  }
}

fn flags_for_modifier(modifier: KeyboardModifier, flags: CGEventFlags) -> bool {
  match modifier {
    KeyboardModifier::Command => flags.contains(CGEventFlags::CGEventFlagCommand),
    KeyboardModifier::Control => flags.contains(CGEventFlags::CGEventFlagControl),
    KeyboardModifier::Option => flags.contains(CGEventFlags::CGEventFlagAlternate),
    KeyboardModifier::Shift => flags.contains(CGEventFlags::CGEventFlagShift),
    KeyboardModifier::Function => flags.contains(CGEventFlags::CGEventFlagSecondaryFn),
  }
}

fn is_printable(key_code: u16) -> bool {
  matches!(
    key_code,
    KeyCode::ANSI_A
      | KeyCode::ANSI_S
      | KeyCode::ANSI_D
      | KeyCode::ANSI_F
      | KeyCode::ANSI_H
      | KeyCode::ANSI_G
      | KeyCode::ANSI_Z
      | KeyCode::ANSI_X
      | KeyCode::ANSI_C
      | KeyCode::ANSI_V
      | KeyCode::ISO_SECTION
      | KeyCode::ANSI_B
      | KeyCode::ANSI_Q
      | KeyCode::ANSI_W
      | KeyCode::ANSI_E
      | KeyCode::ANSI_R
      | KeyCode::ANSI_Y
      | KeyCode::ANSI_T
      | KeyCode::ANSI_1
      | KeyCode::ANSI_2
      | KeyCode::ANSI_3
      | KeyCode::ANSI_4
      | KeyCode::ANSI_6
      | KeyCode::ANSI_5
      | KeyCode::ANSI_EQUAL
      | KeyCode::ANSI_9
      | KeyCode::ANSI_7
      | KeyCode::ANSI_MINUS
      | KeyCode::ANSI_8
      | KeyCode::ANSI_0
      | KeyCode::ANSI_RIGHT_BRACKET
      | KeyCode::ANSI_O
      | KeyCode::ANSI_U
      | KeyCode::ANSI_LEFT_BRACKET
      | KeyCode::ANSI_I
      | KeyCode::ANSI_P
      | KeyCode::ANSI_L
      | KeyCode::ANSI_J
      | KeyCode::ANSI_QUOTE
      | KeyCode::ANSI_K
      | KeyCode::ANSI_SEMICOLON
      | KeyCode::ANSI_BACKSLASH
      | KeyCode::ANSI_COMMA
      | KeyCode::ANSI_SLASH
      | KeyCode::ANSI_N
      | KeyCode::ANSI_M
      | KeyCode::ANSI_PERIOD
      | KeyCode::SPACE
      | KeyCode::ANSI_GRAVE
      | KeyCode::ANSI_KEYPAD_DECIMAL
      | KeyCode::ANSI_KEYPAD_MULTIPLY
      | KeyCode::ANSI_KEYPAD_PLUS
      | KeyCode::ANSI_KEYPAD_DIVIDE
      | KeyCode::ANSI_KEYPAD_MINUS
      | KeyCode::ANSI_KEYPAD_EQUAL
      | KeyCode::ANSI_KEYPAD_0
      | KeyCode::ANSI_KEYPAD_1
      | KeyCode::ANSI_KEYPAD_2
      | KeyCode::ANSI_KEYPAD_3
      | KeyCode::ANSI_KEYPAD_4
      | KeyCode::ANSI_KEYPAD_5
      | KeyCode::ANSI_KEYPAD_6
      | KeyCode::ANSI_KEYPAD_7
      | KeyCode::ANSI_KEYPAD_8
      | KeyCode::ANSI_KEYPAD_9
      | KeyCode::JIS_YEN
      | KeyCode::JIS_UNDERSCORE
      | KeyCode::JIS_KEYPAD_COMMA
  )
}
