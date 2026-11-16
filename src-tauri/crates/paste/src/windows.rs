// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Pasting on Windows. Text set on the clipboard needs a window to own it,
//! so a hidden one is made for the moment it is set; the text stays once the
//! window goes, as anything copied does.

mod focus;
mod guard;
mod keys;
mod target;

pub(super) use self::target::{bring_back, is_in_front, Target};

use std::time::Duration;

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{GlobalFree, HANDLE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::DataExchange::{
  CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
use windows::Win32::System::Ole::CF_UNICODETEXT;
use windows::Win32::UI::WindowsAndMessaging::{
  CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassW, HWND_MESSAGE, WINDOW_EX_STYLE,
  WINDOW_STYLE, WNDCLASSW,
};

use self::focus::Seen;
use self::keys::press_paste;
use super::{OnMain, Pasted, Refusal};

/// How long the wait sleeps between tries at a busy clipboard.
const POLL: Duration = Duration::from_millis(20);
/// How many times opening a busy clipboard is tried, `POLL` apart.
const OPEN_TRIES: u32 = 10;

pub(super) fn paste(text: &str, _on_main: OnMain<'_>) -> Result<Pasted, Refusal> {
  let focus = match focus::seen() {
    Seen::Password => return Err(Refusal::SecureField),
    Seen::Focus(focus) => focus,
  };
  let window = Window::create()?;
  with_clipboard(window.0, || copy(text))?;
  drop(window);
  if guard::is_elevated_above_us() {
    return Err(Refusal::Elevated);
  }
  if press_paste().is_err() {
    return Ok(Pasted::OnClipboard);
  }
  Ok(focus.pasted())
}

/// The hidden window that owns the clipboard while the text is set.
struct Window(HWND);

impl Window {
  fn create() -> Result<Self, Refusal> {
    let failed = |_| Refusal::Failed("Could not make the clipboard's window".to_owned());
    // Safety: the class is registered once per process, with a procedure
    // that lives as long as the program; registering again only fails.
    unsafe {
      let instance = GetModuleHandleW(PCWSTR::null()).map_err(failed)?;
      let class = w!("ScreenwidePaste");
      RegisterClassW(&WNDCLASSW {
        hInstance: instance.into(),
        lpfnWndProc: Some(procedure),
        lpszClassName: class,
        ..Default::default()
      });
      CreateWindowExW(
        WINDOW_EX_STYLE::default(),
        class,
        PCWSTR::null(),
        WINDOW_STYLE::default(),
        0,
        0,
        0,
        0,
        Some(HWND_MESSAGE),
        None,
        Some(instance.into()),
        None,
      )
      .map(Self)
      .map_err(failed)
    }
  }
}

impl Drop for Window {
  fn drop(&mut self) {
    // Safety: the window is this thread's own.
    let _ = unsafe { DestroyWindow(self.0) };
  }
}

/// The window answers nothing of its own.
extern "system" fn procedure(
  window: HWND,
  message: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  // Safety: every message is the default's.
  unsafe { DefWindowProcW(window, message, wparam, lparam) }
}

/// Runs `work` with the clipboard open for `window`, trying a few times
/// while another program has it open.
fn with_clipboard<T>(
  window: HWND,
  work: impl FnOnce() -> Result<T, Refusal>,
) -> Result<T, Refusal> {
  for _ in 0..OPEN_TRIES {
    // Safety: closed below on every path.
    if unsafe { OpenClipboard(Some(window)) }.is_ok() {
      let result = work();
      // Safety: opened above.
      let _ = unsafe { CloseClipboard() };
      return result;
    }
    std::thread::sleep(POLL);
  }
  Err(Refusal::Failed(
    "Another program is holding the clipboard".to_owned(),
  ))
}

/// Puts `text` on the open clipboard in place of what it held.
fn copy(text: &str) -> Result<(), Refusal> {
  // Safety: the clipboard is open.
  unsafe { EmptyClipboard() }
    .map_err(|_| Refusal::Failed("Could not empty the clipboard".to_owned()))?;
  let bytes: Vec<u8> = text
    .encode_utf16()
    .chain([0])
    .flat_map(u16::to_ne_bytes)
    .collect();
  if set(u32::from(CF_UNICODETEXT.0), &bytes) {
    Ok(())
  } else {
    Err(Refusal::Failed(
      "Could not put the text on the clipboard".to_owned(),
    ))
  }
}

/// Puts `bytes` on the open clipboard as `format`. False when it could not.
fn set(format: u32, bytes: &[u8]) -> bool {
  // Safety: the memory is written while locked, and either handed to the
  // clipboard, which then owns it, or freed.
  unsafe {
    let Ok(memory) = GlobalAlloc(GMEM_MOVEABLE, bytes.len().max(1)) else {
      return false;
    };
    let data = GlobalLock(memory);
    if data.is_null() {
      let _ = GlobalFree(Some(memory));
      return false;
    }
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), data.cast::<u8>(), bytes.len());
    let _ = GlobalUnlock(memory);
    if SetClipboardData(format, Some(HANDLE(memory.0))).is_err() {
      let _ = GlobalFree(Some(memory));
      return false;
    }
  }
  true
}
