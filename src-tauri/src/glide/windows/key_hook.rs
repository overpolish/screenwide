// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::atomic::{AtomicIsize, Ordering};

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
        let configured = configured(packet.vkCode);
        if configured {
          forward(packet.vkCode, pressed);
        }
        if suppresses(packet.vkCode) {
          return LRESULT(1);
        }
      }
    }
  }
  unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

pub(super) fn configured(key: u32) -> bool {
  let settings = native_settings::snapshot();
  settings.mouse_modifier.matches(key) || settings.thirds_modifier.matches(key)
}

pub(super) fn forward(key: u32, pressed: bool) {
  let target = TARGET.load(Ordering::Acquire);
  if target != 0 {
    let _ = unsafe {
      PostMessageW(
        Some(HWND(target as *mut std::ffi::c_void)),
        WM_GLIDE_KEY,
        WPARAM(key as usize),
        LPARAM(isize::from(pressed)),
      )
    };
  }
}

pub(super) fn suppresses(key: u32) -> bool {
  let settings = native_settings::snapshot();
  if !settings.enabled || crate::shortcuts::is_capturing() {
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
  settings.thirds_modifier.matches(key)
    && !settings.thirds_modifier.is_keyboard_modifier()
    && session::active_input().is_some()
}
