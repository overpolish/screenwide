// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::OnceLock;

use tauri::AppHandle;
use windows::Win32::UI::{
  Input::KeyboardAndMouse::VK_ESCAPE,
  Input::{
    GetRawInputData, HRAWINPUT, MOUSE_MOVE_ABSOLUTE, RAWINPUT, RAWINPUTHEADER, RID_INPUT,
    RIM_TYPEKEYBOARD, RIM_TYPEMOUSE,
  },
  WindowsAndMessaging::{
    SetCursorPos, RI_MOUSE_HWHEEL, RI_MOUSE_WHEEL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
  },
};

#[path = "windows/cursor.rs"]
mod cursor;
#[path = "windows/input_window.rs"]
mod input_window;
#[path = "windows/native_settings.rs"]
mod native_settings;
#[path = "windows/session.rs"]
mod session;
#[path = "windows/target.rs"]
mod target;
#[path = "windows/titlebar.rs"]
mod titlebar;

use session::InputKind;

static APP: OnceLock<AppHandle> = OnceLock::new();

pub(super) fn start(app: AppHandle) -> Result<(), String> {
  let _ = APP.set(app.clone());
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
  let mut input = RAWINPUT::default();
  let mut size = std::mem::size_of::<RAWINPUT>() as u32;
  let read = unsafe {
    GetRawInputData(
      handle,
      RID_INPUT,
      Some(std::ptr::from_mut(&mut input).cast()),
      &mut size,
      std::mem::size_of::<RAWINPUTHEADER>() as u32,
    )
  };
  if read == u32::MAX || read < std::mem::size_of::<RAWINPUTHEADER>() as u32 {
    return;
  }

  if input.header.dwType == RIM_TYPEMOUSE.0 {
    handle_mouse(unsafe { input.data.mouse });
  } else if input.header.dwType == RIM_TYPEKEYBOARD.0 {
    let keyboard = unsafe { input.data.keyboard };
    let key = u32::from(keyboard.VKey);
    let pressed = matches!(keyboard.Message, WM_KEYDOWN | WM_SYSKEYDOWN);
    let released = matches!(keyboard.Message, WM_KEYUP | WM_SYSKEYUP);
    if key == u32::from(VK_ESCAPE.0) && pressed {
      finish_current_session(true);
      return;
    }
    let settings = native_settings::snapshot();
    if session::active_input() == Some(InputKind::Mouse)
      && released
      && native_settings::matches(settings.mouse_modifier, key)
      && !native_settings::is_down(settings.mouse_modifier)
    {
      finish_current_session(false);
      return;
    }
    if session::active_input().is_some() && native_settings::matches(settings.thirds_modifier, key)
    {
      if let Some(app) = APP.get() {
        session::set_thirds(app, native_settings::is_down(settings.thirds_modifier));
      }
    }
  }
}

fn handle_mouse(mouse: windows::Win32::UI::Input::RAWMOUSE) {
  let button_flags = u32::from(unsafe { mouse.Anonymous.Anonymous.usButtonFlags });
  let button_data = unsafe { mouse.Anonymous.Anonymous.usButtonData } as i16;
  let wheel_x = if button_flags & RI_MOUSE_HWHEEL != 0 {
    f64::from(button_data)
  } else {
    0.0
  };
  let wheel_y = if button_flags & RI_MOUSE_WHEEL != 0 {
    -f64::from(button_data)
  } else {
    0.0
  };
  if wheel_x != 0.0 || wheel_y != 0.0 {
    handle_trackpad_delta(wheel_x, wheel_y);
    return;
  }

  let relative = mouse.usFlags.0 & MOUSE_MOVE_ABSOLUTE.0 == 0;
  if relative && (mouse.lLastX != 0 || mouse.lLastY != 0) {
    handle_mouse_delta(mouse.lLastX, mouse.lLastY);
  }
}

fn handle_trackpad_delta(delta_x: f64, delta_y: f64) {
  if session::active_input().is_none() && delta_x == 0.0 {
    return;
  }
  if session::active_input().is_none() && !begin_session(InputKind::Trackpad) {
    return;
  }
  if session::active_input() != Some(InputKind::Trackpad) {
    return;
  }
  if let Some(app) = APP.get() {
    let settings = native_settings::snapshot();
    session::update(
      app,
      delta_x,
      delta_y,
      native_settings::is_down(settings.thirds_modifier),
    );
  }
}

fn handle_mouse_delta(delta_x: i32, delta_y: i32) {
  let settings = native_settings::snapshot();
  let modifier_down = native_settings::is_down(settings.mouse_modifier);
  if session::active_input() == Some(InputKind::Mouse) && !modifier_down {
    finish_current_session(false);
    return;
  }
  if session::active_input().is_none() && (!modifier_down || !begin_session(InputKind::Mouse)) {
    return;
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
    session::tick(app);
  }
}

fn finish_current_session(cancelled: bool) {
  if let Some(app) = APP.get() {
    session::end(app, cancelled);
  }
}

pub(super) fn end_session(app: &AppHandle) {
  session::end(app, true);
}
