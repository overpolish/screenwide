// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Windows' half: a menu of this process is open between its popup's start
//! and end events, `EndMenu` closes it, and each display's picture is a
//! topmost layered popup the pointer passes through, left in captures.

use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::OnceLock;

use windows::core::{w, PCWSTR};
use windows::Win32::{
  Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, SIZE, WPARAM},
  Graphics::Gdi::{
    CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetDC, ReleaseDC, SelectObject,
    AC_SRC_ALPHA, AC_SRC_OVER, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, BLENDFUNCTION, DIB_RGB_COLORS,
  },
  System::Threading::GetCurrentProcessId,
  UI::{
    Accessibility::{SetWinEventHook, HWINEVENTHOOK},
    WindowsAndMessaging::{
      CreateWindowExW, DefWindowProcW, DestroyWindow, EndMenu, SetWindowPos, UpdateLayeredWindow,
      EVENT_SYSTEM_MENUPOPUPEND, EVENT_SYSTEM_MENUPOPUPSTART, HMENU, HWND_TOPMOST, SWP_NOACTIVATE,
      SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW, ULW_ALPHA, WINEVENT_OUTOFCONTEXT, WNDCLASS_STYLES,
      WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT,
      WS_POPUP,
    },
  },
};

use super::Picture;
use crate::app_windows::overlay_surface;

const CLASS: PCWSTR = w!("ScreenwideMenuFreeze");
static CLASS_ATOM: OnceLock<u16> = OnceLock::new();
static HOOKED: AtomicBool = AtomicBool::new(false);
/// Popups open, a submenu counting as one more.
static OPEN_POPUPS: AtomicI32 = AtomicI32::new(0);

thread_local! {
  static BACKDROPS: RefCell<Vec<HWND>> = const { RefCell::new(Vec::new()) };
}

/// On the main thread: an out-of-context hook reports on the installing
/// thread's message loop, which a menu's own loop keeps pumping.
pub(super) fn observe(_app: &tauri::AppHandle) {
  if HOOKED.swap(true, Ordering::AcqRel) {
    return;
  }
  let hook = unsafe {
    SetWinEventHook(
      EVENT_SYSTEM_MENUPOPUPSTART,
      EVENT_SYSTEM_MENUPOPUPEND,
      None,
      Some(on_menu),
      GetCurrentProcessId(),
      0,
      WINEVENT_OUTOFCONTEXT,
    )
  };
  if hook.is_invalid() {
    HOOKED.store(false, Ordering::Release);
    eprintln!(
      "Could not watch for menus opening: {}",
      std::io::Error::last_os_error()
    );
  }
}

unsafe extern "system" fn on_menu(
  _hook: HWINEVENTHOOK,
  event: u32,
  _window: HWND,
  _object: i32,
  _child: i32,
  _thread: u32,
  _time: u32,
) {
  if event == EVENT_SYSTEM_MENUPOPUPSTART {
    OPEN_POPUPS.fetch_add(1, Ordering::AcqRel);
  } else if event == EVENT_SYSTEM_MENUPOPUPEND {
    let _ = OPEN_POPUPS.try_update(Ordering::AcqRel, Ordering::Acquire, |open| {
      Some((open - 1).max(0))
    });
  }
}

pub(super) fn menu_open() -> bool {
  OPEN_POPUPS.load(Ordering::Acquire) > 0
}

/// On the main thread, which owns the menu.
pub(super) fn close_menu() {
  if let Err(error) = unsafe { EndMenu() } {
    eprintln!("Could not close the menu: {error}");
  }
}

/// On the main thread.
pub(super) fn show(pictures: &[Picture]) {
  CLASS_ATOM
    .get_or_init(|| overlay_surface::register_class(CLASS, Some(window_proc), WNDCLASS_STYLES(0)));
  for picture in pictures {
    match backdrop(picture) {
      Ok(window) => BACKDROPS.with(|backdrops| backdrops.borrow_mut().push(window)),
      Err(error) => eprintln!("Could not show the frozen menu: {error}"),
    }
  }
}

/// On the main thread.
pub(super) fn hide() {
  for window in BACKDROPS.with(|backdrops| std::mem::take(&mut *backdrops.borrow_mut())) {
    let _ = unsafe { DestroyWindow(window) };
  }
}

fn backdrop(picture: &Picture) -> Result<HWND, String> {
  let (width, height) = (picture.image.width as i32, picture.image.height as i32);
  let window = unsafe {
    CreateWindowExW(
      WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TRANSPARENT,
      CLASS,
      PCWSTR::null(),
      WS_POPUP,
      picture.position.x,
      picture.position.y,
      width,
      height,
      None,
      Some(HMENU::default()),
      None,
      None,
    )
  }
  .map_err(|error| error.to_string())?;
  if let Err(error) = paint(window, picture, width, height) {
    let _ = unsafe { DestroyWindow(window) };
    return Err(error);
  }
  unsafe {
    SetWindowPos(
      window,
      Some(HWND_TOPMOST),
      0,
      0,
      0,
      0,
      SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
    )
  }
  .map_err(|error| error.to_string())?;
  Ok(window)
}

/// Hands the still to the layered window as an opaque top-down BGRA bitmap.
fn paint(window: HWND, picture: &Picture, width: i32, height: i32) -> Result<(), String> {
  let info = BITMAPINFO {
    bmiHeader: BITMAPINFOHEADER {
      biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
      biWidth: width,
      biHeight: -height,
      biPlanes: 1,
      biBitCount: 32,
      biCompression: BI_RGB.0,
      ..Default::default()
    },
    ..Default::default()
  };
  let screen = unsafe { GetDC(None) };
  let memory = unsafe { CreateCompatibleDC(Some(screen)) };
  let mut bits = std::ptr::null_mut();
  let bitmap = unsafe {
    CreateDIBSection(
      Some(memory),
      &raw const info,
      DIB_RGB_COLORS,
      &mut bits,
      None,
      0,
    )
  };
  let result = bitmap
    .map_err(|error| error.to_string())
    .and_then(|bitmap| {
      let pixels =
        unsafe { std::slice::from_raw_parts_mut(bits.cast::<u8>(), picture.image.rgba.len()) };
      for (target, source) in pixels
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .zip(picture.image.rgba.as_chunks::<4>().0)
      {
        *target = [source[2], source[1], source[0], 255];
      }
      let previous = unsafe { SelectObject(memory, bitmap.into()) };
      let blend = BLENDFUNCTION {
        BlendOp: AC_SRC_OVER as u8,
        BlendFlags: 0,
        SourceConstantAlpha: 255,
        AlphaFormat: AC_SRC_ALPHA as u8,
      };
      let at = POINT {
        x: picture.position.x,
        y: picture.position.y,
      };
      let size = SIZE {
        cx: width,
        cy: height,
      };
      let origin = POINT { x: 0, y: 0 };
      let updated = unsafe {
        UpdateLayeredWindow(
          window,
          Some(screen),
          Some(&raw const at),
          Some(&raw const size),
          Some(memory),
          Some(&raw const origin),
          COLORREF(0),
          Some(&raw const blend),
          ULW_ALPHA,
        )
      }
      .map_err(|error| error.to_string());
      unsafe {
        SelectObject(memory, previous);
        let _ = DeleteObject(bitmap.into());
      }
      updated
    });
  unsafe {
    let _ = DeleteDC(memory);
    ReleaseDC(None, screen);
  }
  result
}

unsafe extern "system" fn window_proc(
  window: HWND,
  message: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  unsafe { DefWindowProcW(window, message, wparam, lparam) }
}
