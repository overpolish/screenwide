// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// Registers one overlay window class. Reports the class atom, or 0 the way
/// `RegisterClassW` does, so the caller can name the class that failed. The
/// caller keeps the `OnceLock` holding the atom: a class name may only be
/// registered once per process.
pub(crate) fn register_class(name: PCWSTR, window_proc: WNDPROC, style: WNDCLASS_STYLES) -> u16 {
  let Ok(instance) = (unsafe { GetModuleHandleW(None) }) else {
    return 0;
  };
  unsafe {
    RegisterClassW(&WNDCLASSW {
      style,
      lpfnWndProc: window_proc,
      hInstance: HINSTANCE(instance.0),
      // The arrow is the class default; an overlay that wants another cursor
      // sets it per frame in `WM_SETCURSOR`.
      hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
      lpszClassName: name,
      ..Default::default()
    })
  }
}

/// An overlay child over `parent`: DirectComposition owns every pixel
/// (`WS_EX_NOREDIRECTIONBITMAP`), and the window never takes activation, so
/// pressing it leaves the foreground where it was. It is created at 1x1 and
/// sized by whoever lays it out.
pub(crate) fn create_child(parent: HWND, class_name: PCWSTR) -> Result<HWND, String> {
  let instance = unsafe { GetModuleHandleW(None) }.map_err(|error| error.to_string())?;
  unsafe {
    CreateWindowExW(
      WS_EX_NOACTIVATE | WS_EX_NOREDIRECTIONBITMAP,
      class_name,
      PCWSTR::null(),
      WS_CHILD | WS_CLIPSIBLINGS,
      0,
      0,
      1,
      1,
      Some(parent),
      Some(HMENU::default()),
      Some(HINSTANCE(instance.0)),
      None,
    )
  }
  .map_err(|error| format!("The Windows overlay child could not be created: {error}"))
}

/// Creates a window on the thread that owns the host HWND: Win32 queues a
/// window's messages on its creating thread, so a surface window created on a
/// worker thread would never see a mouse message.
pub(crate) fn create_on_owning_thread<F>(
  window: &tauri::WebviewWindow,
  host: HWND,
  build: F,
) -> Result<HWND, String>
where
  F: FnOnce(HWND) -> Result<HWND, String> + Send + 'static,
{
  struct HostHandle(HWND);
  unsafe impl Send for HostHandle {}
  struct Created(Result<HWND, String>);
  unsafe impl Send for Created {}

  if unsafe { GetWindowThreadProcessId(host, None) } == unsafe { GetCurrentThreadId() } {
    return build(host);
  }
  let handle = HostHandle(host);
  let (sender, receiver) = std::sync::mpsc::sync_channel(1);
  window
    .run_on_main_thread(move || {
      let handle = handle;
      let _ = sender.send(Created(build(handle.0)));
    })
    .map_err(|error| format!("The Windows overlay window could not be dispatched: {error}"))?;
  receiver
    .recv()
    .map_err(|_| "The Windows overlay window was never created".to_owned())?
    .0
}

/// Overlays appear and disappear with their session; the shell's own open and
/// close animations would show them sliding.
pub(crate) fn disable_transitions(hwnd: HWND) -> Result<(), String> {
  let disabled = windows::core::BOOL(1);
  unsafe {
    DwmSetWindowAttribute(
      hwnd,
      DWMWA_TRANSITIONS_FORCEDISABLED,
      (&raw const disabled).cast(),
      std::mem::size_of::<windows::core::BOOL>() as u32,
    )
  }
  .map_err(|error| error.to_string())
}

/// `NSWindowSharingNone`'s twin. Only top-level windows are accepted: a child
/// inherits the affinity of the window it sits in.
pub(crate) fn set_capture_affinity(hwnd: HWND, capturable: bool) -> Result<(), String> {
  let affinity = if capturable {
    WDA_NONE
  } else {
    WDA_EXCLUDEFROMCAPTURE
  };
  unsafe { SetWindowDisplayAffinity(hwnd, affinity) }.map_err(|error| error.to_string())
}
