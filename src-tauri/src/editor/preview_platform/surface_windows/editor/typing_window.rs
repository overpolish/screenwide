// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The window a text box's typing takes the keyboard in.
//!
//! The editor window never takes the keyboard: shortcuts belong to the
//! webview. A box being typed into needs keys, input methods and the
//! clipboard, so a child with no picture of its own takes focus for as long
//! as the typing lasts, and hands it back to the webview when it ends. It sits
//! at the caret, which is where an input method opens its candidates.

use std::sync::LazyLock;

use windows::{
  core::{w, PCWSTR},
  Win32::{
    Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
    Graphics::Gdi::ValidateRect,
    System::LibraryLoader::GetModuleHandleW,
    UI::{
      Input::KeyboardAndMouse::{GetFocus, GetKeyState, SetFocus, VK_CONTROL, VK_SHIFT},
      WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, GetCaretBlinkTime, GetParent, KillTimer, LoadCursorW,
        PostMessageW, RegisterClassW, SetTimer, SetWindowPos, ShowWindow, HMENU, IDC_IBEAM,
        SWP_ASYNCWINDOWPOS, SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER, SW_HIDE, SW_SHOWNOACTIVATE,
        WINDOW_EX_STYLE, WM_APP, WM_CHAR, WM_ERASEBKGND, WM_KEYDOWN, WM_KILLFOCUS, WM_PAINT,
        WM_TIMER, WNDCLASSW, WS_CHILD,
      },
    },
  },
};

use super::EditorWindow;

/// What the typing window hands on to the box being typed into.
#[derive(Clone, Copy)]
pub(in crate::editor::preview_platform::surface) enum TypingInput {
  /// A key went down, with whether Ctrl and Shift were held.
  Key { key: u16, ctrl: bool, shift: bool },
  /// One UTF-16 unit of typed text, after the keyboard layout and any input
  /// method have had their say.
  Char(u16),
  /// The caret's blink turned over.
  Blink,
  /// The keyboard went elsewhere: to the webview's panel, another window.
  FocusLost,
  /// The box went away from under the typing.
  Finish,
  /// The typing ended while it held the keyboard, which goes back to the
  /// webview.
  Released,
}

const BEGIN: u32 = WM_APP + 1;
const END: u32 = WM_APP + 2;
const FINISH: u32 = WM_APP + 3;
const BLINK: usize = 1;

/// The typing window's class, registered the first time one is made. Zero
/// when Windows refused it.
static CLASS: LazyLock<u16> = LazyLock::new(|| unsafe {
  let Ok(instance) = GetModuleHandleW(None) else {
    return 0;
  };
  RegisterClassW(&WNDCLASSW {
    lpfnWndProc: Some(window_proc),
    hInstance: HINSTANCE(instance.0),
    hCursor: LoadCursorW(None, IDC_IBEAM).unwrap_or_default(),
    lpszClassName: w!("ScreenwideAnnotationTyping"),
    ..Default::default()
  })
});

/// The typing window, hidden until a box is opened. Created beside the
/// editor window, on its thread, which is the one that pumps its messages.
pub(super) fn create(editor: HWND) -> Result<HWND, String> {
  let instance = unsafe { GetModuleHandleW(None) }.map_err(|error| error.to_string())?;
  if *CLASS == 0 {
    return Err("The Windows typing window class could not be registered".to_owned());
  }
  unsafe {
    CreateWindowExW(
      WINDOW_EX_STYLE(0),
      w!("ScreenwideAnnotationTyping"),
      PCWSTR::null(),
      WS_CHILD,
      0,
      0,
      1,
      1,
      Some(editor),
      Some(HMENU::default()),
      Some(HINSTANCE(instance.0)),
      None,
    )
  }
  .map_err(|error| format!("The Windows typing window could not be created: {error}"))
}

impl EditorWindow {
  /// The window that owns the clipboard while text is copied from a box.
  pub(in crate::editor::preview_platform::surface) fn typing_window(&self) -> HWND {
    self.typing
  }

