// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

use std::{
  collections::HashSet,
  sync::{
    atomic::{AtomicIsize, Ordering},
    LazyLock, Mutex,
  },
};

use screenwide_keyboard_hook::{KeyEvent, Phase, Priority, Registration, Route};
use windows::Win32::{
  Foundation::{HWND, LPARAM, WPARAM},
  UI::WindowsAndMessaging::{PostMessageW, WM_APP},
};

use super::{native_settings, session, titlebar, InputKind};

pub(super) const WM_GLIDE_KEY: u32 = WM_APP + 3;
pub(super) const WM_GLIDE_SHORTCUT: u32 = WM_APP + 7;
static TARGET: AtomicIsize = AtomicIsize::new(0);
/// Mouse buttons Glide took, so their release is kept from the system too.
/// The keyboard hook keeps a taken key's own.
#[derive(Default)]
pub(super) struct Reservation {
  keys: HashSet<u32>,
}
impl Reservation {
  pub(super) fn transition(
    &mut self,
    key: u32,
    pressed: bool,
    eligible: bool,
    posted: bool,
  ) -> bool {
    if pressed {
      if self.keys.contains(&key) {
        return true;
      }
      if eligible && posted {
        self.keys.insert(key);
        return true;
      }
      false
    } else {
      self.keys.remove(&key)
    }
  }
}
static RESERVED: LazyLock<Mutex<Reservation>> = LazyLock::new(|| {
  Mutex::new(Reservation {
    keys: HashSet::new(),
  })
});

pub(super) fn reserve(key: u32, pressed: bool, eligible: bool, posted: bool) -> bool {
  RESERVED
    .lock()
    .is_ok_and(|mut reservation| reservation.transition(key, pressed, eligible, posted))
}

/// Glide's place on the process's keyboard hook, which hands Glide every key
/// the overlays and the system keys leave, injected ones included: a
/// remote-desktop or streaming host types that way.
pub(super) struct KeyHook {
  _registration: Registration,
}

impl Drop for KeyHook {
  fn drop(&mut self) {
    TARGET.store(0, Ordering::Release);
  }
}

pub(super) fn install(target: HWND) -> Result<KeyHook, String> {
  TARGET.store(target.0 as isize, Ordering::Release);
  match screenwide_keyboard_hook::take(Priority::Glide, route) {
    Ok(registration) => Ok(KeyHook {
      _registration: registration,
    }),
    Err(error) => {
      TARGET.store(0, Ordering::Release);
      Err(error)
    }
  }
}

/// Posts Glide's keys to its input window and takes a control's press while
/// it drives Glide.
fn route(event: &KeyEvent) -> Route {
  let key = event.key;
  let pressed = event.phase != Phase::Release;
  if super::deliberate::types(key, pressed) {
    return Route::Pass;
  }
  native_settings::observe(key, pressed);
  let configured = configured(key);
  let escaped = key == u32::from(windows::Win32::UI::Input::KeyboardAndMouse::VK_ESCAPE.0)
    && (session::active_input().is_some() || super::spaces::active_input().is_some())
    && !configured;
  let eligible = event.may_take && (suppresses(key) || escaped);
  let posted = (configured || escaped) && forward(key, pressed);
  if pressed && eligible && posted {
    Route::Take
  } else {
    Route::Pass
  }
}

pub(super) fn configured(key: u32) -> bool {
  let settings = native_settings::snapshot();
  settings.mouse_modifier.matches(key)
    || settings.monitors_modifier.matches(key)
    || settings.spaces_modifier.matches(key)
    || settings.thirds_modifier.matches(key)
}

pub(super) fn forward(key: u32, pressed: bool) -> bool {
  let target = TARGET.load(Ordering::Acquire);
  if target != 0 {
    unsafe {
      PostMessageW(
        Some(HWND(target as *mut std::ffi::c_void)),
        WM_GLIDE_KEY,
        WPARAM(key as usize),
        LPARAM(isize::from(pressed)),
      )
      .is_ok()
    }
  } else {
    false
  }
}

/// Asks the input window to cancel the glide a shortcut interrupted, outside
/// the hook, which must return quickly.
pub(super) fn post_shortcut() {
  let target = TARGET.load(Ordering::Acquire);
  if target != 0 {
    let _ = unsafe {
      PostMessageW(
        Some(HWND(target as *mut std::ffi::c_void)),
        WM_GLIDE_SHORTCUT,
        WPARAM(0),
        LPARAM(0),
      )
    };
  }
}

pub(super) fn suppresses(key: u32) -> bool {
  let settings = native_settings::snapshot();
  if !settings.enabled
    || crate::shortcuts::is_capturing()
    || super::APP
      .get()
      .is_some_and(crate::capture_overlays::blocks_glide)
  {
    return false;
  }
  if settings.mouse_modifier.matches(key) && !settings.mouse_modifier.is_keyboard_modifier() {
    let active = session::active_input() == Some(InputKind::Mouse);
    let mut point = windows::Win32::Foundation::POINT::default();
    let over_titlebar =
      unsafe { windows::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut point) }.is_ok()
        && titlebar::cached_window_at(point).is_some();
    return active || over_titlebar;
  }
  let mut point = windows::Win32::Foundation::POINT::default();
  let over_titlebar = unsafe { windows::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut point) }
    .is_ok()
    && titlebar::cached_window_at(point).is_some();
  let monitor_reserved = settings.monitors_modifier.matches(key)
    && !settings.monitors_modifier.is_keyboard_modifier()
    && (session::active_input().is_some() || over_titlebar);
  let spaces_reserved = settings.spaces_modifier.matches(key)
    && !settings.spaces_modifier.is_keyboard_modifier()
    && (super::spaces::active_input().is_some() || (super::spaces::supported() && over_titlebar));
  let reserved_control = monitor_reserved || spaces_reserved;
  if reserved_control {
    return true;
  }
  settings.thirds_modifier.matches(key)
    && !settings.thirds_modifier.is_keyboard_modifier()
    && session::active_input().is_some()
}

#[cfg(test)]
mod tests {
  use super::Reservation;

  #[test]
  fn reserves_multiple_keys_and_repeats_after_eligibility_loss() {
    let mut reservation = Reservation::default();
    assert!(reservation.transition(1, true, true, true));
    assert!(reservation.transition(2, true, true, true));
    assert!(reservation.transition(1, true, false, false));
    assert!(reservation.transition(1, false, false, false));
    assert!(!reservation.transition(1, false, false, false));
    assert!(reservation.transition(2, false, false, false));
  }

  #[test]
  fn unposted_or_unpaired_events_pass_through() {
    let mut reservation = Reservation::default();
    assert!(!reservation.transition(1, true, true, false));
    assert!(!reservation.transition(1, false, false, false));
  }
}
