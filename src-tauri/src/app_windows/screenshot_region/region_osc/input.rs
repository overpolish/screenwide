// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The region overlay's input. A platform decodes its own events and hands
//! them in through [`events`]; pass-through is the host's to express, from
//! [`hit_test`].

mod cursor;
mod dispatch;
mod events;
mod keyboard;
mod ruler_input;
use cursor::{apply_cursor, set_cursor};
use dispatch::{animation_frame, apply_result_cursor, dispatch, process, ruler_timer};
#[cfg(target_os = "windows")]
pub(crate) use events::take_press;
pub(crate) use events::{
  alt_changed, cursor_request, display_changed, hit_test, key_down, key_up, pan_begin, pan_end,
  pointer_cancelled, pointer_down, pointer_moved, pointer_up, right_click, timer, wheel,
};
#[cfg(target_os = "macos")]
pub(crate) use events::{magnify, trackpad_pan};
use keyboard::{keyboard_release, modifier_changed, overlay_keyboard_command};
use ruler_input::{
  ruler_label_step, ruler_pan_begin, ruler_pan_drag, ruler_pan_end, ruler_right_click,
  ruler_viewport, ruler_viewport_screen,
};

use super::host::{host, WindowId};

use super::ocr;
use super::ruler;
use super::state::{self, Context};
use screenwide_osc::{
  geometry::{Point, Rect},
  protocol::OscResult,
};

/// 16ms control-transition frames - the Win32 form of the self-rescheduling
/// `dispatch_after` chains the macOS chrome used.
pub(crate) const ANIMATION_TIMER: usize = 1;
/// The armed close button's two-second expiry.
pub(crate) const CONFIRM_TIMER: usize = 2;
/// The ruler's one-shot settle frame (`InputPhase::RulerAnimationFrame`).
pub(crate) const RULER_SETTLE_TIMER: usize = 3;
/// The copied checkmark's 900ms expiry.
pub(crate) const RULER_COPIED_TIMER: usize = 4;
/// The tolerance notice's 900ms expiry.
pub(crate) const RULER_TOLERANCE_TIMER: usize = 5;
const CONFIRM_TIMEOUT_MS: u32 = 2000;
const CONFIRM_RETRY_MS: u32 = 16;

/// One wheel notch's zoom factor, `exp(0.1)`. macOS derived it from
/// `exp(scrollingDeltaY * 0.01)` over a ten-point line scroll.
const WHEEL_ZOOM_EXPONENT: f64 = 0.1;
/// Logical points a wheel notch pans by.
const WHEEL_PAN_POINTS: f64 = 40.0;

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

/// `edgesForHandle`: W=1, E=2, N=4, S=8.
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
pub(crate) fn modifier_bits(shift: bool, control: bool, double_click: bool, alt: bool) -> u8 {
  let mut bits = 0_u8;
  if shift {
    bits |= 1;
  }
  if control {
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
fn vertical_wheel_zooms(notches: f64, notched: bool, control: bool) -> bool {
  control || (notches != 0.0 && notched)
}

fn confirm_expired(hwnd: WindowId) {
  host().kill_timer(hwnd, CONFIRM_TIMER);
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
      host().set_timer(hwnd, CONFIRM_TIMER, CONFIRM_RETRY_MS);
    }
  }
}

/// `updateMagnifier`: only a live resize with a handle and
/// a committed frame shows the lens.
fn magnifier_for(phase: u32, gesture: u8, has_region: u8, handle: u8) -> Option<u32> {
  (phase == PHASE_DRAG && gesture == GESTURE_RESIZING && has_region != 0 && handle != 0)
    .then(|| edges_for_handle(handle))
}

#[cfg(test)]
mod tests;

#[cfg(test)]
use keyboard::ocr_keyboard_phase;
