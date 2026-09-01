// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use windows::Win32::{
  Foundation::{HWND, LPARAM, POINT, WPARAM},
  UI::WindowsAndMessaging::{
    GetAncestor, GetWindowThreadProcessId, SendMessageTimeoutW, WindowFromPoint, GA_ROOT,
    HTCAPTION, HTCLOSE, HTHELP, HTMAXBUTTON, HTMINBUTTON, HTSYSMENU, SMTO_ABORTIFHUNG,
    WM_NCHITTEST,
  },
};

pub(super) fn window_at(point: POINT) -> Option<HWND> {
  let hovered = unsafe { WindowFromPoint(point) };
  if hovered.0.is_null() {
    return None;
  }
  let window = unsafe { GetAncestor(hovered, GA_ROOT) };
  if window.0.is_null() {
    return None;
  }
  let mut process_id = 0;
  unsafe { GetWindowThreadProcessId(window, Some(&mut process_id)) };
  if process_id == std::process::id() {
    return None;
  }

  let packed = ((point.y as u32 & 0xffff) << 16) | (point.x as u32 & 0xffff);
  let mut hit = 0usize;
  unsafe {
    SendMessageTimeoutW(
      window,
      WM_NCHITTEST,
      WPARAM(0),
      LPARAM(packed as isize),
      SMTO_ABORTIFHUNG,
      50,
      Some(&mut hit),
    );
  }
  matches!(
    hit as u32,
    HTCAPTION | HTCLOSE | HTHELP | HTMAXBUTTON | HTMINBUTTON | HTSYSMENU
  )
  .then_some(window)
}

pub(super) fn process_id(window: HWND) -> Option<u32> {
  let mut process_id = 0;
  unsafe { GetWindowThreadProcessId(window, Some(&mut process_id)) };
  (process_id != 0).then_some(process_id)
}
