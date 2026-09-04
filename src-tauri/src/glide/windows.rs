// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::{
  atomic::{AtomicBool, Ordering},
  Mutex, OnceLock,
};

use tauri::AppHandle;
use windows::Win32::UI::{
  Input::{
    GetRawInputData, HRAWINPUT, MOUSE_MOVE_ABSOLUTE, MOUSE_VIRTUAL_DESKTOP, RAWINPUT,
    RAWINPUTHEADER, RAWMOUSE, RID_INPUT, RIM_TYPEHID, RIM_TYPEKEYBOARD, RIM_TYPEMOUSE,
  },
  WindowsAndMessaging::{
    GetSystemMetrics, SetCursorPos, RI_MOUSE_HWHEEL, RI_MOUSE_WHEEL, SM_CXSCREEN,
    SM_CXVIRTUALSCREEN, SM_CYSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
  },
};

#[path = "windows/control.rs"]
mod control;
#[path = "windows/cursor.rs"]
mod cursor;
#[path = "windows/input_kind.rs"]
mod input_kind;
#[path = "windows/input_window.rs"]
mod input_window;
#[path = "windows/key.rs"]
mod key;
#[path = "windows/key_hook.rs"]
mod key_hook;
#[path = "windows/keyboard.rs"]
mod keyboard;
#[path = "windows/native_settings.rs"]
mod native_settings;
#[path = "windows/native_trackpad.rs"]
mod native_trackpad;
#[path = "windows/precision_touchpad.rs"]
mod precision_touchpad;
#[path = "windows/session.rs"]
mod session;
#[path = "windows/target.rs"]
mod target;
#[path = "windows/titlebar.rs"]
mod titlebar;
#[path = "windows/trackpad.rs"]
mod trackpad;
#[path = "windows/tween.rs"]
mod tween;
#[path = "windows/wheel_hook.rs"]
mod wheel_hook;

use input_kind::InputKind;

static APP: OnceLock<AppHandle> = OnceLock::new();
/// Real pointer travel during a trackpad session commits the gesture.
const POINTER_DISMISS_DISTANCE: f64 = 3.0;
const ABSOLUTE_RANGE: f64 = 65_535.0;
static WHEEL_HOOK_ACTIVE: AtomicBool = AtomicBool::new(false);
static LAST_ABSOLUTE: Mutex<Option<(i32, i32)>> = Mutex::new(None);

pub(super) fn start(app: AppHandle) -> Result<(), String> {
  let _ = APP.set(app.clone());
  titlebar::cache_own_windows(&app);
  tween::start();
  let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel(1);
  std::thread::Builder::new()
    .name("glide-input".to_owned())
    .spawn(move || input_window::run(app, ready_tx))
    .map_err(|error| format!("Could not start Glide input monitoring: {error}"))?;
  ready_rx
    .recv()
    .map_err(|_| "Glide input monitoring stopped before it was ready".to_owned())?
}

pub(super) fn apply_settings(settings: &crate::glide::settings::GlideSettings) {
  native_settings::apply(settings);
  if !settings.enabled {
    finish_current_session(true);
  }
}

fn handle_raw_input(handle: HRAWINPUT) {
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
    keyboard::handle(unsafe { input.data.keyboard });
  } else if input.header.dwType == RIM_TYPEHID.0 {
    let hid = unsafe { &input.data.hid };
    native_trackpad::handle_raw_input(&input.header, hid, read as usize);
  }
}

fn handle_mouse(mouse: RAWMOUSE) {
  let button_flags = u32::from(unsafe { mouse.Anonymous.Anonymous.usButtonFlags });
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
  let settings = native_settings::snapshot();
  if settings.mouse_modifier.is_mouse_button()
    && (session::active_input() == Some(InputKind::Mouse)
      || native_settings::is_down(settings.mouse_modifier))
  {
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

pub(super) fn set_wheel_hook_active(active: bool) {
  WHEEL_HOOK_ACTIVE.store(active, Ordering::Relaxed);
}

fn handle_mouse_delta(delta_x: i32, delta_y: i32) {
  if session::active_input().is_some_and(InputKind::is_trackpad) {
    let distance = f64::from(delta_x.unsigned_abs()) + f64::from(delta_y.unsigned_abs());
    if session::accumulate_pointer_travel(distance) >= POINTER_DISMISS_DISTANCE {
      if let Some(app) = APP.get() {
        session::end(app, false);
      }
    }
    return;
  }
  let settings = native_settings::snapshot();
  let modifier_down = native_settings::is_down(settings.mouse_modifier);
  if native_trackpad::blocks_mouse_glide(modifier_down) {
    finish_current_session(true);
    return;
  }
  if session::active_input() == Some(InputKind::Mouse) && !modifier_down {
    finish_current_session(false);
    return;
  }
  if session::active_input().is_none() {
    if !modifier_down {
      return;
    }
    if !begin_session(InputKind::Mouse) {
      return;
    }
  }
  if session::active_input() != Some(InputKind::Mouse) {
    return;
  }
  if let Some(anchor) = session::anchor() {
    let _ = unsafe { SetCursorPos(anchor.x, anchor.y) };
  }
  if let Some(app) = APP.get() {
    session::update(
      app,
      f64::from(delta_x),
      f64::from(delta_y),
      native_settings::is_down(settings.thirds_modifier),
    );
  }
}

fn begin_session(input: InputKind) -> bool {
  let Some(app) = APP.get() else {
    return false;
  };
  session::begin(app, input)
}

pub(super) fn tick() {
  if let Some(app) = APP.get() {
    let settings = native_settings::snapshot();
    if mouse_session_released(
      session::active_input(),
      native_settings::is_down(settings.mouse_modifier),
    ) {
      session::end(app, false);
      return;
    }
    if session::pointer_displacement().is_some_and(|distance| distance >= POINTER_DISMISS_DISTANCE)
    {
      session::end(app, false);
      return;
    }
    session::tick(app);
  }
}

fn mouse_session_released(active: Option<InputKind>, modifier_down: bool) -> bool {
  active == Some(InputKind::Mouse) && !modifier_down
}

fn finish_current_session(cancelled: bool) {
  if let Some(app) = APP.get() {
    session::end(app, cancelled);
  }
}

pub(super) fn end_session(app: &AppHandle) {
  session::end(app, true);
}

pub(super) fn suspend_for_capture(app: &AppHandle) {
  session::end(app, true);
}

pub(super) fn supports_control(control: crate::glide::settings::GlideControl) -> bool {
  control::NativeControl::from_control(control).is_some()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn tick_ends_only_a_mouse_session_whose_control_was_released() {
    assert!(mouse_session_released(Some(InputKind::Mouse), false));
    assert!(!mouse_session_released(Some(InputKind::Mouse), true));
    assert!(!mouse_session_released(
      Some(InputKind::TrackpadContacts),
      false
    ));
    assert!(!mouse_session_released(None, false));
  }
}
