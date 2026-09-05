// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Global cursor visibility for a Glide session.
//!
//! `ShowCursor` and `SetCursor` only affect windows owned by the calling
//! thread, and the pointer sits over another app's titlebar for the whole
//! session. The one process-independent switch Windows offers is the system
//! cursor scheme itself: every standard cursor is swapped for a blank one while
//! the session runs, and the user's scheme is reloaded from the registry when
//! it ends.

use windows::Win32::UI::WindowsAndMessaging::{
  CreateCursor, GetCursorPos, SetSystemCursor, SystemParametersInfoW, OCR_APPSTARTING, OCR_CROSS,
  OCR_HAND, OCR_IBEAM, OCR_NO, OCR_NORMAL, OCR_SIZEALL, OCR_SIZENESW, OCR_SIZENS, OCR_SIZENWSE,
  OCR_SIZEWE, OCR_UP, OCR_WAIT, SPI_SETCURSORS, SYSTEM_CURSOR_ID,
  SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
};

const CURSOR_IDS: [SYSTEM_CURSOR_ID; 13] = [
  OCR_NORMAL,
  OCR_IBEAM,
  OCR_WAIT,
  OCR_CROSS,
  OCR_UP,
  OCR_SIZENWSE,
  OCR_SIZENESW,
  OCR_SIZEWE,
  OCR_SIZENS,
  OCR_SIZEALL,
  OCR_NO,
  OCR_HAND,
  OCR_APPSTARTING,
];
const BLANK_SIZE: i32 = 32;

pub(super) fn hide_cursor() {
  crate::recording::cursor::glide_cursor_visibility(false, None);
  for id in CURSOR_IDS {
    // `SetSystemCursor` takes ownership of the handle and destroys it when the
    // scheme changes again, so every slot needs its own cursor.
    if let Ok(blank) = blank_cursor() {
      let _ = unsafe { SetSystemCursor(blank, id) };
    }
  }
}

pub(super) fn show_cursor() {
  let mut point = windows::Win32::Foundation::POINT::default();
  let position = unsafe { GetCursorPos(&mut point) }
    .ok()
    .map(|()| (f64::from(point.x), f64::from(point.y)));
  crate::recording::cursor::glide_cursor_visibility(true, position);
  let restored = unsafe {
    SystemParametersInfoW(
      SPI_SETCURSORS,
      0,
      None,
      SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS::default(),
    )
  };
  if let Err(error) = restored {
    eprintln!("Could not restore the system cursor scheme: {error}");
  }
}

/// A fully transparent monochrome cursor: the AND mask keeps every screen
/// pixel and the XOR mask changes none of them.
fn blank_cursor() -> windows::core::Result<windows::Win32::UI::WindowsAndMessaging::HCURSOR> {
  let bytes = (BLANK_SIZE * BLANK_SIZE / 8) as usize;
  let and_mask = vec![0xffu8; bytes];
  let xor_mask = vec![0x00u8; bytes];
  unsafe {
    CreateCursor(
      None,
      0,
      0,
      BLANK_SIZE,
      BLANK_SIZE,
      and_mask.as_ptr().cast(),
      xor_mask.as_ptr().cast(),
    )
  }
}
