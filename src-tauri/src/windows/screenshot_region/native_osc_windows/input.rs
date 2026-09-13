// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Window procedure for the region overlay, porting `+input.m`'s
//! `processInput` ordering minus the ruler steps.
//!
//! macOS installed a global `NSEvent` monitor and returned `nil` to swallow;
//! Win32 gives the overlay the events it hit-tests into, so pass-through is
//! expressed as `HTTRANSPARENT` instead.

#[path = "input/cursor.rs"]
mod cursor;
#[path = "input/dispatch.rs"]
mod dispatch;
#[path = "input/keyboard.rs"]
mod keyboard;
#[path = "input/ruler_input.rs"]
mod ruler_input;
#[path = "input/window_proc.rs"]
mod window_proc;
use cursor::{apply_cursor, set_cursor};
use dispatch::{animation_frame, apply_result_cursor, dispatch, process, ruler_timer};
use keyboard::{keyboard_command, keyboard_release, modifier_changed, overlay_keyboard_command};
use ruler_input::{
  ruler_label_step, ruler_pan_begin, ruler_pan_drag, ruler_pan_end, ruler_right_click,
  ruler_viewport, ruler_viewport_screen,
};
use window_proc::guard;
pub(super) use window_proc::window_proc;

use windows::{
  core::PCWSTR,
  Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    UI::{
      Input::KeyboardAndMouse::{
        GetKeyState, ReleaseCapture, SetCapture, VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT,
      },
      WindowsAndMessaging::{
        DefWindowProcW, KillTimer, LoadCursorW, SetCursor, SetTimer, HTCLIENT, HTTRANSPARENT,
        IDC_ARROW, IDC_CROSS, IDC_HAND, IDC_IBEAM, IDC_SIZEALL, IDC_SIZENESW, IDC_SIZENS,
        IDC_SIZENWSE, IDC_SIZEWE, MA_NOACTIVATE, WHEEL_DELTA, WM_APP, WM_CANCELMODE,
        WM_CAPTURECHANGED, WM_DISPLAYCHANGE, WM_DPICHANGED, WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDBLCLK,
        WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEACTIVATE,
        WM_MOUSEHWHEEL, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_NCHITTEST, WM_RBUTTONDOWN, WM_SETCURSOR,
        WM_SYSKEYDOWN, WM_SYSKEYUP, WM_TIMER,
      },
    },
  },
};

use super::ocr;
use super::ruler;
use super::state::{self, Context};
use crate::osc::{
  geometry::{Point, Rect},
  protocol::OscResult,
};

/// 16ms control-transition frames - the Win32 form of the self-rescheduling
/// `dispatch_after` chains the macOS chrome used.
pub(crate) const ANIMATION_TIMER: usize = 1;
/// The armed close button's two-second expiry (`+ocr_toolbar_input.m:20-34`).
pub(crate) const CONFIRM_TIMER: usize = 2;
/// The ruler's one-shot settle frame (`InputPhase::RulerAnimationFrame`).
pub(crate) const RULER_SETTLE_TIMER: usize = 3;
/// The copied checkmark's 900ms expiry.
pub(crate) const RULER_COPIED_TIMER: usize = 4;
/// The tolerance notice's 900ms expiry.
pub(crate) const RULER_TOLERANCE_TIMER: usize = 5;
pub(crate) const OVERLAY_KEY_EVENT: u32 = WM_APP + 0x341;
pub(crate) const RULER_CURSOR_EVENT: u32 = WM_APP + 0x342;
const CONFIRM_TIMEOUT_MS: u32 = 2000;
const CONFIRM_RETRY_MS: u32 = 16;

/// One wheel notch's zoom factor, `exp(0.1)`. macOS derived it from
/// `exp(scrollingDeltaY * 0.01)` over a ten-point line scroll.
const WHEEL_ZOOM_EXPONENT: f64 = 0.1;
/// Logical points a wheel notch pans by.
const WHEEL_PAN_POINTS: f64 = 40.0;

const MK_SHIFT: usize = 0x0004;
const MK_CONTROL: usize = 0x0008;

/// Pointer phases on the wire, mirroring `InputPhase`.
const PHASE_MOVE: u32 = 1;
const PHASE_DOWN: u32 = 2;
const PHASE_DRAG: u32 = 3;
const PHASE_UP: u32 = 4;
const PHASE_CANCEL: u32 = 5;

const STATUS_INVALID: u8 = 255;
const GESTURE_RESIZING: u8 = 3;

mod magnifier;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum CursorShape {
  #[default]
  None,
  Crosshair,
  Move,
  ResizeHorizontal,
  ResizeVertical,
  ResizeNesw,
  ResizeNwse,
  Arrow,
  IBeam,
  Hand,
}

