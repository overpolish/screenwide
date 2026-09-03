// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Window procedure for the region overlay, porting `+input.m`'s
//! `processInput` ordering minus the ruler steps.
//!
//! macOS installed a global `NSEvent` monitor and returned `nil` to swallow;
//! Win32 gives the overlay the events it hit-tests into, so pass-through is
//! expressed as `HTTRANSPARENT` instead.

use windows::{
  core::PCWSTR,
  Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    UI::{
      Input::KeyboardAndMouse::{GetKeyState, ReleaseCapture, SetCapture, VK_CONTROL, VK_SHIFT},
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

/// 16ms control-transition frames — the Win32 form of the self-rescheduling
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

pub(super) unsafe extern "system" fn window_proc(
  hwnd: HWND,
  message: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  match message {
    WM_NCHITTEST => {
      let transparent = guard(|| hit_test(hwnd, lparam)).unwrap_or(true);
      LRESULT(if transparent {
        HTTRANSPARENT as isize
      } else {
        HTCLIENT as isize
      })
    }
    // The overlay never takes activation or focus; the webview keeps both.
    WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
    WM_MOUSEMOVE => {
      let (x, y) = client_point(lparam);
      // A middle-button pan owns the pointer until the button comes back up.
      if guard(|| ruler_pan_drag(hwnd, x, y)).unwrap_or(false) {
        return LRESULT(0);
      }
      let dragging = guard(|| pointer_drag_active(hwnd)).unwrap_or(false);
      dispatch(
        hwnd,
        if dragging { PHASE_DRAG } else { PHASE_MOVE },
        x,
        y,
        modifier_bits(wparam.0, false, alt_pressed()),
      );
      LRESULT(0)
    }
    WM_LBUTTONDOWN | WM_LBUTTONDBLCLK => {
      SetCapture(hwnd);
      let (x, y) = client_point(lparam);
      // A double click resets this display's viewport before the region
      // gesture ever sees it (`+input.m:645-649`).
      if message == WM_LBUTTONDBLCLK
        && guard(|| ruler_viewport(hwnd, 3, x, y, Point::default())).unwrap_or(false)
      {
        return LRESULT(0);
      }
      let alt = alt_pressed();
      dispatch(
        hwnd,
        PHASE_DOWN,
        x,
        y,
        modifier_bits(wparam.0, message == WM_LBUTTONDBLCLK, alt),
      );
      LRESULT(0)
    }
    // Right-click hides the label under the pointer, or toggles the labels of
    // whatever artifact is there (`processRulerLabelRightClick`).
    WM_RBUTTONDOWN => {
      let (x, y) = client_point(lparam);
      if guard(|| ruler_right_click(hwnd, x, y)).unwrap_or(false) {
        LRESULT(0)
      } else {
        DefWindowProcW(hwnd, message, wparam, lparam)
      }
    }
    WM_MBUTTONDOWN => {
      let (x, y) = client_point(lparam);
      if guard(|| ruler_pan_begin(hwnd, x, y)).unwrap_or(false) {
        SetCapture(hwnd);
        LRESULT(0)
      } else {
        DefWindowProcW(hwnd, message, wparam, lparam)
      }
    }
    WM_MBUTTONUP => {
      if guard(|| ruler_pan_end(hwnd)).unwrap_or(false) {
        let _ = ReleaseCapture();
        LRESULT(0)
      } else {
        DefWindowProcW(hwnd, message, wparam, lparam)
      }
    }
    // A notched mouse wheel zooms around the pointer without a modifier.
    // Precision deltas and horizontal wheel input remain two-axis panning;
    // Ctrl is retained as an explicit zoom override.
    WM_MOUSEWHEEL | WM_MOUSEHWHEEL => {
      let wheel_delta = (wparam.0 >> 16) as u16 as i16;
      let notches = f64::from(wheel_delta) / f64::from(WHEEL_DELTA);
      let zoom =
        message == WM_MOUSEWHEEL && vertical_wheel_zooms(wheel_delta, wparam.0 & MK_CONTROL != 0);
      let delta = if zoom {
        Point {
          x: (notches * WHEEL_ZOOM_EXPONENT).exp(),
          y: 0.0,
        }
      } else if message == WM_MOUSEHWHEEL {
        Point {
          // Win32's horizontal wheel sign describes content scrolling. AppKit
          // reports the trackpad gesture itself, so invert it to keep Ruler's
          // pan direction identical on both platforms.
          x: -notches * WHEEL_PAN_POINTS,
          y: 0.0,
        }
      } else {
        Point {
          x: 0.0,
          y: notches * WHEEL_PAN_POINTS,
        }
      };
      let (screen_x, screen_y) = screen_point(lparam);
      if guard(|| ruler_viewport_screen(hwnd, if zoom { 1 } else { 2 }, screen_x, screen_y, delta))
        .unwrap_or(false)
      {
        LRESULT(0)
      } else {
        DefWindowProcW(hwnd, message, wparam, lparam)
      }
    }
    WM_LBUTTONUP => {
      let (x, y) = client_point(lparam);
      dispatch(
        hwnd,
        PHASE_UP,
        x,
        y,
        modifier_bits(wparam.0, false, alt_pressed()),
      );
      let _ = ReleaseCapture();
      LRESULT(0)
    }
    WM_CANCELMODE | WM_CAPTURECHANGED => {
      dispatch(hwnd, PHASE_CANCEL, 0.0, 0.0, 0);
      LRESULT(0)
    }
    WM_SETCURSOR => {
      if guard(|| apply_cursor(hwnd)).unwrap_or(false) {
        LRESULT(1)
      } else {
        DefWindowProcW(hwnd, message, wparam, lparam)
      }
    }
    WM_TIMER => {
      match wparam.0 {
        ANIMATION_TIMER => guard(|| animation_frame(hwnd)),
        CONFIRM_TIMER => guard(|| confirm_expired(hwnd)),
        RULER_SETTLE_TIMER => guard(|| ruler_timer(hwnd, RULER_SETTLE_TIMER)),
        RULER_COPIED_TIMER => guard(|| ruler_timer(hwnd, RULER_COPIED_TIMER)),
        RULER_TOLERANCE_TIMER => guard(|| ruler_timer(hwnd, RULER_TOLERANCE_TIMER)),
        _ => Some(()),
      };
      LRESULT(0)
    }
    OVERLAY_KEY_EVENT => {
      let flags = lparam.0;
      let handled = if flags & 16 != 0 {
        let alt = flags & 32 != 0;
        guard(|| modifier_changed(hwnd, alt)).unwrap_or(false)
      } else if flags & 8 != 0 {
        guard(|| keyboard_release(hwnd, wparam)).unwrap_or(false)
      } else {
        guard(|| {
          overlay_keyboard_command(
            hwnd,
            wparam.0 as u16,
            flags & 1 != 0,
            flags & 2 != 0,
            flags & 4 != 0,
          )
        })
        .unwrap_or(false)
      };
      LRESULT(isize::from(handled))
    }
    RULER_CURSOR_EVENT => {
      let _ = guard(|| apply_cursor(hwnd));
      LRESULT(0)
    }
    // The overlay remains `WS_EX_NOACTIVATE` for pointer input, but the Ruler
    // explicitly focuses this child while active so keyboard commands arrive
    // directly. The low-level monitor remains the fallback for focus changes.
    // Holding Alt changes ordinary key messages into system-key messages.
    // Feed both families into the same Ruler command path; bare Alt itself is
    // consumed here so DefWindowProc cannot enter menu-activation mode.
    WM_SYSKEYDOWN | WM_SYSKEYUP if matches!(wparam.0 as u32, 0x12 | 0xa4 | 0xa5) => LRESULT(0),
    WM_KEYDOWN | WM_SYSKEYDOWN => {
      let handled = guard(|| keyboard_command(hwnd, wparam, lparam)).unwrap_or(false);
      if handled {
        LRESULT(0)
      } else {
        DefWindowProcW(hwnd, message, wparam, lparam)
      }
    }
    WM_KEYUP | WM_SYSKEYUP => {
      if guard(|| keyboard_release(hwnd, wparam)).unwrap_or(false) {
        LRESULT(0)
      } else {
        DefWindowProcW(hwnd, message, wparam, lparam)
      }
    }
    // Peers are top-level, so they are the surfaces Windows tells about a
    // topology change. The notification is coalesced: it arrives once per peer.
    WM_DISPLAYCHANGE | WM_DPICHANGED => {
      guard(|| state::notify_layout_changed_for_surface(hwnd));
      LRESULT(0)
    }
    _ => DefWindowProcW(hwnd, message, wparam, lparam),
  }
}

/// `window_proc` is an `extern "system"` callback: a panic that reaches it
/// cannot unwind and aborts the process.
fn guard<T>(work: impl FnOnce() -> T) -> Option<T> {
  std::panic::catch_unwind(std::panic::AssertUnwindSafe(work))
    .inspect_err(|_| eprintln!("The Windows region OSC dropped an input after a panic"))
    .ok()
}

fn hit_test(hwnd: HWND, lparam: LPARAM) -> bool {
  let Some(context) = state::context_for_surface(hwnd) else {
    return true;
  };
  let Ok(mut set) = context.surfaces.lock() else {
    return true;
  };
  let Some(surface) = set.find_mut(hwnd) else {
    return true;
  };
  let (screen_x, screen_y) = screen_point(lparam);
  let (x, y) = surface.screen_to_client(screen_x, screen_y);
  hit_transparent(
    surface.input_enabled,
    surface.visible,
    surface.exclusion_rect,
    surface.logical_point(x, y),
  )
}

fn pointer_drag_active(hwnd: HWND) -> bool {
  state::context_for_surface(hwnd).is_some_and(|context| {
    let region_drag = context
      .surfaces
      .lock()
      .map(|mut set| {
        set
          .find_mut(hwnd)
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

fn apply_cursor(hwnd: HWND) -> bool {
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

fn set_cursor(shape: CursorShape) -> bool {
  let Some(name) = shape.name() else {
    return false;
  };
  if let Ok(cursor) = unsafe { LoadCursorW(None, name) } {
    unsafe { SetCursor(Some(cursor)) };
    return true;
  }
  false
}

/// Re-samples the pointer when Option/Alt changes. Unlike a mouse move, a
/// modifier transition carries no HWND for the display beneath the pointer,
/// so find the owning compositor surface before replaying the move.
fn modifier_changed(hwnd: HWND, alt: bool) -> bool {
  let Some(context) = state::context_for_surface(hwnd) else {
    return false;
  };
  let Some(pointer) = super::surface::cursor_position() else {
    return false;
  };
  let target = context.surfaces.lock().ok().and_then(|mut set| {
    set
      .all_mut()
      .find(|surface| {
        surface.input_enabled && surface.visible && surface.contains_screen_point(pointer)
      })
      .map(|surface| {
        let (x, y) = surface.screen_to_client(pointer.x, pointer.y);
        (surface.hwnd(), x, y)
      })
  });
  let Some((target, x, y)) = target else {
    return false;
  };
  process(
    &context,
    target,
    PHASE_MOVE,
    x,
    y,
    modifier_bits(0, false, alt),
  );
  true
}

fn dispatch(hwnd: HWND, phase: u32, x: f64, y: f64, modifiers: u8) {
  guard(|| {
    if let Some(context) = state::context_for_surface(hwnd) {
      process(&context, hwnd, phase, x, y, modifiers);
    }
  });
}

/// Port of `processInput` (`+input.m:175-295`), keeping its ordering. `hwnd`
/// names the surface the event landed on: each window only receives its own
/// messages, which is how macOS's `event.window != s.host.window` bail is
/// satisfied for free.
fn process(context: &Context, hwnd: HWND, phase: u32, x: f64, y: f64, modifiers: u8) {
  // Step 3: the OCR chrome is offered the event before anything else, and a
  // consumed event never reaches the region gesture. The command it activates
  // is dispatched after the surface lock is released, because the runtime
  // reaches back into the compositor through `set_ocr`.
  let Some((point, desktop_point)) = ocr_control_step(context, hwnd, phase, x, y) else {
    return;
  };
  // Steps 4 and 5: the ruler's floating labels are offered the event before
  // the region gesture, exactly where macOS put them.
  if ruler_label_step(context, hwnd, phase, point, desktop_point) {
    return;
  }

  let Ok(mut set) = context.surfaces.lock() else {
    return;
  };
  let Some(surface) = set.find_mut(hwnd) else {
    return;
  };
  // Drags arrive after a rejected press. Each of them would otherwise leave
  // the drawing crosshair behind, so ownership is handed back instead.
  if (phase == PHASE_DRAG || phase == PHASE_UP) && !surface.gesture_active {
    surface.release_pointer();
    return;
  }
  // Step 7: a press dismisses the cancel button before the region sees it.
  if phase == PHASE_DOWN && set.all_mut().any(|surface| surface.ocr.cancel_visible) {
    for surface in set.all_mut() {
      surface.ocr.set_cancel_visible(false);
      surface.draw();
    }
  }
  let Some(surface) = set.find_mut(hwnd) else {
    return;
  };
  // The exclusion rect is the webview's own toolbar.
  if phase == PHASE_DOWN
    && surface.exclusion_rect.size.width > 0.0
    && surface.exclusion_rect.size.height > 0.0
    && surface.exclusion_rect.contains(point)
  {
    return;
  }
  drop(set);

  let result = state::dispatch_input(context, phase, desktop_point, modifiers);
  if result.status == STATUS_INVALID {
    // A non-drawing Region editor rejects presses outside its committed
    // region; the press must not leave a claimed pointer behind.
    if phase == PHASE_DOWN {
      let _ = unsafe { ReleaseCapture() };
      if let Ok(mut set) = context.surfaces.lock() {
        if let Some(surface) = set.find_mut(hwnd) {
          surface.release_pointer();
        }
      }
    }
    return;
  }
  // Step 10: a ruler-flagged result re-pulls the whole draw set and redraws
  // every surface before the region geometry below is even considered.
  state::apply_ruler_result(context, &result);

  let Ok(mut set) = context.surfaces.lock() else {
    return;
  };
  let Some(surface) = set.find_mut(hwnd) else {
    return;
  };
  if phase == PHASE_DOWN {
    surface.gesture_active = true;
  }
  if phase == PHASE_UP || phase == PHASE_CANCEL {
    surface.gesture_active = false;
  }
  if result.cursor != 0 && surface.input_enabled {
    let shape = cursor_shape(result.cursor, result.handle);
    surface.cursor = shape;
    set_cursor(shape);
  }
  let visible = surface.visible;
  // Step 10: pointer capture keeps drag messages on the surface where the
  // resize began. Route the lens by its desktop point instead, then convert
  // the anchor into the destination surface's coordinates.
  let magnifier_changed = magnifier::route(&mut set, desktop_point, &result, phase);
  let redraw_only = !(result.ruler_flags & 1 == 0
    && (result.status == 1 || result.status == 2 || result.status == 3));
  if redraw_only {
    if magnifier_changed {
      for surface in set.all_mut() {
        surface.draw();
      }
    }
    return;
  }
  let region = if result.has_region == 0 {
    Rect::default()
  } else {
    Rect::from_xywh(result.x, result.y, result.width, result.height)
  };
  // One desktop-global region reaches every surface; each subtracts its own
  // offset, so a frame spanning a seam stays continuous.
  for surface in set.all_mut() {
    surface.set_region(region, visible);
  }
}

/// Runs the OCR chrome's share of `processInput`. Returns the surface-local
/// and desktop points when the event should continue to the region gesture,
/// and `None` when the chrome consumed it or the surface takes no input.
fn ocr_control_step(
  context: &Context,
  hwnd: HWND,
  phase: u32,
  x: f64,
  y: f64,
) -> Option<(Point, Point)> {
  let (points, dispatch) = {
    let mut set = context.surfaces.lock().ok()?;
    let surface = set.find_mut(hwnd)?;
    if !surface.input_enabled {
      return None;
    }
    let point = surface.logical_point(x, y);
    // The controller and every semantic event live in the desktop plane.
    let desktop_point = surface.desktop_point(point);
    let outcome = surface.ocr.control_input(point, phase);
    if outcome.redraw {
      surface.draw();
    }
    if outcome.consumed {
      // macOS set the pointing-hand cursor for the duration of the hover.
      surface.cursor = CursorShape::Hand;
      set_cursor(CursorShape::Hand);
    }
    if outcome.arm_confirm {
      let _ = unsafe { SetTimer(Some(hwnd), CONFIRM_TIMER, CONFIRM_TIMEOUT_MS, None) };
    }
    if outcome.disarm_confirm {
      let _ = unsafe { KillTimer(Some(hwnd), CONFIRM_TIMER) };
    }
    (
      (!outcome.consumed).then_some((point, desktop_point)),
      outcome.dispatch,
    )
  };
  if let Some(phase) = dispatch {
    state::dispatch_input(context, phase, Point::default(), 0);
  }
  points
}

fn animation_frame(hwnd: HWND) {
  let Some(context) = state::context_for_surface(hwnd) else {
    return;
  };
  let Ok(mut set) = context.surfaces.lock() else {
    return;
  };
  if let Some(surface) = set.find_mut(hwnd) {
    // Redrawing re-evaluates the animation and stops the timer once every
    // transition has settled.
    surface.draw();
  }
}

/// The three one-shot ruler timers. Each is killed on arrival because
/// `SetTimer` repeats and macOS scheduled a single `dispatch_after`.
fn ruler_timer(hwnd: HWND, timer: usize) {
  let _ = unsafe { KillTimer(Some(hwnd), timer) };
  let Some(context) = state::context_for_surface(hwnd) else {
    return;
  };
  match timer {
    RULER_SETTLE_TIMER => state::ruler_settle_frame(&context),
    RULER_COPIED_TIMER => state::ruler_expire_copied(&context),
    RULER_TOLERANCE_TIMER => state::ruler_expire_tolerance(&context),
    _ => {}
  }
}

/// Applies a result's cursor to the surface that produced it. macOS ran this
/// as `applyCursor` after every ruler result.
fn apply_result_cursor(context: &Context, hwnd: HWND, result: &OscResult) {
  if result.cursor == 0 {
    return;
  }
  let shape = cursor_shape(result.cursor, result.handle);
  let applied = context
    .surfaces
    .lock()
    .map(|mut set| {
      set.find_mut(hwnd).is_some_and(|surface| {
        if !surface.input_enabled {
          return false;
        }
        surface.cursor = shape;
        true
      })
    })
    .unwrap_or(false);
  if applied {
    set_cursor(shape);
  }
}

/// Port of `processRulerViewportInput` (`+input.m:381-398`) with the anchor
/// already in this surface's client pixels.
fn ruler_viewport(hwnd: HWND, operation: u32, x: f64, y: f64, delta: Point) -> bool {
  let Some(context) = state::context_for_surface(hwnd) else {
    return false;
  };
  if !context.is_ruler() {
    return false;
  }
  let anchor = {
    let Ok(mut set) = context.surfaces.lock() else {
      return false;
    };
    let Some(surface) = set.find_mut(hwnd) else {
      return false;
    };
    if !surface.input_enabled {
      return false;
    }
    (surface.display_id, surface.logical_point(x, y))
  };
  let result = context.ruler_viewport_input(anchor.0, operation, anchor.1, delta);
  if result.status == STATUS_INVALID {
    return false;
  }
  if let Ok(mut set) = context.surfaces.lock() {
    if let Some(surface) = set.find_mut(hwnd) {
      surface.claim_pointer();
    }
  }
  state::apply_ruler_result(&context, &result);
  apply_result_cursor(&context, hwnd, &result);
  true
}

/// The wheel reports screen coordinates, unlike every other pointer message.
fn ruler_viewport_screen(
  hwnd: HWND,
  operation: u32,
  screen_x: i32,
  screen_y: i32,
  delta: Point,
) -> bool {
  let Some(context) = state::context_for_surface(hwnd) else {
    return false;
  };
  let client = context.surfaces.lock().ok().and_then(|mut set| {
    set
      .find_mut(hwnd)
      .map(|s| s.screen_to_client(screen_x, screen_y))
  });
  let Some((x, y)) = client else {
    return false;
  };
  ruler_viewport(hwnd, operation, x, y, delta)
}

fn ruler_pan_begin(hwnd: HWND, x: f64, y: f64) -> bool {
  let Some(context) = state::context_for_surface(hwnd) else {
    return false;
  };
  if !context.is_ruler() {
    return false;
  }
  let point = context.surfaces.lock().ok().and_then(|mut set| {
    set
      .find_mut(hwnd)
      .filter(|surface| surface.input_enabled)
      .map(|surface| surface.logical_point(x, y))
  });
  let Some(point) = point else {
    return false;
  };
  let started = context
    .ruler
    .lock()
    .map(|mut session| {
      session.pan_last = Some(point);
    })
    .is_ok();
  started
}

fn ruler_pan_drag(hwnd: HWND, x: f64, y: f64) -> bool {
  let Some(context) = state::context_for_surface(hwnd) else {
    return false;
  };
  if !context.is_ruler() {
    return false;
  }
  let Some(last) = context
    .ruler
    .lock()
    .ok()
    .and_then(|session| session.pan_last)
  else {
    return false;
  };
  let point = context.surfaces.lock().ok().and_then(|mut set| {
    set
      .find_mut(hwnd)
      .map(|surface| surface.logical_point(x, y))
  });
  let Some(point) = point else {
    return false;
  };
  if let Ok(mut session) = context.ruler.lock() {
    session.pan_last = Some(point);
  }
  ruler_viewport(
    hwnd,
    2,
    x,
    y,
    Point {
      x: point.x - last.x,
      y: point.y - last.y,
    },
  )
}

fn ruler_pan_end(hwnd: HWND) -> bool {
  let Some(context) = state::context_for_surface(hwnd) else {
    return false;
  };
  context
    .ruler
    .lock()
    .map(|mut session| session.pan_last.take().is_some())
    .unwrap_or(false)
}

/// Port of `processRulerLabelRightClick` (`+input.m:321-344`): a label under
/// the pointer is hidden, an empty spot toggles the labels there.
fn ruler_right_click(hwnd: HWND, x: f64, y: f64) -> bool {
  let Some(context) = state::context_for_surface(hwnd) else {
    return false;
  };
  if !context.is_ruler() {
    return false;
  }
  let located = context.surfaces.lock().ok().and_then(|mut set| {
    set
      .find_mut(hwnd)
      .filter(|surface| surface.input_enabled && surface.ruler.visible)
      .map(|surface| {
        let point = surface.logical_point(x, y);
        (surface.ruler.label_hit(point), surface.desktop_point(point))
      })
  });
  let Some((hit, desktop_point)) = located else {
    return false;
  };
  let (operation, kind, id) = hit.map_or((6, 0, 0), |hit| (5, hit.kind, hit.id));
  let result = context.ruler_label_input(operation, kind, id, desktop_point, Point::default());
  if result.status == STATUS_INVALID {
    return false;
  }
  state::apply_ruler_result(&context, &result);
  apply_result_cursor(&context, hwnd, &result);
  true
}

/// Steps 4 and 5 of `processInput` (`+input.m:193-239`): an active label drag
/// owns every drag and up, then a hover or press over a label rectangle is a
/// label interaction rather than a region gesture. `true` means consumed.
fn ruler_label_step(
  context: &Context,
  hwnd: HWND,
  phase: u32,
  point: Point,
  desktop_point: Point,
) -> bool {
  if !context.is_ruler() {
    return false;
  }
  let dragging = context
    .ruler
    .lock()
    .map(|session| session.label_drag_active)
    .unwrap_or(false);
  if dragging && (phase == PHASE_DRAG || phase == PHASE_UP) {
    let result = context.ruler_label_input(
      if phase == PHASE_DRAG { 2 } else { 3 },
      0,
      0,
      desktop_point,
      Point::default(),
    );
    if phase == PHASE_UP {
      if let Ok(mut session) = context.ruler.lock() {
        session.label_drag_active = false;
      }
    }
    if result.status != STATUS_INVALID {
      state::apply_ruler_result(context, &result);
      apply_result_cursor(context, hwnd, &result);
    }
    return true;
  }
  if phase != PHASE_MOVE && phase != PHASE_DOWN {
    return false;
  }
  // A guide is being placed with a held key: the pointer belongs to it.
  if context
    .ruler
    .lock()
    .map(|session| session.guide_key != 0)
    .unwrap_or(false)
  {
    return false;
  }
  let located = context.surfaces.lock().ok().and_then(|mut set| {
    set.find_mut(hwnd).and_then(|surface| {
      surface
        .ruler
        .visible
        .then(|| surface.ruler.label_hit(point))
        .flatten()
        .map(|hit| (hit, surface.desktop_point(hit.center)))
    })
  });
  let Some((hit, center)) = located else {
    return false;
  };
  let result = if phase == PHASE_DOWN {
    let begin = context.ruler_label_input(1, hit.kind, hit.id, desktop_point, center);
    if begin.status != STATUS_INVALID {
      if let Ok(mut session) = context.ruler.lock() {
        session.label_drag_active = true;
      }
    }
    begin
  } else {
    context.ruler_label_input(7, hit.kind, hit.id, desktop_point, Point::default())
  };
  if result.status != STATUS_INVALID {
    state::apply_ruler_result(context, &result);
    apply_result_cursor(context, hwnd, &result);
  }
  true
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

/// The ruler's keyboard phases first, then Ctrl+A / Ctrl+C while a recognition
/// is ready — the Windows spelling of the macOS key-down monitor
/// (`+input.m:472-581`).
fn keyboard_command(hwnd: HWND, wparam: WPARAM, lparam: LPARAM) -> bool {
  let command = (unsafe { GetKeyState(VK_CONTROL.0 as i32) }) < 0;
  let shift = (unsafe { GetKeyState(VK_SHIFT.0 as i32) }) < 0;
  // Bit 30 of `lparam` is set when this key-down is an auto-repeat.
  let repeat = lparam.0 & (1 << 30) != 0;
  overlay_keyboard_command(hwnd, wparam.0 as u16, command, shift, repeat)
}

fn overlay_keyboard_command(hwnd: HWND, vk: u16, command: bool, shift: bool, repeat: bool) -> bool {
  let Some(context) = state::context_for_surface(hwnd) else {
    return false;
  };
  if context.is_ruler() {
    return ruler_keyboard_command(&context, hwnd, vk, command, shift, repeat);
  }
  if !context.is_text_recognition() {
    return false;
  }
  let Some(phase) = ocr_keyboard_phase(vk, command, repeat) else {
    return false;
  };
  let ready = context
    .surfaces
    .lock()
    .map(|mut set| {
      set
        .find_mut(hwnd)
        .is_some_and(|surface| surface.input_enabled && surface.ocr.phase == ocr::PHASE_READY)
    })
    .unwrap_or(false);
  if !ready {
    return false;
  }
  state::dispatch_input(&context, phase, Point::default(), 0);
  true
}

fn ruler_keyboard_command(
  context: &Context,
  hwnd: HWND,
  vk: u16,
  command: bool,
  shift: bool,
  repeat: bool,
) -> bool {
  let latched = context
    .ruler
    .lock()
    .map(|session| session.latched())
    .unwrap_or(false);
  let Some(key) = ruler::key_command(vk, command, shift, repeat, latched) else {
    return false;
  };
  if !ruler_keyboard_phase(context, hwnd, key.phase) {
    return false;
  }
  // A held key latches so the same physical press cannot re-fire and so its
  // key-up knows which release phase to send.
  if key.release.is_some() {
    if let Ok(mut session) = context.ruler.lock() {
      match key.phase {
        20 | 21 => session.range_key = vk,
        26 | 27 => session.guide_key = vk,
        31 => session.radius_key = vk,
        _ => {}
      }
    }
  }
  true
}

fn ocr_keyboard_phase(vk: u16, command: bool, repeat: bool) -> Option<u32> {
  if !command || repeat {
    return None;
  }
  match vk {
    0x41 => Some(6),
    0x43 => Some(7),
    _ => None,
  }
}

/// The key-up half of a latched range, guide or radius key (`+input.m:440-471`).
fn keyboard_release(hwnd: HWND, wparam: WPARAM) -> bool {
  let Some(context) = state::context_for_surface(hwnd) else {
    return false;
  };
  if !context.is_ruler() {
    return false;
  }
  let vk = wparam.0 as u16;
  let phase = {
    let Ok(mut session) = context.ruler.lock() else {
      return false;
    };
    if session.range_key == vk {
      session.range_key = 0;
      22
    } else if session.guide_key == vk {
      session.guide_key = 0;
      28
    } else if session.radius_key == vk {
      session.radius_key = 0;
      32
    } else {
      return false;
    }
  };
  ruler_keyboard_phase(&context, hwnd, phase);
  true
}

/// Port of `processKeyboardCommand` (`+input.m:299-319`).
fn ruler_keyboard_phase(context: &Context, hwnd: HWND, phase: u32) -> bool {
  let live = context
    .surfaces
    .lock()
    .map(|mut set| {
      set
        .find_mut(hwnd)
        .is_some_and(|surface| surface.input_enabled && surface.visible)
    })
    .unwrap_or(false);
  if !live {
    return false;
  }
  let result = state::dispatch_input(context, phase, Point::default(), 0);
  if result.status == STATUS_INVALID {
    return false;
  }
  state::apply_ruler_result(context, &result);
  apply_result_cursor(context, hwnd, &result);
  true
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
