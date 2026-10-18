// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! AppKit's mouse and trackpad events as the engine's pointer input.

use objc2_app_kit::{NSEvent, NSEventModifierFlags};

use super::EditorView;
use screenwide_workspace::editing::PointerInput as Input;

fn held(event: &NSEvent, flag: NSEventModifierFlags) -> bool {
  event.modifierFlags().contains(flag)
}

pub(super) fn down(view: &EditorView, event: &NSEvent) {
  let (x, y) = view.event_point(event);
  // Control-click is the Mac's secondary click.
  if held(event, NSEventModifierFlags::Control) {
    view.pointer(Input::ContextMenu { x, y });
    return;
  }
  // A platform reports a double-click in place of its second press.
  if event.clickCount() == 2 {
    view.pointer(Input::DoubleClick { x, y });
    return;
  }
  view.pointer(Input::Down {
    centered: held(event, NSEventModifierFlags::Option),
    x,
    y,
    snapping: held(event, NSEventModifierFlags::Command),
  });
}

pub(super) fn moved(view: &EditorView, event: &NSEvent, pressed: bool) {
  let (x, y) = view.event_point(event);
  view.pointer(Input::Move {
    centered: held(event, NSEventModifierFlags::Option),
    x,
    y,
    pressed,
    snapping: held(event, NSEventModifierFlags::Command),
  });
}

pub(super) fn up(view: &EditorView, event: &NSEvent) {
  let (x, y) = view.event_point(event);
  view.pointer(Input::Up { x, y });
}

pub(super) fn context_menu(view: &EditorView, event: &NSEvent) {
  let (x, y) = view.event_point(event);
  view.pointer(Input::ContextMenu { x, y });
}

/// The middle button pans from wherever it lands.
pub(super) fn pan(view: &EditorView, event: &NSEvent, down: bool) {
  if event.buttonNumber() != 2 {
    return;
  }
  let (x, y) = view.event_point(event);
  view.pointer(if down {
    Input::PanDown { x, y }
  } else {
    Input::PanUp { x, y }
  });
}

/// A trackpad's scroll pans and a mouse wheel's zooms, as on Windows; with
/// Control held either zooms about the pointer.
pub(super) fn scroll(view: &EditorView, event: &NSEvent) {
  let (x, y) = view.event_point(event);
  let precise = event.hasPreciseScrollingDeltas();
  if held(event, NSEventModifierFlags::Control) || !precise {
    // A notch of a wheel scrolls a line; a trackpad's points are finer.
    let delta = if precise {
      event.scrollingDeltaY() / 10.0
    } else {
      event.scrollingDeltaY()
    };
    view.pointer(Input::Wheel { x, y, delta });
    return;
  }
  view.pointer(Input::Scroll {
    dx: event.scrollingDeltaX(),
    dy: event.scrollingDeltaY(),
  });
}

pub(super) fn magnify(view: &EditorView, event: &NSEvent) {
  let (x, y) = view.event_point(event);
  view.pointer(Input::Magnify {
    x,
    y,
    factor: 1.0 + event.magnification(),
  });
}
