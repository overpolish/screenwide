// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Pointer events: the overlay under the pointer, or the one a press is
//! held by, and the point in its client pixels.

use std::cell::Cell;

use objc2_app_kit::{NSEvent, NSEventModifierFlags, NSEventType};
use objc2_foundation::{NSPoint, NSRect, NSSize};

use super::super::super::region_osc::host::{ScreenPoint, WindowId};
use super::super::super::region_osc::input::{self as engine, modifier_bits};
use super::super::{geometry, windows};
use super::CAPTURED;

/// The overlay `event` happened over, and the point in its client pixels.
fn overlay_under(event: &NSEvent) -> Option<(WindowId, (f64, f64))> {
  let window = event.window(objc2::MainThreadMarker::new()?)?;
  let location = event.locationInWindow();
  let mut found = None;
  windows::each(|id, overlay| {
    let ours = overlay
      .view
      .window()
      .is_some_and(|view_window| std::ptr::eq(&*view_window, &*window));
    if !ours || overlay.view.isHidden() {
      return false;
    }
    let local = overlay.view.convertPoint_fromView(location, None);
    let bounds = overlay.view.bounds();
    let inside = local.x >= 0.0
      && local.y >= 0.0
      && local.x < bounds.size.width
      && local.y < bounds.size.height;
    if inside {
      found = Some((id, client(&overlay.view, local)));
    }
    inside
  })?;
  found
}

/// A point in `view`'s points as the engine's client pixels, from the top
/// left.
fn client(view: &objc2_app_kit::NSView, local: NSPoint) -> (f64, f64) {
  let scale = view
    .window()
    .map_or(1.0, |window| window.backingScaleFactor());
  let top = if view.isFlipped() {
    local.y
  } else {
    view.bounds().size.height - local.y
  };
  (local.x * scale, top * scale)
}

/// Where `event` happened in `window`'s client pixels, for a held press that
/// may have left the view.
fn client_in(window: WindowId, event: &NSEvent) -> Option<(f64, f64)> {
  windows::with(window, |overlay| {
    let local = overlay
      .view
      .convertPoint_fromView(event.locationInWindow(), None);
    client(&overlay.view, local)
  })
}

fn screen_of(event: &NSEvent) -> ScreenPoint {
  let location = event.locationInWindow();
  let global = event
    .window(objc2::MainThreadMarker::new().expect("events arrive on the main thread"))
    .map_or(location, |window| {
      window
        .convertRectToScreen(NSRect::new(location, NSSize::new(0.0, 0.0)))
        .origin
    });
  geometry::screen_point(global)
}

fn modifiers(event: &NSEvent, double: bool) -> u8 {
  let flags = event.modifierFlags();
  modifier_bits(
    flags.contains(NSEventModifierFlags::Shift),
    flags.contains(NSEventModifierFlags::Control),
    double,
    flags.contains(NSEventModifierFlags::Option),
  )
}

/// Hands a pointer `event` to the overlay it is over or held by. Answers
/// whether it was the engine's.
pub(super) fn handle(event: &NSEvent, kind: NSEventType) -> bool {
  let captured = CAPTURED.with(Cell::get);
  let held = matches!(
    kind,
    NSEventType::LeftMouseDragged
      | NSEventType::LeftMouseUp
      | NSEventType::OtherMouseDragged
      | NSEventType::OtherMouseUp
  );
  let (window, (x, y)) = match captured.filter(|_| held) {
    Some(window) => match client_in(window, event) {
      Some(point) => (window, point),
      None => return false,
    },
    None => {
      let Some(found) = overlay_under(event) else {
        return false;
      };
      if engine::hit_test(found.0, screen_of(event)) {
        return false;
      }
      found
    }
  };
  match kind {
    NSEventType::MouseMoved | NSEventType::LeftMouseDragged | NSEventType::OtherMouseDragged => {
      engine::pointer_moved(window, x, y, modifiers(event, false));
      engine::cursor_request(window);
      true
    }
    NSEventType::LeftMouseDown => {
      let double = event.clickCount() == 2;
      CAPTURED.with(|captured| captured.set(Some(window)));
      engine::pointer_down(window, x, y, modifiers(event, double), double);
      engine::cursor_request(window);
      true
    }
    NSEventType::LeftMouseUp => {
      CAPTURED.with(|captured| captured.set(None));
      engine::pointer_up(window, x, y, modifiers(event, false));
      engine::cursor_request(window);
      true
    }
    NSEventType::RightMouseDown => engine::right_click(window, x, y),
    NSEventType::OtherMouseDown => {
      let began = engine::pan_begin(window, x, y);
      if began {
        CAPTURED.with(|captured| captured.set(Some(window)));
      }
      began
    }
    NSEventType::OtherMouseUp => {
      CAPTURED.with(|captured| captured.set(None));
      engine::pan_end(window)
    }
    NSEventType::ScrollWheel => {
      let precise = event.hasPreciseScrollingDeltas();
      let (dx, dy) = (event.scrollingDeltaX(), event.scrollingDeltaY());
      let horizontal = dx.abs() > dy.abs();
      let delta = if horizontal { dx } else { dy };
      // A wheel's notch is a line; a trackpad's points are finer.
      let notches = if precise { delta / 10.0 } else { delta };
      let control = event
        .modifierFlags()
        .contains(NSEventModifierFlags::Control);
      engine::wheel(
        window,
        screen_of(event),
        notches,
        horizontal,
        !precise,
        control,
      )
    }
    _ => false,
  }
}
