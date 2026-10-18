// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The events a platform hands the overlay, already decoded: points in the
//! window's client pixels, modifiers as [`modifier_bits`] packs them, keys as
//! virtual-key codes. Each answers whether the overlay took the event, so the
//! host can pass the rest on.

use super::*;
use crate::app_windows::screenshot_region::region_osc::host::ScreenPoint;

/// Whether a press at `screen` passes through the overlay to what is under
/// it: disabled input, a hidden scene, and the webview's own toolbar inside
/// `exclusion_rect` are not the overlay's.
pub(crate) fn hit_test(window: WindowId, screen: ScreenPoint) -> bool {
  let Some(context) = state::context_for_surface(window) else {
    return true;
  };
  let Ok(mut set) = context.surfaces.lock() else {
    return true;
  };
  let Some(surface) = set.find_mut(window) else {
    return true;
  };
  let (x, y) = surface.screen_to_client(screen.x, screen.y);
  hit_transparent(
    surface.input_enabled,
    surface.visible,
    surface.exclusion_rect,
    surface.logical_point(x, y),
  )
}

/// A region drag or a label drag owns every pointer move until it ends.
fn pointer_drag_active(window: WindowId) -> bool {
  state::context_for_surface(window).is_some_and(|context| {
    let region_drag = context
      .surfaces
      .lock()
      .map(|mut set| {
        set
          .find_mut(window)
          .is_some_and(|surface| surface.gesture_active)
      })
      .unwrap_or(false);
    let label_drag = context
      .ruler
      .lock()
      .map(|session| session.label_drag_active)
      .unwrap_or(false);
    region_drag || label_drag
  })
}

pub(crate) fn pointer_moved(window: WindowId, x: f64, y: f64, modifiers: u8) {
  // A pan owns the pointer until its button comes back up.
  if ruler_pan_drag(window, x, y) {
    return;
  }
  let phase = if pointer_drag_active(window) {
    PHASE_DRAG
  } else {
    PHASE_MOVE
  };
  dispatch(window, phase, x, y, modifiers);
}

pub(crate) fn pointer_down(window: WindowId, x: f64, y: f64, modifiers: u8, double_click: bool) {
  // A double click resets this display's viewport before the region gesture
  // ever sees it.
  if double_click && ruler_viewport(window, 3, x, y, Point::default()) {
    return;
  }
  dispatch(window, PHASE_DOWN, x, y, modifiers);
}

pub(crate) fn pointer_up(window: WindowId, x: f64, y: f64, modifiers: u8) {
  dispatch(window, PHASE_UP, x, y, modifiers);
}

pub(crate) fn pointer_cancelled(window: WindowId) {
  dispatch(window, PHASE_CANCEL, 0.0, 0.0, 0);
}

/// Hides the label under the pointer, or toggles the labels of whatever
/// artifact is there (`processRulerLabelRightClick`).
pub(crate) fn right_click(window: WindowId, x: f64, y: f64) -> bool {
  ruler_right_click(window, x, y)
}

pub(crate) fn pan_begin(window: WindowId, x: f64, y: f64) -> bool {
  ruler_pan_begin(window, x, y)
}

pub(crate) fn pan_end(window: WindowId) -> bool {
  ruler_pan_end(window)
}

/// A wheel turn of `notches` at `screen`. A notched vertical wheel zooms
/// around the pointer without a modifier; precision deltas and horizontal
/// turns pan, and `control` forces a zoom for either source.
pub(crate) fn wheel(
  window: WindowId,
  screen: ScreenPoint,
  notches: f64,
  horizontal: bool,
  notched: bool,
  control: bool,
) -> bool {
  let zoom = !horizontal && vertical_wheel_zooms(notches, notched, control);
  let delta = if zoom {
    Point {
      x: (notches * WHEEL_ZOOM_EXPONENT).exp(),
      y: 0.0,
    }
  } else if horizontal {
    Point {
      // A horizontal wheel's sign describes content scrolling. AppKit reports
      // the trackpad gesture itself, so invert it to keep Ruler's pan
      // direction identical on both platforms.
      x: -notches * WHEEL_PAN_POINTS,
      y: 0.0,
    }
  } else {
    Point {
      x: 0.0,
      y: notches * WHEEL_PAN_POINTS,
    }
  };
  ruler_viewport_screen(window, if zoom { 1 } else { 2 }, screen.x, screen.y, delta)
}

/// A trackpad pinch by `factor` at `screen`: the Ruler zooms about the
/// pointer, as a wheel turn with Control does.
#[cfg(target_os = "macos")]
pub(crate) fn magnify(window: WindowId, screen: ScreenPoint, factor: f64) -> bool {
  if !factor.is_finite() || factor <= 0.0 {
    return false;
  }
  ruler_viewport_screen(window, 1, screen.x, screen.y, Point { x: factor, y: 0.0 })
}

/// A trackpad's two-finger scroll of `dx`, `dy` points at `screen`: the
/// Ruler pans by it as it comes, as the gesture moves the content.
#[cfg(target_os = "macos")]
pub(crate) fn trackpad_pan(window: WindowId, screen: ScreenPoint, dx: f64, dy: f64) -> bool {
  ruler_viewport_screen(window, 2, screen.x, screen.y, Point { x: dx, y: dy })
}

/// Sets the pointer the window's surface asks for. Answers whether it had
/// one to set.
pub(crate) fn cursor_request(window: WindowId) -> bool {
  apply_cursor(window)
}

pub(crate) fn timer(window: WindowId, id: usize) {
  match id {
    ANIMATION_TIMER => animation_frame(window),
    CONFIRM_TIMER => confirm_expired(window),
    RULER_SETTLE_TIMER | RULER_COPIED_TIMER | RULER_TOLERANCE_TIMER => ruler_timer(window, id),
    _ => {}
  }
}

/// A key went down: `vk` a virtual-key code, `modifiers` as
/// [`super::host::RegionHost::keyboard_modifiers`] packs them.
pub(crate) fn key_down(window: WindowId, vk: u16, modifiers: u32, repeat: bool) -> bool {
  overlay_keyboard_command(window, vk, modifiers, repeat)
}

pub(crate) fn key_up(window: WindowId, vk: u16) -> bool {
  keyboard_release(window, vk)
}

/// Option or Alt changed while the pointer rests: the pointer is sampled
/// again under it.
pub(crate) fn alt_changed(window: WindowId, alt: bool) -> bool {
  super::modifier_changed(window, alt)
}

/// The displays changed under one of the overlay's windows.
pub(crate) fn display_changed(window: WindowId) {
  state::notify_layout_changed_for_surface(window);
}