  /// Takes the keyboard for a box being typed into. Posted, so it may be
  /// asked for from any thread.
  pub(in crate::editor::preview_platform::surface) fn begin_typing(&self) {
    let _ = unsafe { PostMessageW(Some(self.typing), BEGIN, WPARAM(0), LPARAM(0)) };
  }

  /// Gives the keyboard back once the typing is over.
  pub(in crate::editor::preview_platform::surface) fn end_typing(&self) {
    let _ = unsafe { PostMessageW(Some(self.typing), END, WPARAM(0), LPARAM(0)) };
  }

  /// Ends the typing and reports what was typed, a turn later: the box went
  /// away while something up the stack may still hold the surface.
  pub(in crate::editor::preview_platform::surface) fn finish_typing(&self) {
    let _ = unsafe { PostMessageW(Some(self.typing), FINISH, WPARAM(0), LPARAM(0)) };
  }

  /// Shows the caret for a whole blink again, as after every edit. Only on
  /// the typing window's own thread, where every edit arrives.
  pub(in crate::editor::preview_platform::surface) fn restart_blink(&self) {
    unsafe { SetTimer(Some(self.typing), BLINK, GetCaretBlinkTime(), None) };
  }

  /// Moves the typing window to the caret, in the editor's device pixels,
  /// for an input method to open its candidates beside it.
  pub(in crate::editor::preview_platform::surface) fn place_typing(&self, x: i32, y: i32) {
    let flags = SWP_ASYNCWINDOWPOS | SWP_NOACTIVATE | SWP_NOSIZE | SWP_NOZORDER;
    let _ = unsafe { SetWindowPos(self.typing, None, x, y, 0, 0, flags) };
  }
}

unsafe extern "system" fn window_proc(
  hwnd: HWND,
  message: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  let pressed = |key: u16| GetKeyState(i32::from(key)) < 0;
  match message {
    // The window has no picture: the compositor draws the caret into the box.
    WM_PAINT => {
      let _ = ValidateRect(Some(hwnd), None);
      LRESULT(0)
    }
    WM_ERASEBKGND => LRESULT(1),
    BEGIN => {
      let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
      let _ = SetFocus(Some(hwnd));
      SetTimer(Some(hwnd), BLINK, GetCaretBlinkTime(), None);
      LRESULT(0)
    }
    END => {
      let _ = KillTimer(Some(hwnd), BLINK);
      let focused = GetFocus() == hwnd;
      let _ = ShowWindow(hwnd, SW_HIDE);
      if focused {
        dispatch(hwnd, TypingInput::Released);
      }
      LRESULT(0)
    }
    FINISH => {
      dispatch(hwnd, TypingInput::Finish);
      LRESULT(0)
    }
    WM_TIMER if wparam.0 == BLINK => {
      dispatch(hwnd, TypingInput::Blink);
      LRESULT(0)
    }
    WM_KEYDOWN => {
      dispatch(
        hwnd,
        TypingInput::Key {
          key: wparam.0 as u16,
          ctrl: pressed(VK_CONTROL.0),
          shift: pressed(VK_SHIFT.0),
        },
      );
      LRESULT(0)
    }
    WM_CHAR => {
      dispatch(hwnd, TypingInput::Char(wparam.0 as u16));
      LRESULT(0)
    }
    WM_KILLFOCUS => {
      dispatch(hwnd, TypingInput::FocusLost);
      LRESULT(0)
    }
    _ => DefWindowProcW(hwnd, message, wparam, lparam),
  }
}

/// The typing window belongs to the editor window, which is what names the
/// surface. A panic cannot unwind out of a window procedure, so one is logged
/// and the input dropped, as the editor window's own procedure does.
fn dispatch(hwnd: HWND, input: TypingInput) {
  let Ok(editor) = (unsafe { GetParent(hwnd) }) else {
    return;
  };
  let work = || super::super::handle_typing_input(editor, input);
  if std::panic::catch_unwind(std::panic::AssertUnwindSafe(work)).is_err() {
    eprintln!("The Windows typing window dropped an input after a panic");
  }
}
