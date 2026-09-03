// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Dynamically probes the Windows 11 Precision Touchpad opt-in API. The
//! function is newer than the SDK used by the current windows crate.

use windows::{
  core::{w, BOOL, PCSTR},
  Win32::{
    Foundation::HWND,
    System::LibraryLoader::{GetModuleHandleW, GetProcAddress},
  },
};

type RegisterTouchpadCapableWindow = unsafe extern "system" fn(HWND, BOOL) -> BOOL;

pub(super) struct Registration {
  window: HWND,
  unregister: RegisterTouchpadCapableWindow,
}

impl Drop for Registration {
  fn drop(&mut self) {
    let _ = unsafe { (self.unregister)(self.window, false.into()) };
  }
}

pub(super) fn register(window: HWND) -> Option<Registration> {
  let module = unsafe { GetModuleHandleW(w!("user32.dll")) }.ok()?;
  let address = (unsafe {
    GetProcAddress(
      module,
      PCSTR(c"RegisterTouchpadCapableWindow".as_ptr().cast()),
    )
  })?;
  let register: RegisterTouchpadCapableWindow = unsafe { std::mem::transmute(address) };
  if unsafe { register(window, true.into()) }.as_bool() {
    Some(Registration {
      window,
      unregister: register,
    })
  } else {
    None
  }
}
