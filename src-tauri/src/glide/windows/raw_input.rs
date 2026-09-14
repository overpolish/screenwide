// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{
  handle_mouse_delta, key_hook, keyboard, native_trackpad, trackpad, wheel_hook, WHEEL_HOOK_ACTIVE,
};
use std::sync::atomic::Ordering;
use std::sync::Mutex;
use windows::Win32::UI::{
  Input::{
    GetRawInputData, HRAWINPUT, MOUSE_MOVE_ABSOLUTE, MOUSE_VIRTUAL_DESKTOP, RAWINPUT,
    RAWINPUTHEADER, RAWMOUSE, RID_INPUT, RIM_TYPEHID, RIM_TYPEKEYBOARD, RIM_TYPEMOUSE,
  },
  WindowsAndMessaging::{
    GetMessageTime, GetSystemMetrics, RI_MOUSE_BUTTON_3_UP, RI_MOUSE_BUTTON_4_UP,
    RI_MOUSE_BUTTON_5_UP, RI_MOUSE_HWHEEL, RI_MOUSE_WHEEL, SM_CXSCREEN, SM_CXVIRTUALSCREEN,
    SM_CYSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
  },
};
const ABSOLUTE_RANGE: f64 = 65_535.0;
static LAST_ABSOLUTE: Mutex<Option<(i32, i32)>> = Mutex::new(None);

pub(crate) fn handle_raw_input(handle: HRAWINPUT) {
  let header_size = std::mem::size_of::<RAWINPUTHEADER>() as u32;
  let mut size = 0;
  let query = unsafe { GetRawInputData(handle, RID_INPUT, None, &mut size, header_size) };
  if query == u32::MAX || size < header_size {
    return;
  }
  let mut buffer = vec![0_u8; size as usize];
  let read = unsafe {
    GetRawInputData(
      handle,
      RID_INPUT,
      Some(buffer.as_mut_ptr().cast()),
      &mut size,
      header_size,
    )
  };
  if read == u32::MAX || read < header_size {
    return;
  }
  let input = unsafe { &*buffer.as_ptr().cast::<RAWINPUT>() };

  if input.header.dwType == RIM_TYPEMOUSE.0 {
    let mouse = unsafe { input.data.mouse };
    handle_mouse(mouse);
  } else if input.header.dwType == RIM_TYPEKEYBOARD.0 {
    handle_keyboard(unsafe { input.data.keyboard });
  } else if input.header.dwType == RIM_TYPEHID.0 {
    let hid = unsafe { &input.data.hid };
    native_trackpad::handle_raw_input(&input.header, hid, read as usize);
  }
}

fn handle_keyboard(keyboard_input: windows::Win32::UI::Input::RAWKEYBOARD) {
  let key = normalize_key(
    keyboard_input.VKey,
    keyboard_input.MakeCode,
    keyboard_input.Flags,
  );
  let released = matches!(
    keyboard_input.Message,
    windows::Win32::UI::WindowsAndMessaging::WM_KEYUP
      | windows::Win32::UI::WindowsAndMessaging::WM_SYSKEYUP
  );
  if released {
    let timestamp = unsafe { windows::Win32::UI::WindowsAndMessaging::GetMessageTime() } as u32;
    if super::release_tracker::raw_release(key, timestamp) {
      super::native_settings::observe(key, false);
      key_hook::reserve(key, false, false, false);
      keyboard::handle_transition(key, false, true);
    }
  } else {
    let timestamp = unsafe { windows::Win32::UI::WindowsAndMessaging::GetMessageTime() } as u32;
    if super::release_tracker::raw_press(key, timestamp) {
      super::native_settings::observe(key, true);
      keyboard::handle_transition(key, true, false);
    }
  }
}

fn normalize_key(vkey: u16, make_code: u16, flags: u16) -> u32 {
  match vkey {
    0x10 => {
      if make_code == 0x36 {
        0xA1
      } else {
        0xA0
      }
    }
    0x11 => {
      if flags & 0x02 != 0 {
        0xA3
      } else {
        0xA2
      }
    }
    0x12 => {
      if flags & 0x02 != 0 {
        0xA5
      } else {
        0xA4
      }
    }
    _ => {
      let _ = make_code;
      u32::from(vkey)
    }
  }
}

