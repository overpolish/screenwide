// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Session lock notifications reach a window, so the watch keeps a
//! message-only one on a thread of its own, blocked in its message loop
//! between them.

use std::cell::RefCell;
use std::sync::mpsc;
use std::thread::JoinHandle;

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::RemoteDesktop::{
  WTSRegisterSessionNotification, WTSUnRegisterSessionNotification, NOTIFY_FOR_THIS_SESSION,
};
use windows::Win32::UI::WindowsAndMessaging::{
  CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW, PostMessageW,
  PostQuitMessage, RegisterClassW, HWND_MESSAGE, MSG, WINDOW_EX_STYLE, WINDOW_STYLE, WM_CLOSE,
  WM_DESTROY, WM_WTSSESSION_CHANGE, WNDCLASSW, WTS_SESSION_LOCK, WTS_SESSION_UNLOCK,
};

const CLASS: PCWSTR = w!("ScreenwideLockWatch");

type Changed = Box<dyn Fn(bool) + Send>;

thread_local! {
  /// The watch's callback, on its own thread, where its window's procedure
  /// runs.
  static CHANGED: RefCell<Option<Changed>> = const { RefCell::new(None) };
}

/// Tells whether the screen has locked or unlocked, until dropped.
pub(crate) struct LockWatch {
  /// The window's handle as a number: a handle may be posted to from any
  /// thread, but `HWND` is not `Send`.
  window: usize,
  worker: Option<JoinHandle<()>>,
}

impl LockWatch {
  pub(crate) fn start(changed: impl Fn(bool) + Send + Sync + 'static) -> Result<Self, String> {
    let (ready, opened) = mpsc::channel();
    let changed: Changed = Box::new(changed);
    let worker = std::thread::Builder::new()
      .name("screenwide-lock-watch".to_owned())
      .spawn(move || run(changed, &ready))
      .map_err(|error| format!("Could not watch the screen lock: {error}"))?;
    match opened.recv() {
      Ok(Ok(window)) => Ok(Self {
        window,
        worker: Some(worker),
      }),
      Ok(Err(error)) => {
        let _ = worker.join();
        Err(error)
      }
      Err(_) => Err("The screen lock watch stopped before it started".to_owned()),
    }
  }
}

impl Drop for LockWatch {
  fn drop(&mut self) {
    // SAFETY: the window belongs to the watch's thread, which is still in
    // its message loop; WM_CLOSE ends it there.
    let _ = unsafe {
      PostMessageW(
        Some(HWND(self.window as *mut _)),
        WM_CLOSE,
        WPARAM(0),
        LPARAM(0),
      )
    };
    if let Some(worker) = self.worker.take() {
      let _ = worker.join();
    }
  }
}

fn run(changed: Changed, ready: &mpsc::Sender<Result<usize, String>>) {
  let window = match open() {
    Ok(window) => window,
    Err(error) => {
      let _ = ready.send(Err(error));
      return;
    }
  };
  CHANGED.with_borrow_mut(|slot| *slot = Some(changed));
  let _ = ready.send(Ok(window.0 as usize));
  let mut message = MSG::default();
  // SAFETY: `message` is a valid MSG; zero ends the loop at WM_QUIT and a
  // negative result on error.
  while unsafe { GetMessageW(&mut message, None, 0, 0) }.0 > 0 {
    // SAFETY: a message just taken from this thread's queue.
    unsafe { DispatchMessageW(&message) };
  }
  CHANGED.with_borrow_mut(Option::take);
}

fn open() -> Result<HWND, String> {
  // SAFETY: asks for this executable's own module handle.
  let module = unsafe { GetModuleHandleW(None) }
    .map_err(|error| format!("Could not watch the screen lock: {error}"))?;
  let instance = HINSTANCE(module.0);
  let class = WNDCLASSW {
    lpfnWndProc: Some(window_procedure),
    hInstance: instance,
    lpszClassName: CLASS,
    ..Default::default()
  };
  // SAFETY: `class` is fully initialised. A watch started again finds the
  // class already registered, which is fine, so the result is not checked:
  // creating the window fails if the class is missing.
  unsafe { RegisterClassW(&class) };
  // SAFETY: a message-only window of the class registered above.
  let window = unsafe {
    CreateWindowExW(
      WINDOW_EX_STYLE(0),
      CLASS,
      None,
      WINDOW_STYLE(0),
      0,
      0,
      0,
      0,
      Some(HWND_MESSAGE),
      None,
      Some(instance),
      None,
    )
  }
  .map_err(|error| format!("Could not watch the screen lock: {error}"))?;
  // SAFETY: the window was just created on this thread.
  if let Err(error) = unsafe { WTSRegisterSessionNotification(window, NOTIFY_FOR_THIS_SESSION) } {
    // SAFETY: as above; it is destroyed once.
    let _ = unsafe { DestroyWindow(window) };
    return Err(format!("Could not watch the screen lock: {error}"));
  }
  Ok(window)
}

unsafe extern "system" fn window_procedure(
  window: HWND,
  message: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  match message {
    WM_WTSSESSION_CHANGE => {
      let locked = match wparam.0 as u32 {
        WTS_SESSION_LOCK => Some(true),
        WTS_SESSION_UNLOCK => Some(false),
        _ => None,
      };
      if let Some(locked) = locked {
        CHANGED.with_borrow(|changed| {
          if let Some(changed) = changed {
            changed(locked);
          }
        });
      }
      LRESULT(0)
    }
    WM_CLOSE => {
      // SAFETY: the window registered for notifications in `open` and is
      // destroyed once, on its own thread.
      unsafe {
        let _ = WTSUnRegisterSessionNotification(window);
        let _ = DestroyWindow(window);
      }
      LRESULT(0)
    }
    WM_DESTROY => {
      // SAFETY: ends this thread's message loop.
      unsafe { PostQuitMessage(0) };
      LRESULT(0)
    }
    // SAFETY: everything else gets the default handling.
    _ => unsafe { DefWindowProcW(window, message, wparam, lparam) },
  }
}
