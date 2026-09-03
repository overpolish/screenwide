// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::OnceLock;

use tauri::AppHandle;
use windows::{
  core::{w, PCWSTR},
  Win32::{
    Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
    System::LibraryLoader::GetModuleHandleW,
    UI::{
      Input::{RegisterRawInputDevices, HRAWINPUT, RAWINPUTDEVICE, RIDEV_INPUTSINK},
      WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW, KillTimer,
        RegisterClassW, SetTimer, TranslateMessage, CS_NOCLOSE, HMENU, HWND_MESSAGE, MSG,
        WINDOW_EX_STYLE, WINDOW_STYLE, WM_DESTROY, WM_INPUT, WM_TIMER, WNDCLASSW,
      },
    },
  },
};

use super::{
  end_session, handle_raw_input, key_hook, precision_touchpad, set_wheel_hook_active, tick,
  trackpad, wheel_hook,
};

const TIMER_ID: usize = 1;
const USAGE_PAGE_GENERIC_DESKTOP: u16 = 0x01;
const USAGE_PAGE_DIGITIZER: u16 = 0x0d;
const USAGE_MOUSE: u16 = 0x02;
const USAGE_KEYBOARD: u16 = 0x06;
const USAGE_TOUCHPAD: u16 = 0x05;
static WINDOW_CLASS: OnceLock<u16> = OnceLock::new();

pub(super) fn run(app: AppHandle, ready: std::sync::mpsc::SyncSender<Result<(), String>>) {
  let instance = match unsafe { GetModuleHandleW(None) } {
    Ok(instance) => HINSTANCE(instance.0),
    Err(error) => {
      let _ = ready.send(Err(format!("Could not locate the app module: {error}")));
      return;
    }
  };
  let atom = *WINDOW_CLASS.get_or_init(|| unsafe {
    RegisterClassW(&WNDCLASSW {
      style: CS_NOCLOSE,
      lpfnWndProc: Some(window_proc),
      hInstance: instance,
      lpszClassName: w!("ScreenwideGlideInput"),
      ..Default::default()
    })
  });
  if atom == 0 {
    let _ = ready.send(Err("Could not register the Glide input window".to_owned()));
    return;
  }
  let window = match create_window(instance) {
    Ok(window) => window,
    Err(error) => {
      let _ = ready.send(Err(error));
      return;
    }
  };
  let devices = [
    raw_input_device(USAGE_PAGE_GENERIC_DESKTOP, USAGE_MOUSE, window),
    raw_input_device(USAGE_PAGE_GENERIC_DESKTOP, USAGE_KEYBOARD, window),
  ];
  if let Err(error) =
    unsafe { RegisterRawInputDevices(&devices, std::mem::size_of::<RAWINPUTDEVICE>() as u32) }
  {
    let _ = unsafe { DestroyWindow(window) };
    let _ = ready.send(Err(format!("Could not register Glide raw input: {error}")));
    return;
  }
  let touchpad = [raw_input_device(
    USAGE_PAGE_DIGITIZER,
    USAGE_TOUCHPAD,
    window,
  )];
  let _ =
    unsafe { RegisterRawInputDevices(&touchpad, std::mem::size_of::<RAWINPUTDEVICE>() as u32) };
  let precision_touchpad = precision_touchpad::register(window);
  let key_hook = key_hook::install(window)
    .map_err(|error| {
      eprintln!("Could not suppress Glide control keys: {error}");
      error
    })
    .ok();
  let wheel_hook = match wheel_hook::install(window) {
    Ok(hook) => {
      set_wheel_hook_active(true);
      Some(hook)
    }
    Err(_) => None,
  };
  let _ = unsafe { SetTimer(Some(window), TIMER_ID, 16, None) };
  let _ = ready.send(Ok(()));

  let mut message = MSG::default();
  while unsafe { GetMessageW(&mut message, None, 0, 0) }.as_bool() {
    unsafe {
      let _ = TranslateMessage(&message);
      DispatchMessageW(&message);
    }
  }
  end_session(&app);
  drop(precision_touchpad);
  drop(key_hook);
  set_wheel_hook_active(false);
  drop(wheel_hook);
  let _ = unsafe { KillTimer(Some(window), TIMER_ID) };
  let _ = unsafe { DestroyWindow(window) };
}

fn create_window(instance: HINSTANCE) -> Result<HWND, String> {
  unsafe {
    CreateWindowExW(
      WINDOW_EX_STYLE::default(),
      w!("ScreenwideGlideInput"),
      PCWSTR::null(),
      WINDOW_STYLE::default(),
      0,
      0,
      0,
      0,
      Some(HWND_MESSAGE),
      Some(HMENU::default()),
      Some(instance),
      None,
    )
  }
  .map_err(|error| format!("Could not create the Glide input window: {error}"))
}

const fn raw_input_device(page: u16, usage: u16, target: HWND) -> RAWINPUTDEVICE {
  RAWINPUTDEVICE {
    usUsagePage: page,
    usUsage: usage,
    dwFlags: RIDEV_INPUTSINK,
    hwndTarget: target,
  }
}

unsafe extern "system" fn window_proc(
  window: HWND,
  message: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  match message {
    WM_INPUT => {
      handle_raw_input(HRAWINPUT(lparam.0 as *mut std::ffi::c_void));
      unsafe { DefWindowProcW(window, message, wparam, lparam) }
    }
    WM_TIMER => {
      tick();
      LRESULT(0)
    }
    wheel_hook::WM_GLIDE_WHEEL_X => {
      trackpad::handle_legacy_wheel(true, lparam.0 as i16);
      LRESULT(0)
    }
    wheel_hook::WM_GLIDE_WHEEL_Y => {
      trackpad::handle_legacy_wheel(false, lparam.0 as i16);
      LRESULT(0)
    }
    wheel_hook::WM_GLIDE_MOUSE_MOVE => {
      super::handle_mouse_delta(wparam.0 as u32 as i32, lparam.0 as i32);
      LRESULT(0)
    }
    key_hook::WM_GLIDE_KEY => {
      super::keyboard::handle_transition(wparam.0 as u32, lparam.0 != 0, lparam.0 == 0);
      LRESULT(0)
    }
    WM_DESTROY => LRESULT(0),
    _ => unsafe { DefWindowProcW(window, message, wparam, lparam) },
  }
}