fn handle_mouse(mouse: RAWMOUSE) {
  let button_flags = u32::from(unsafe { mouse.Anonymous.Anonymous.usButtonFlags });
  let timestamp = unsafe { GetMessageTime() } as u32;
  for (flag, button) in [
    (RI_MOUSE_BUTTON_3_UP, super::control::MOUSE_MIDDLE),
    (RI_MOUSE_BUTTON_4_UP, super::control::MOUSE_BACK),
    (RI_MOUSE_BUTTON_5_UP, super::control::MOUSE_FORWARD),
  ] {
    if button_flags & flag != 0 && super::release_tracker::raw_release(button, timestamp) {
      super::native_settings::observe(button, false);
      key_hook::reserve(button, false, false, false);
      keyboard::handle_transition(button, false, true);
    }
  }
  let button_data = unsafe { mouse.Anonymous.Anonymous.usButtonData } as i16;
  let wheel_x = if button_flags & RI_MOUSE_HWHEEL != 0 {
    // Convert content direction to finger travel; session normalization below
    // handles the user's reversed-scroll preference.
    -f64::from(button_data)
  } else {
    0.0
  };
  let wheel_y = if button_flags & RI_MOUSE_WHEEL != 0 {
    f64::from(button_data)
  } else {
    0.0
  };
  if wheel_x != 0.0 || wheel_y != 0.0 {
    if !WHEEL_HOOK_ACTIVE.load(Ordering::Relaxed) {
      trackpad::handle_delta(wheel_x, wheel_y);
    }
    return;
  }
  if wheel_hook::hook_owns_mouse_motion() {
    return;
  }
  if mouse.usFlags.0 & MOUSE_MOVE_ABSOLUTE.0 == 0 {
    if mouse.lLastX != 0 || mouse.lLastY != 0 {
      handle_mouse_delta(mouse.lLastX, mouse.lLastY);
    }
    return;
  }
  // Remote and tablet drivers report position, so derive motion between reports.
  let position = absolute_position(mouse);
  let previous = LAST_ABSOLUTE
    .lock()
    .ok()
    .and_then(|mut last| last.replace(position));
  if let Some((x, y)) = previous {
    let (delta_x, delta_y) = (position.0 - x, position.1 - y);
    if delta_x != 0 || delta_y != 0 {
      handle_mouse_delta(delta_x, delta_y);
    }
  }
}

fn absolute_position(mouse: RAWMOUSE) -> (i32, i32) {
  let virtual_desktop = mouse.usFlags.0 & MOUSE_VIRTUAL_DESKTOP.0 != 0;
  let (left, top, width, height) = if virtual_desktop {
    unsafe {
      (
        GetSystemMetrics(SM_XVIRTUALSCREEN),
        GetSystemMetrics(SM_YVIRTUALSCREEN),
        GetSystemMetrics(SM_CXVIRTUALSCREEN),
        GetSystemMetrics(SM_CYVIRTUALSCREEN),
      )
    }
  } else {
    unsafe {
      (
        0,
        0,
        GetSystemMetrics(SM_CXSCREEN),
        GetSystemMetrics(SM_CYSCREEN),
      )
    }
  };
  (
    left + (f64::from(mouse.lLastX) * f64::from(width) / ABSOLUTE_RANGE).round() as i32,
    top + (f64::from(mouse.lLastY) * f64::from(height) / ABSOLUTE_RANGE).round() as i32,
  )
}

#[cfg(test)]
mod tests {
  use super::normalize_key;

  #[test]
  fn raw_modifier_normalization_uses_make_code_and_e0() {
    assert_eq!(normalize_key(0x10, 0x2a, 0), 0xa0);
    assert_eq!(normalize_key(0x10, 0x36, 0x01), 0xa1);
    assert_eq!(normalize_key(0x11, 0, 0), 0xa2);
    assert_eq!(normalize_key(0x11, 0, 0x02), 0xa3);
    assert_eq!(normalize_key(0x12, 0, 0), 0xa4);
    assert_eq!(normalize_key(0x12, 0, 0x02), 0xa5);
  }
}
