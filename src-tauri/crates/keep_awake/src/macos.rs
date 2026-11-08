// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::time::Duration;

use core_foundation::base::{CFType, TCFType};
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
use core_foundation::string::CFString;
use core_graphics::display::CGDisplay;
use core_graphics::event::{CGEvent, CGEventTapLocation, CGEventType, CGMouseButton, EventField};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

use super::EVENT_TAG;

/// `kCGEventSourceStateCombinedSessionState`: every input source together,
/// what Chromium reads for idle time.
const COMBINED_SESSION_STATE: i32 = 0;
/// `kCGAnyInputEventType`.
const ANY_INPUT_EVENT: u32 = !0;

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
  fn CGEventSourceSecondsSinceLastEventType(state: i32, event_type: u32) -> f64;
  fn CGPreflightPostEventAccess() -> bool;
  fn CGSessionCopyCurrentDictionary() -> CFDictionaryRef;
}

pub(super) fn permitted() -> bool {
  // SAFETY: takes nothing and only reads this process's permission.
  unsafe { CGPreflightPostEventAccess() }
}

pub(super) fn idle() -> Duration {
  // SAFETY: plain values in, a number of seconds out.
  let seconds =
    unsafe { CGEventSourceSecondsSinceLastEventType(COMBINED_SESSION_STATE, ANY_INPUT_EVENT) };
  Duration::try_from_secs_f64(seconds).unwrap_or(Duration::ZERO)
}

/// Whether this session is the one on screen, unlocked, with its display on.
pub(super) fn screen_in_use() -> bool {
  // SAFETY: returns a dictionary this call owns, or null outside a login
  // session.
  let dictionary = unsafe { CGSessionCopyCurrentDictionary() };
  if dictionary.is_null() {
    return false;
  }
  // SAFETY: a non-null dictionary from a Copy function, released on drop.
  let session: CFDictionary<CFString, CFType> =
    unsafe { CFDictionary::wrap_under_create_rule(dictionary) };
  let flag = |key: &'static str| {
    session
      .find(CFString::from_static_string(key))
      .and_then(|value| value.downcast::<CFBoolean>())
      .is_some_and(bool::from)
  };
  // The lock key is absent unless the screen is locked; the console key is
  // false while another user's session is on screen.
  flag("kCGSSessionOnConsoleKey")
    && !flag("CGSSessionScreenIsLocked")
    && !CGDisplay::main().is_asleep()
}

/// Posts a mouse move to where the pointer already is.
pub(super) fn nudge() {
  let Ok(source) = CGEventSource::new(CGEventSourceStateID::HIDSystemState) else {
    return;
  };
  let Ok(here) = CGEvent::new(source.clone()) else {
    return;
  };
  let Ok(event) = CGEvent::new_mouse_event(
    source,
    CGEventType::MouseMoved,
    here.location(),
    CGMouseButton::Left,
  ) else {
    return;
  };
  event.set_integer_value_field(EventField::EVENT_SOURCE_USER_DATA, EVENT_TAG);
  event.post(CGEventTapLocation::HID);
}
