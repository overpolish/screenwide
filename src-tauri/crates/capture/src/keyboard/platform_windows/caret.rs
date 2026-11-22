// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where the text caret is, in global physical pixels as the cursor file
//! measures on Windows. A classic edit control shows the system caret, which
//! the window's thread reports; anything else that types, a browser or a
//! modern app, says where its selection is through UI Automation; a control
//! that says neither is passed over. Every question here crosses into the
//! focused app, so it is only asked from the focus probe's own thread.

use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::System::Com::SAFEARRAY;
use windows::Win32::System::Ole::{
  SafeArrayAccessData, SafeArrayDestroy, SafeArrayGetUBound, SafeArrayUnaccessData,
};
use windows::Win32::UI::Accessibility::{
  IUIAutomation, IUIAutomationTextPattern, IUIAutomationTextRange, TextUnit_Character,
  UIA_TextPatternId,
};
use windows::Win32::UI::WindowsAndMessaging::{
  GetForegroundWindow, GetGUIThreadInfo, GetWindowThreadProcessId, GUITHREADINFO,
};

use super::super::CaretRect;

/// Where the caret is in the control that has the keyboard, or `None` where
/// it does not say.
pub(super) fn caret(automation: &IUIAutomation) -> Option<CaretRect> {
  system_caret().or_else(|| selection(automation))
}

/// The system caret of the foreground window's thread, which classic edit
/// controls draw. Its box is in the caret window's client area.
fn system_caret() -> Option<CaretRect> {
  let window = unsafe { GetForegroundWindow() };
  if window.is_invalid() {
    return None;
  }
  let thread = unsafe { GetWindowThreadProcessId(window, None) };
  let mut info = GUITHREADINFO {
    cbSize: size_of::<GUITHREADINFO>() as u32,
    ..Default::default()
  };
  unsafe { GetGUIThreadInfo(thread, &mut info) }.ok()?;
  let RECT {
    left,
    top,
    right,
    bottom,
  } = info.rcCaret;
  if info.hwndCaret.is_invalid() || bottom <= top {
    return None;
  }
  let origin = on_screen(info.hwndCaret, left, top)?;
  Some(CaretRect {
    x: f64::from(origin.x),
    y: f64::from(origin.y),
    width: f64::from((right - left).max(1)),
    height: f64::from(bottom - top),
  })
}

fn on_screen(window: HWND, x: i32, y: i32) -> Option<POINT> {
  let mut point = POINT { x, y };
  unsafe { ClientToScreen(window, &mut point) }
    .as_bool()
    .then_some(point)
}

/// The box of the focused control's selection, as UI Automation tells it: a
/// caret is an empty selection, which many controls give no box for, so the
/// character it sits before is measured instead and its start taken.
fn selection(automation: &IUIAutomation) -> Option<CaretRect> {
  let element = unsafe { automation.GetFocusedElement() }.ok()?;
  let pattern: IUIAutomationTextPattern =
    unsafe { element.GetCurrentPatternAs(UIA_TextPatternId) }.ok()?;
  let ranges = unsafe { pattern.GetSelection() }.ok()?;
  if unsafe { ranges.Length() }.ok()? < 1 {
    return None;
  }
  let range = unsafe { ranges.GetElement(0) }.ok()?;
  if let Some(rect) = last_box(&range) {
    return Some(CaretRect {
      x: rect.x + rect.width,
      width: 1.0,
      ..rect
    });
  }
  let character = unsafe { range.Clone() }.ok()?;
  unsafe { character.ExpandToEnclosingUnit(TextUnit_Character) }.ok()?;
  last_box(&character).map(|rect| CaretRect { width: 1.0, ..rect })
}

/// The last box of a text range, which is where a selection ends.
fn last_box(range: &IUIAutomationTextRange) -> Option<CaretRect> {
  let boxes = unsafe { range.GetBoundingRectangles() }.ok()?;
  let found = read_last_box(boxes);
  let _ = unsafe { SafeArrayDestroy(boxes) };
  found
}

/// The last of the left, top, width and height runs in an array of doubles.
fn read_last_box(boxes: *mut SAFEARRAY) -> Option<CaretRect> {
  if boxes.is_null() {
    return None;
  }
  let count = usize::try_from(unsafe { SafeArrayGetUBound(boxes, 1) }.ok()? + 1).ok()?;
  if count < 4 {
    return None;
  }
  let mut data = std::ptr::null_mut();
  unsafe { SafeArrayAccessData(boxes, &mut data) }.ok()?;
  // SAFETY: UI Automation gives a one-dimensional array of `count` doubles,
  // held still while accessed.
  let values = unsafe { std::slice::from_raw_parts(data.cast::<f64>(), count) };
  let last = &values[count - count % 4 - 4..count - count % 4];
  let rect = (last[3] > 0.0).then_some(CaretRect {
    x: last[0],
    y: last[1],
    width: last[2],
    height: last[3],
  });
  let _ = unsafe { SafeArrayUnaccessData(boxes) };
  rect
}
