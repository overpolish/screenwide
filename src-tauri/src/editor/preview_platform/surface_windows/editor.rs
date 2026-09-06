// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Non-activating native input window for the DirectComposition workspace.

use std::sync::OnceLock;

use windows::{
  core::{w, PCWSTR},
  Win32::{
    Foundation::{HINSTANCE, HWND},
    System::LibraryLoader::GetModuleHandleW,
    UI::WindowsAndMessaging::{
      CreateWindowExW, DestroyWindow, KillTimer, LoadCursorW, RegisterClassW, SetCursor, SetTimer,
      SetWindowPos, ShowWindowAsync, CS_DBLCLKS, CW_USEDEFAULT, HMENU, HWND_TOP, IDC_ARROW,
      IDC_SIZEALL, IDC_SIZENESW, IDC_SIZENS, IDC_SIZENWSE, IDC_SIZEWE, SWP_ASYNCWINDOWPOS,
      SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOOWNERZORDER, SWP_NOSIZE, SWP_SHOWWINDOW, SW_HIDE,
      SW_SHOWNOACTIVATE, WNDCLASSW, WS_CHILD, WS_CLIPSIBLINGS, WS_EX_NOACTIVATE,
      WS_EX_NOREDIRECTIONBITMAP,
    },
  },
};

#[path = "editor/input.rs"]
mod input;

#[derive(Clone, Copy)]
pub(super) enum CursorKind {
  Arrow,
  Move,
  ResizeHorizontal,
  ResizeVertical,
  ResizeNesw,
  ResizeNwse,
}

#[derive(Clone, Copy)]
// Payloads mirror the Win32 messages; not every field is consumed yet.
#[allow(dead_code)]
pub(super) enum Input {
  AnimateAction,
  DoubleClick {
    x: f64,
    y: f64,
  },
  Down {
    centered: bool,
    x: f64,
    y: f64,
    snapping: bool,
  },
  Move {
    centered: bool,
    x: f64,
    y: f64,
    pressed: bool,
    snapping: bool,
  },
  Cancel,
  /// Middle button: pans from wherever it lands, like any non-primary button
  /// on macOS. Trackpads are rare on Windows and the primary button is taken
  /// by selection over a pane.
  PanDown {
    x: f64,
    y: f64,
  },
  PanUp {
    x: f64,
    y: f64,
  },
  Up {
    x: f64,
    y: f64,
  },
  Wheel {
    x: f64,
    y: f64,
    delta: f64,
  },
}

pub(super) struct EditorWindow {
  hwnd: HWND,
}

unsafe impl Send for EditorWindow {}
unsafe impl Sync for EditorWindow {}

impl EditorWindow {
  const ACTION_TIMER: usize = 1;

  pub(super) fn new(parent: HWND) -> Result<Self, String> {
    let instance = unsafe { GetModuleHandleW(None) }.map_err(|error| error.to_string())?;
    let atom = *CLASS.get_or_init(|| unsafe {
      RegisterClassW(&WNDCLASSW {
        style: CS_DBLCLKS,
        lpfnWndProc: Some(input::window_proc),
        hInstance: HINSTANCE(instance.0),
        hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
        lpszClassName: w!("ScreenwidePreviewEditor"),
        ..Default::default()
      })
    });
    if atom == 0 {
      return Err("The Windows preview editor class could not be registered".to_owned());
    }
    let hwnd = unsafe {
      CreateWindowExW(
        WS_EX_NOACTIVATE | WS_EX_NOREDIRECTIONBITMAP,
        w!("ScreenwidePreviewEditor"),
        PCWSTR::null(),
        WS_CHILD | WS_CLIPSIBLINGS,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        1,
        1,
        Some(parent),
        Some(HMENU::default()),
        Some(HINSTANCE(instance.0)),
        None,
      )
    }
    .map_err(|error| format!("The Windows preview editor could not be created: {error}"))?;
    // A freshly created child lands at the bottom of the sibling z-order, so
    // raise it above WebView2 before the first `set_frame` arrives.
    raise(hwnd);
    Ok(Self { hwnd })
  }

  pub(super) fn hwnd(&self) -> HWND {
    self.hwnd
  }

  /// `ShowWindowAsync` posts rather than sends, so callers off the event-loop
  /// thread never block inside the main thread while holding surface state.
  pub(super) fn set_active(&self, active: bool) {
    let _ = unsafe { ShowWindowAsync(self.hwnd, if active { SW_SHOWNOACTIVATE } else { SW_HIDE }) };
    if active {
      raise(self.hwnd);
    }
  }

  /// `SWP_ASYNCWINDOWPOS` likewise posts the request when it arrives from a
  /// non-owning thread. The z-order is deliberately re-asserted to `HWND_TOP`
  /// on every move: the editor must stay above the sibling WebView2 child.
  pub(super) fn set_frame(&self, x: i32, y: i32, width: i32, height: i32, active: bool) {
    let visibility = if active {
      SWP_SHOWWINDOW
    } else {
      Default::default()
    };
    let flags = SWP_ASYNCWINDOWPOS | SWP_NOACTIVATE | SWP_NOOWNERZORDER | visibility;
    let _ = unsafe {
      SetWindowPos(
        self.hwnd,
        Some(HWND_TOP),
        x,
        y,
        width.max(1),
        height.max(1),
        flags,
      )
    };
  }

  pub(super) fn set_cursor(kind: CursorKind) {
    let name = match kind {
      CursorKind::Arrow => IDC_ARROW,
      CursorKind::Move => IDC_SIZEALL,
      CursorKind::ResizeHorizontal => IDC_SIZEWE,
      CursorKind::ResizeVertical => IDC_SIZENS,
      CursorKind::ResizeNesw => IDC_SIZENESW,
      CursorKind::ResizeNwse => IDC_SIZENWSE,
    };
    if let Ok(cursor) = unsafe { LoadCursorW(None, name) } {
      unsafe { SetCursor(Some(cursor)) };
    }
  }

  pub(super) fn animate_action(hwnd: HWND) {
    let _ = unsafe { SetTimer(Some(hwnd), Self::ACTION_TIMER, 16, None) };
  }

  pub(super) fn stop_action_animation(hwnd: HWND) {
    let _ = unsafe { KillTimer(Some(hwnd), Self::ACTION_TIMER) };
  }
}

impl Drop for EditorWindow {
  // `DestroyWindow` only works on the creating thread; the surface lives in the
  // process-lifetime registry keyed by its host window and is never dropped in
  // practice.
  fn drop(&mut self) {
    let _ = unsafe { DestroyWindow(self.hwnd) };
  }
}

/// Re-asserts the editor above its WebView2 sibling without moving, sizing,
/// or activating it.
fn raise(hwnd: HWND) {
  let _ = unsafe {
    SetWindowPos(
      hwnd,
      Some(HWND_TOP),
      0,
      0,
      0,
      0,
      SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_ASYNCWINDOWPOS,
    )
  };
}

static CLASS: OnceLock<u16> = OnceLock::new();
