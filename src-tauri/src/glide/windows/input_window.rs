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
        RegisterClassW, SetTimer, TranslateMessage, CS_NOCLOSE, HMENU, MSG, WINDOW_EX_STYLE,
        WINDOW_STYLE, WM_DESTROY, WM_INPUT, WM_TIMER, WNDCLASSW,
      },
    },
  },
};

use super::{end_session, handle_raw_input, tick};

const TIMER_ID: usize = 1;
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
    raw_input_device(0x02, window),
    raw_input_device(0x06, window),
  ];
  if let Err(error) =
    unsafe { RegisterRawInputDevices(&devices, std::mem::size_of::<RAWINPUTDEVICE>() as u32) }
  {
    let _ = unsafe { DestroyWindow(window) };
    let _ = ready.send(Err(format!("Could not register Glide raw input: {error}")));
    return;
  }
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

const fn raw_input_device(usage: u16, target: HWND) -> RAWINPUTDEVICE {
  RAWINPUTDEVICE {
    usUsagePage: 0x01,
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
    WM_DESTROY => LRESULT(0),
    _ => unsafe { DefWindowProcW(window, message, wparam, lparam) },
  }
}
