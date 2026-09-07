// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
  collections::HashSet,
  sync::{
    atomic::{AtomicIsize, Ordering},
    LazyLock, Mutex,
  },
};

use windows::Win32::{
  Foundation::{HWND, LPARAM, LRESULT, WPARAM},
  UI::WindowsAndMessaging::{
    CallNextHookEx, PostMessageW, SetWindowsHookExW, UnhookWindowsHookEx, HC_ACTION, HHOOK,
    KBDLLHOOKSTRUCT, LLKHF_INJECTED, WH_KEYBOARD_LL, WM_APP, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN,
    WM_SYSKEYUP,
  },
};

use super::{native_settings, session, titlebar, InputKind};

pub(super) const WM_GLIDE_KEY: u32 = WM_APP + 3;
static TARGET: AtomicIsize = AtomicIsize::new(0);
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

pub(super) struct KeyHook(HHOOK);

impl Drop for KeyHook {
  fn drop(&mut self) {
    let _ = unsafe { UnhookWindowsHookEx(self.0) };
    TARGET.store(0, Ordering::Release);
  }
}

pub(super) fn install(target: HWND) -> Result<KeyHook, String> {
  TARGET.store(target.0 as isize, Ordering::Release);
  match unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(callback), None, 0) } {
    Ok(hook) => Ok(KeyHook(hook)),
    Err(error) => {
      TARGET.store(0, Ordering::Release);
      Err(error.to_string())
    }
  }
}

unsafe extern "system" fn callback(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
  if code == HC_ACTION as i32 {
    let packet = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
    if !packet.flags.contains(LLKHF_INJECTED) {
      let message = wparam.0 as u32;
      let pressed = matches!(message, WM_KEYDOWN | WM_SYSKEYDOWN);
      let released = matches!(message, WM_KEYUP | WM_SYSKEYUP);
      if pressed || released {
        native_settings::observe(packet.vkCode, pressed);
        super::release_tracker::hook(packet.vkCode, pressed, packet.time);
        let configured = configured(packet.vkCode);
        let escaped = packet.vkCode
          == u32::from(windows::Win32::UI::Input::KeyboardAndMouse::VK_ESCAPE.0)
          && (session::active_input().is_some() || super::spaces::active_input().is_some())
          && !configured;
        let eligible = suppresses(packet.vkCode) || escaped;
        let posted = (configured || escaped) && forward(packet.vkCode, pressed);
        if reserve(packet.vkCode, pressed, eligible, posted) {
          return LRESULT(1);
        }
      }
    }
  }
  unsafe { CallNextHookEx(None, code, wparam, lparam) }
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
