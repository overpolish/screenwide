// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The clipboard as a box being typed into copies to and pastes from it.

use windows::Win32::{
  Foundation::{GlobalFree, HANDLE, HGLOBAL, HWND},
  System::{
    DataExchange::{
      CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard, SetClipboardData,
    },
    Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE},
    Ole::CF_UNICODETEXT,
  },
};

/// Puts `text` on the clipboard, with the line breaks Windows text uses.
pub(in crate::editor::preview_platform::surface_windows) fn copy(window: HWND, text: &str) {
  let wide: Vec<u16> = text
    .replace('\n', "\r\n")
    .encode_utf16()
    .chain([0])
    .collect();
  unsafe {
    if OpenClipboard(Some(window)).is_err() {
      return;
    }
    let _ = EmptyClipboard();
    if let Ok(memory) = GlobalAlloc(GMEM_MOVEABLE, wide.len() * 2) {
      let target = GlobalLock(memory).cast::<u16>();
      if target.is_null() {
        let _ = GlobalFree(Some(memory));
      } else {
        std::ptr::copy_nonoverlapping(wide.as_ptr(), target, wide.len());
        let _ = GlobalUnlock(memory);
        // The clipboard owns the memory once it takes it, and only then.
        if SetClipboardData(u32::from(CF_UNICODETEXT.0), Some(HANDLE(memory.0))).is_err() {
          let _ = GlobalFree(Some(memory));
        }
      }
    }
    let _ = CloseClipboard();
  }
}

/// The text on the clipboard, if there is any.
pub(in crate::editor::preview_platform::surface_windows) fn paste(window: HWND) -> Option<String> {
  unsafe {
    OpenClipboard(Some(window)).ok()?;
    let text = GetClipboardData(u32::from(CF_UNICODETEXT.0))
      .ok()
      .and_then(|handle| {
        let memory = HGLOBAL(handle.0);
        let source = GlobalLock(memory).cast::<u16>();
        if source.is_null() {
          return None;
        }
        let mut length = 0;
        while *source.add(length) != 0 {
          length += 1;
        }
        let text = String::from_utf16_lossy(std::slice::from_raw_parts(source, length));
        let _ = GlobalUnlock(memory);
        Some(text)
      });
    let _ = CloseClipboard();
    text
  }
}
