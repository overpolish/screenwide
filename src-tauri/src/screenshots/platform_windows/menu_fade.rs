// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A clicked menu item fading away. Windows closes a menu as an item is
//! clicked but draws the item fading for a moment more, in no window of its
//! own, so a desktop frozen straight after a tray click would keep it.

use std::ffi::c_void;
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use windows::core::BOOL;
use windows::Win32::UI::WindowsAndMessaging::{
  SystemParametersInfoW, SPI_GETSELECTIONFADE, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
};

/// When a menu item was last clicked.
static CLICKED: Mutex<Option<Instant>> = Mutex::new(None);
/// How long after a click the desktop is clear of the item: it was gone
/// within 280 ms.
const FADE: Duration = Duration::from_millis(350);

/// Notes that a menu item was just clicked.
pub fn clicked() {
  *CLICKED.lock().unwrap_or_else(PoisonError::into_inner) = Some(Instant::now());
}

/// Waits for the item last clicked to have faded, if one was clicked just
/// now and Windows fades it.
pub(super) fn wait() {
  let clicked = CLICKED
    .lock()
    .unwrap_or_else(PoisonError::into_inner)
    .take();
  let Some(left) = clicked.and_then(|clicked| FADE.checked_sub(clicked.elapsed())) else {
    return;
  };
  if items_fade() {
    std::thread::sleep(left);
  }
}

/// Whether Windows fades a clicked menu item out, as it does by default.
fn items_fade() -> bool {
  let mut fades = BOOL::default();
  unsafe {
    SystemParametersInfoW(
      SPI_GETSELECTIONFADE,
      0,
      Some(&mut fades as *mut BOOL as *mut c_void),
      SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
    )
  }
  .is_ok()
    && fades.as_bool()
}
