// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::{mpsc, Mutex, OnceLock};

use tauri::ipc::Channel;
use windows::{
  core::{w, PCWSTR},
  Win32::{
    Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, WPARAM},
    System::{LibraryLoader::GetModuleHandleW, Threading::GetCurrentThreadId},
    UI::{
      Input::KeyboardAndMouse::{GetAsyncKeyState, VK_ESCAPE, VK_MENU, VK_SHIFT},
      Input::{
        GetRawInputData, RegisterRawInputDevices, HRAWINPUT, MOUSE_MOVE_ABSOLUTE, RAWINPUT,
        RAWINPUTDEVICE, RAWINPUTHEADER, RIDEV_INPUTSINK, RIDEV_REMOVE, RID_INPUT, RIM_TYPEKEYBOARD,
        RIM_TYPEMOUSE,
      },
      WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetCursorPos,
        GetMessageW, PostThreadMessageW, RegisterClassW, SetCursorPos, TranslateMessage,
        CS_NOCLOSE, HMENU, HWND_MESSAGE, MSG, RI_MOUSE_LEFT_BUTTON_UP, WINDOW_EX_STYLE,
        WINDOW_STYLE, WM_DESTROY, WM_INPUT, WM_KEYDOWN, WM_QUIT, WM_SYSKEYDOWN, WNDCLASSW,
      },
    },
  },
};

use super::CursorScrubEvent;

struct ActiveScrub {
  anchor: POINT,
  events: mpsc::Sender<CursorScrubEvent>,
  thread_id: u32,
}

static ACTIVE_SCRUB: Mutex<Option<ActiveScrub>> = Mutex::new(None);
static WINDOW_CLASS: OnceLock<u16> = OnceLock::new();

pub(super) fn begin(channel: Channel<CursorScrubEvent>) -> Result<(), String> {
  if ACTIVE_SCRUB
    .lock()
    .map_err(|_| "The cursor scrub state is unavailable".to_owned())?
    .is_some()
  {
    return Err("A cursor scrub is already active".to_owned());
  }

  let mut anchor = POINT::default();
  unsafe { GetCursorPos(&mut anchor) }
    .map_err(|error| format!("Could not read the cursor position: {error}"))?;
  let (event_tx, event_rx) = mpsc::channel();
  std::thread::Builder::new()
    .name("cursor-scrub-events".to_owned())
    .spawn(move || {
      for event in event_rx {
        if channel.send(event).is_err() {
          break;
        }
      }
    })
    .map_err(|error| format!("Could not start the cursor scrub event stream: {error}"))?;

  let (ready_tx, ready_rx) = mpsc::sync_channel(1);
  std::thread::Builder::new()
    .name("cursor-scrub-input".to_owned())
    .spawn(move || run_input_monitor(ready_tx))
    .map_err(|error| format!("Could not start the raw input monitor: {error}"))?;
  let thread_id = ready_rx
    .recv()
    .map_err(|_| "The raw input monitor stopped before it was ready".to_owned())??;

  *ACTIVE_SCRUB
    .lock()
    .map_err(|_| "The cursor scrub state is unavailable".to_owned())? = Some(ActiveScrub {
    anchor,
    events: event_tx,
    thread_id,
  });
  Ok(())
}

pub(super) fn end(offset_x: f64) -> Result<(), String> {
  let scrub = ACTIVE_SCRUB
    .lock()
    .map_err(|_| "The cursor scrub state is unavailable".to_owned())?
    .take();
  if let Some(scrub) = scrub {
    let _ = unsafe { PostThreadMessageW(scrub.thread_id, WM_QUIT, WPARAM(0), LPARAM(0)) };
    unsafe { SetCursorPos(scrub.anchor.x + offset_x.round() as i32, scrub.anchor.y) }
      .map_err(|error| format!("Could not restore the cursor position: {error}"))?;
  }
  Ok(())
}