impl CursorShape {
  fn name(self) -> Option<PCWSTR> {
    Some(match self {
      Self::None => return None,
      Self::Crosshair => IDC_CROSS,
      // Windows has no open/closed hand: the drag cursor is the move cursor.
      Self::Move => IDC_SIZEALL,
      Self::ResizeHorizontal => IDC_SIZEWE,
      Self::ResizeVertical => IDC_SIZENS,
      Self::ResizeNesw => IDC_SIZENESW,
      Self::ResizeNwse => IDC_SIZENWSE,
      Self::Arrow => IDC_ARROW,
      Self::IBeam => IDC_IBEAM,
      Self::Hand => IDC_HAND,
    })
  }
}

/// `edgesForHandle` (`+input.m:14-26`): W=1, E=2, N=4, S=8.
pub(crate) fn edges_for_handle(handle: u8) -> u32 {
  match handle {
    2 => 4,
    3 => 8,
    4 => 2,
    5 => 1,
    6 => 2 | 4,
    7 => 1 | 4,
    8 => 2 | 8,
    9 => 1 | 8,
    _ => 0,
  }
}

/// `CursorIcon` 1..=9 mapped onto system cursors. The diagonal case picks its
/// axis from the dragged handle's edge bitmask.
pub(crate) fn cursor_shape(cursor: u8, handle: u8) -> CursorShape {
  match cursor {
    1 => CursorShape::Crosshair,
    2 | 3 => CursorShape::Move,
    4 => CursorShape::ResizeHorizontal,
    5 => CursorShape::ResizeVertical,
    6 => match edges_for_handle(handle) {
      edges if edges == 2 | 4 || edges == 1 | 8 => CursorShape::ResizeNesw,
      edges if edges == 1 | 4 || edges == 2 | 8 => CursorShape::ResizeNwse,
      _ => CursorShape::Crosshair,
    },
    7 => CursorShape::Arrow,
    8 => CursorShape::IBeam,
    9 => CursorShape::Hand,
    _ => CursorShape::None,
  }
}

/// The overlay covers the whole client area, so everything it must not eat is
/// declined here: disabled input, a hidden scene, and the webview's own
/// toolbar inside `exclusion_rect`.
pub(crate) fn hit_transparent(
  input_enabled: bool,
  visible: bool,
  exclusion: Rect,
  point: Point,
) -> bool {
  if !input_enabled || !visible {
    return true;
  }
  exclusion.size.width > 0.0 && exclusion.size.height > 0.0 && exclusion.contains(point)
}

/// `1 = shift, 2 = ctrl, 4 = double click, 8 = alt`, the one cross-platform
/// meaning `InputModifiers::from_bits` decodes.
pub(crate) fn modifier_bits(wparam: usize, double_click: bool, alt: bool) -> u8 {
  let mut bits = 0_u8;
  if wparam & MK_SHIFT != 0 {
    bits |= 1;
  }
  if wparam & MK_CONTROL != 0 {
    bits |= 2;
  }
  if double_click {
    bits |= 4;
  }
  if alt {
    bits |= 8;
  }
  bits
}

/// Legacy mouse wheels report integral 120-unit notches. Precision touchpad
/// scrolling normally arrives as smaller deltas, which Ruler treats as pan.
/// Ctrl remains an explicit zoom override for either input source.
fn vertical_wheel_zooms(delta: i16, control: bool) -> bool {
  control || (delta != 0 && i32::from(delta) % WHEEL_DELTA as i32 == 0)
}

fn alt_pressed() -> bool {
  crate::ruler::windows_alt_pressed()
}

fn client_point(lparam: LPARAM) -> (f64, f64) {
  let x = f64::from(lparam.0 as u16 as i16);
  let y = f64::from((lparam.0 >> 16) as u16 as i16);
  (x, y)
}

fn screen_point(lparam: LPARAM) -> (i32, i32) {
  (
    i32::from(lparam.0 as u16 as i16),
    i32::from((lparam.0 >> 16) as u16 as i16),
  )
}

fn confirm_expired(hwnd: HWND) {
  let _ = unsafe { KillTimer(Some(hwnd), CONFIRM_TIMER) };
  let Some(context) = state::context_for_surface(hwnd) else {
    return;
  };
  let Ok(mut set) = context.surfaces.lock() else {
    return;
  };
  if let Some(surface) = set.find_mut(hwnd) {
    let outcome = surface.ocr.expire_confirm();
    if outcome.redraw {
      surface.draw();
    }
    if outcome.arm_confirm {
      let _ = unsafe { SetTimer(Some(hwnd), CONFIRM_TIMER, CONFIRM_RETRY_MS, None) };
    }
  }
}

/// `updateMagnifier` (`+input.m:68-94`): only a live resize with a handle and
/// a committed frame shows the lens.
fn magnifier_for(phase: u32, gesture: u8, has_region: u8, handle: u8) -> Option<u32> {
  (phase == PHASE_DRAG && gesture == GESTURE_RESIZING && has_region != 0 && handle != 0)
    .then(|| edges_for_handle(handle))
}

#[cfg(test)]
#[path = "input/tests.rs"]
mod tests;

#[cfg(test)]
use keyboard::ocr_keyboard_phase;
