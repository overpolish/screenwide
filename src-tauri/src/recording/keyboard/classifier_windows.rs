// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Windows key classification, focus inspection and keycode translation.
//!
//! Every recorded keycode is a macOS virtual keycode: the sidecar format, the
//! export geometry table and the renderer label tables are all keyed on those,
//! so this is the one place where the platform difference is normalised away.

#[cfg(test)]
#[path = "classifier_windows/tests.rs"]
mod tests;

#[path = "classifier_windows/key_mapping.rs"]
mod key_mapping;
use key_mapping::{is_printable, mac_key_code, modifier_for_key_code};

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