fn run_input_monitor(ready: mpsc::SyncSender<Result<u32, String>>) {
  let instance = match unsafe { GetModuleHandleW(None) } {
    Ok(instance) => HINSTANCE(instance.0),
    Err(error) => {
      let _ = ready.send(Err(format!(
        "Could not locate the application module: {error}"
      )));
      return;
    }
  };
  let atom = *WINDOW_CLASS.get_or_init(|| unsafe {
    RegisterClassW(&WNDCLASSW {
      style: CS_NOCLOSE,
      lpfnWndProc: Some(window_proc),
      hInstance: instance,
      lpszClassName: w!("ScreenwideCursorScrubInput"),
      ..Default::default()
    })
  });
  if atom == 0 {
    let _ = ready.send(Err("Could not register the raw input window".to_owned()));
    return;
  }
  let window = match unsafe {
    CreateWindowExW(
      WINDOW_EX_STYLE::default(),
      w!("ScreenwideCursorScrubInput"),
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
  } {
    Ok(window) => window,
    Err(error) => {
      let _ = ready.send(Err(format!(
        "Could not create the raw input window: {error}"
      )));
      return;
    }
  };
  let devices = [
    raw_input_device(0x02, RIDEV_INPUTSINK, window),
    raw_input_device(0x06, RIDEV_INPUTSINK, window),
  ];
  if let Err(error) =
    unsafe { RegisterRawInputDevices(&devices, std::mem::size_of::<RAWINPUTDEVICE>() as u32) }
  {
    let _ = unsafe { DestroyWindow(window) };
    let _ = ready.send(Err(format!("Could not register for raw input: {error}")));
    return;
  }

  let _ = ready.send(Ok(unsafe { GetCurrentThreadId() }));
  let mut message = MSG::default();
  while unsafe { GetMessageW(&mut message, None, 0, 0) }.as_bool() {
    unsafe {
      let _ = TranslateMessage(&message);
      DispatchMessageW(&message);
    }
  }

  let remove = [
    raw_input_device(0x02, RIDEV_REMOVE, HWND::default()),
    raw_input_device(0x06, RIDEV_REMOVE, HWND::default()),
  ];
  let _ = unsafe { RegisterRawInputDevices(&remove, std::mem::size_of::<RAWINPUTDEVICE>() as u32) };
  let _ = unsafe { DestroyWindow(window) };
}

const fn raw_input_device(
  usage: u16,
  flags: windows::Win32::UI::Input::RAWINPUTDEVICE_FLAGS,
  target: HWND,
) -> RAWINPUTDEVICE {
  RAWINPUTDEVICE {
    usUsagePage: 0x01,
    usUsage: usage,
    dwFlags: flags,
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
    WM_DESTROY => LRESULT(0),
    _ => unsafe { DefWindowProcW(window, message, wparam, lparam) },
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
    let mouse = unsafe { input.data.mouse };
    let button_flags = unsafe { mouse.Anonymous.Anonymous.usButtonFlags };
    if u32::from(button_flags) & RI_MOUSE_LEFT_BUTTON_UP != 0 {
      finish_from_input();
      return;
    }
    if mouse.usFlags.0 & MOUSE_MOVE_ABSOLUTE.0 == 0 && (mouse.lLastX != 0 || mouse.lLastY != 0) {
      emit_movement(mouse.lLastX, mouse.lLastY);
    }
  } else if input.header.dwType == RIM_TYPEKEYBOARD.0 {
    let keyboard = unsafe { input.data.keyboard };
    if u32::from(keyboard.VKey) == u32::from(VK_ESCAPE.0)
      && (keyboard.Message == WM_KEYDOWN || keyboard.Message == WM_SYSKEYDOWN)
    {
      finish_from_input();
    }
  }
}

fn emit_movement(delta_x: i32, delta_y: i32) {
  let Ok(active) = ACTIVE_SCRUB.lock() else {
    return;
  };
  let Some(scrub) = active.as_ref() else {
    return;
  };
  let _ = scrub.events.send(CursorScrubEvent::Move {
    alt_key: unsafe { GetAsyncKeyState(VK_MENU.0 as i32) } < 0,
    delta_x,
    delta_y,
    shift_key: unsafe { GetAsyncKeyState(VK_SHIFT.0 as i32) } < 0,
  });
  let anchor = scrub.anchor;
  drop(active);
  let _ = unsafe { SetCursorPos(anchor.x, anchor.y) };
}

fn finish_from_input() {
  let Ok(active) = ACTIVE_SCRUB.lock() else {
    return;
  };
  let Some(scrub) = active.as_ref() else {
    return;
  };
  let _ = scrub.events.send(CursorScrubEvent::End);
  let anchor = scrub.anchor;
  let thread_id = scrub.thread_id;
  drop(active);
  let _ = unsafe { SetCursorPos(anchor.x, anchor.y) };
  unsafe {
    let _ = PostThreadMessageW(thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
  }
}
