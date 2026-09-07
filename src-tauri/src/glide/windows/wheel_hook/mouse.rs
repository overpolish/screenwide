// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use windows::Win32::UI::WindowsAndMessaging::{
  WM_MBUTTONDOWN, WM_MBUTTONUP, WM_XBUTTONDOWN, WM_XBUTTONUP,
};

pub(super) fn set_mouse_point(point: Option<(i32, i32)>) {
  if let Ok(mut last) = super::LAST_MOUSE_POINT.lock() {
    *last = point;
  }
}

pub(super) fn mouse_button(message: u32, data: u32) -> Option<(u32, bool)> {
  use crate::glide::platform::control::{MOUSE_BACK, MOUSE_FORWARD, MOUSE_MIDDLE};
  match message {
    WM_MBUTTONDOWN => Some((MOUSE_MIDDLE, true)),
    WM_MBUTTONUP => Some((MOUSE_MIDDLE, false)),
    WM_XBUTTONDOWN | WM_XBUTTONUP => {
      let button = if data >> 16 == 1 {
        MOUSE_BACK
      } else {
        MOUSE_FORWARD
      };
      Some((button, message == WM_XBUTTONDOWN))
    }
    _ => None,
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::glide::platform::control::{MOUSE_BACK, MOUSE_FORWARD, MOUSE_MIDDLE};

  #[test]
  fn maps_middle_button_transitions() {
    assert_eq!(mouse_button(WM_MBUTTONDOWN, 0), Some((MOUSE_MIDDLE, true)));
    assert_eq!(mouse_button(WM_MBUTTONUP, 0), Some((MOUSE_MIDDLE, false)));
  }

  #[test]
  fn maps_xbutton_transitions() {
    assert_eq!(
      mouse_button(WM_XBUTTONDOWN, 1 << 16),
      Some((MOUSE_BACK, true))
    );
    assert_eq!(
      mouse_button(WM_XBUTTONUP, 2 << 16),
      Some((MOUSE_FORWARD, false))
    );
  }
}
