// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn apply_cursor(hwnd: HWND) -> bool {
  let Some(context) = state::context_for_surface(hwnd) else {
    return false;
  };
  let shape = context
    .surfaces
    .lock()
    .map(|mut set| {
      set
        .find_mut(hwnd)
        .map_or(CursorShape::None, |surface| surface.cursor)
    })
    .unwrap_or_default();
  set_cursor(shape)
}

pub(super) fn set_cursor(shape: CursorShape) -> bool {
  let Some(name) = shape.name() else {
    return false;
  };
  if let Ok(cursor) = unsafe { LoadCursorW(None, name) } {
    unsafe { SetCursor(Some(cursor)) };
    return true;
  }
  false
}
