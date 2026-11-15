// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Pasting on Windows. The text goes on the clipboard by delayed rendering:
//! a hidden window owns the clipboard and is sent `WM_RENDERFORMAT` when an
//! app reads the text, which is the receipt. Ownership and delayed rendering
//! belong to the thread whose window it is, so the whole paste, from what the
//! clipboard held to putting it back, runs on the calling thread, pumping
//! that window's messages while it waits.

mod formats;
mod guard;
mod keys;

use std::cell::RefCell;
use std::time::{Duration, Instant};

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{SetLastError, HWND, LPARAM, LRESULT, WIN32_ERROR, WPARAM};
use windows::Win32::System::DataExchange::{
  CloseClipboard, EmptyClipboard, GetClipboardOwner, OpenClipboard, RegisterClipboardFormatW,
  SetClipboardData,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Ole::CF_UNICODETEXT;
use windows::Win32::UI::WindowsAndMessaging::{
  CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, MsgWaitForMultipleObjects,
  PeekMessageW, RegisterClassW, HWND_MESSAGE, MSG, PM_REMOVE, QS_ALLINPUT, WINDOW_EX_STYLE,
  WINDOW_STYLE, WM_DESTROYCLIPBOARD, WM_RENDERALLFORMATS, WM_RENDERFORMAT, WNDCLASSW,
};

use self::formats::{held, put_back, set};
use self::keys::press_paste;
use super::{settled, OnMain, Refusal};

/// How long the wait sleeps between looks, unless a message arrives.
const POLL: Duration = Duration::from_millis(20);
/// How many times opening a busy clipboard is tried, `POLL` apart.
const OPEN_TRIES: u32 = 10;

/// The paste the hidden window answers for.
struct Pending {
  text: Vec<u16>,
  last_read: Option<Instant>,
}

thread_local! {
  static PENDING: RefCell<Option<Pending>> = const { RefCell::new(None) };
}

pub(super) fn paste(text: &str, _on_main: OnMain<'_>) -> Result<(), Refusal> {
  guard::check()?;
  let window = Window::create()?;
  let held = with_clipboard(window.0, || Ok(held()))?;
  PENDING.with(|pending| {
    *pending.borrow_mut() = Some(Pending {
      text: text.encode_utf16().chain([0]).collect(),
      last_read: None,
    });
  });
  with_clipboard(window.0, publish)?;
  let sent = Instant::now();
  press_paste()?;
  loop {
    pump(window.0);
    let last_read = PENDING.with(|pending| pending.borrow().as_ref().and_then(|p| p.last_read));
    if settled(Instant::now(), sent, last_read) {
      break;
    }
  }
  // Put back only while the window still owns the clipboard: anything copied
  // since took it.
  // Safety: a plain query.
  if unsafe { GetClipboardOwner() }.is_ok_and(|owner| owner == window.0) {
    with_clipboard(window.0, || {
      put_back(&held);
      Ok(())
    })?;
  }
  PENDING.with(|pending| pending.borrow_mut().take());
  Ok(())
}

/// The hidden window that owns the clipboard while the text is on it.
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

extern "system" fn procedure(
  window: HWND,
  message: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  match message {
    // An app reads the text: the system has opened the clipboard for it.
    WM_RENDERFORMAT if u32::try_from(wparam.0).ok() == Some(u32::from(CF_UNICODETEXT.0)) => {
      PENDING.with(|pending| {
        if let Some(pending) = pending.borrow_mut().as_mut() {
          render(&pending.text);
          pending.last_read = Some(Instant::now());
        }
      });
      LRESULT(0)
    }
    // The window goes with the text still promised: it is rendered, so the
    // clipboard is not left holding a promise nobody can keep.
    WM_RENDERALLFORMATS => {
      // Safety: the clipboard is opened and closed here, and only rendered
      // into while this window still owns it.
      unsafe {
        if OpenClipboard(Some(window)).is_ok() {
          if GetClipboardOwner().is_ok_and(|owner| owner == window) {
            PENDING.with(|pending| {
              if let Some(pending) = pending.borrow().as_ref() {
                render(&pending.text);
              }
            });
          }
          let _ = CloseClipboard();
        }
      }
      LRESULT(0)
    }
    WM_DESTROYCLIPBOARD => LRESULT(0),
    // Safety: everything else is the default's.
    _ => unsafe { DefWindowProcW(window, message, wparam, lparam) },
  }
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

/// Promises the text, marked to be kept out of clipboard history, the cloud
/// clipboard and clipboard managers, as browsers mark a private window's
/// copies. The clipboard must be open.
fn publish() -> Result<(), Refusal> {
  // Safety: the clipboard is open, and each handle set is handed to it.
  unsafe {
    EmptyClipboard().map_err(|_| Refusal::Failed("Could not empty the clipboard".to_owned()))?;
    // A null handle is a promise, and success then also answers null, with
    // no error set.
    SetLastError(WIN32_ERROR(0));
    if let Err(error) = SetClipboardData(u32::from(CF_UNICODETEXT.0), None) {
      if error.code().is_err() {
        return Err(Refusal::Failed(
          "Could not put the text on the clipboard".to_owned(),
        ));
      }
    }
    for (name, value) in [
      (w!("ExcludeClipboardContentFromMonitorProcessing"), 0_u32),
      (w!("CanIncludeInClipboardHistory"), 0),
      (w!("CanUploadToCloudClipboard"), 0),
    ] {
      let format = RegisterClipboardFormatW(name);
      if format != 0 {
        set(format, &value.to_ne_bytes());
      }
    }
  }
  Ok(())
}

/// Hands the clipboard `text` as the promised format. The clipboard must be
/// open, by the system for a render or by this thread.
fn render(text: &[u16]) {
  let bytes: Vec<u8> = text.iter().flat_map(|unit| unit.to_ne_bytes()).collect();
  set(u32::from(CF_UNICODETEXT.0), &bytes);
}

/// Answers the window's messages until one arrives or `POLL` passes.
fn pump(window: HWND) {
  // Safety: the messages are this thread's own, for its own window.
  unsafe {
    MsgWaitForMultipleObjects(
      None,
      false,
      u32::try_from(POLL.as_millis()).unwrap_or(20),
      QS_ALLINPUT,
    );
    let mut message = MSG::default();
    while PeekMessageW(&raw mut message, Some(window), 0, 0, PM_REMOVE).as_bool() {
      DispatchMessageW(&raw const message);
    }
  }
}
