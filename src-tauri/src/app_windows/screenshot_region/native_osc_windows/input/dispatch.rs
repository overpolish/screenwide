// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn dispatch(hwnd: HWND, phase: u32, x: f64, y: f64, modifiers: u8) {
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
pub(super) fn process(context: &Context, hwnd: HWND, phase: u32, x: f64, y: f64, modifiers: u8) {
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
pub(super) fn ocr_control_step(
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

pub(super) fn animation_frame(hwnd: HWND) {
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
pub(super) fn ruler_timer(hwnd: HWND, timer: usize) {
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
pub(super) fn apply_result_cursor(context: &Context, hwnd: HWND, result: &OscResult) {
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
