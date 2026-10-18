// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! AppKit's events as the region engine's. A local monitor sees every event
//! the app receives while an overlay is up: a pointer event over an overlay
//! is the engine's unless its hit test lets it through, a press holds the
//! pointer until it is released, and the keyboard goes to the overlay that
//! captured it. A swallowed event never reaches the webview below.

mod pointer;

use std::cell::{Cell, RefCell};
use std::ptr::NonNull;

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, NSObjectProtocol, ProtocolObject};
use objc2_app_kit::{
  NSApplicationDidChangeScreenParametersNotification, NSApplicationDidResignActiveNotification,
  NSEvent, NSEventMask, NSEventModifierFlags, NSEventType,
};
use objc2_foundation::{NSNotification, NSNotificationCenter};

use super::super::region_osc::host::WindowId;
use super::super::region_osc::input as engine;
use super::{geometry, windows};

type Observer = Retained<ProtocolObject<dyn NSObjectProtocol>>;

thread_local! {
  static MONITOR: RefCell<Option<Retained<AnyObject>>> = const { RefCell::new(None) };
  static OBSERVERS: RefCell<Vec<Observer>> = const { RefCell::new(Vec::new()) };
  /// The overlay a press is held by until its release.
  static CAPTURED: Cell<Option<WindowId>> = const { Cell::new(None) };
  /// The overlay the keyboard is captured for.
  pub(super) static KEYBOARD: Cell<Option<WindowId>> = const { Cell::new(None) };
}

/// Starts watching events, once, while any overlay is up.
pub(super) fn install() {
  MONITOR.with(|monitor| {
    if monitor.borrow().is_some() {
      return;
    }
    let mask = NSEventMask::MouseMoved
      | NSEventMask::LeftMouseDown
      | NSEventMask::LeftMouseDragged
      | NSEventMask::LeftMouseUp
      | NSEventMask::RightMouseDown
      | NSEventMask::OtherMouseDown
      | NSEventMask::OtherMouseDragged
      | NSEventMask::OtherMouseUp
      | NSEventMask::ScrollWheel
      | NSEventMask::KeyDown
      | NSEventMask::KeyUp
      | NSEventMask::FlagsChanged;
    let handler = RcBlock::new(|event: NonNull<NSEvent>| -> *mut NSEvent {
      // Safety: AppKit hands over a live event for the call.
      let event_ref = unsafe { event.as_ref() };
      let consumed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| handle(event_ref)))
        .unwrap_or_else(|_| {
          eprintln!("The macOS region overlay dropped an event after a panic");
          false
        });
      if consumed {
        std::ptr::null_mut()
      } else {
        event.as_ptr()
      }
    });
    // Safety: the handler returns the event it was given, or nothing.
    let installed =
      unsafe { NSEvent::addLocalMonitorForEventsMatchingMask_handler(mask, &handler) };
    *monitor.borrow_mut() = installed;
    observe_app();
  });
}

/// The displays changing and the app losing focus, which the engine hears
/// as Windows tells its windows: a topology change, and a press cancelled.
fn observe_app() {
  let center = NSNotificationCenter::defaultCenter();
  let displays = RcBlock::new(|_: NonNull<NSNotification>| {
    if let Some(mtm) = objc2::MainThreadMarker::new() {
      geometry::refresh_screens(mtm);
    }
    let mut peers = Vec::new();
    windows::each(|id, _| {
      peers.push(id);
      false
    });
    for window in peers {
      engine::display_changed(window);
    }
  });
  let resigned = RcBlock::new(|_: NonNull<NSNotification>| {
    if let Some(window) = CAPTURED.with(|captured| captured.take()) {
      engine::pointer_cancelled(window);
    }
  });
  // Safety: the names are AppKit's own constants, and the blocks live as
  // long as the observers, which are removed with the monitor.
  let observers = unsafe {
    [
      center.addObserverForName_object_queue_usingBlock(
        Some(NSApplicationDidChangeScreenParametersNotification),
        None,
        None,
        &displays,
      ),
      center.addObserverForName_object_queue_usingBlock(
        Some(NSApplicationDidResignActiveNotification),
        None,
        None,
        &resigned,
      ),
    ]
  };
  OBSERVERS.with(|kept| kept.borrow_mut().extend(observers));
}

/// Stops watching once the last overlay is gone.
pub(super) fn uninstall_if_unused() {
  if !windows::is_empty() {
    return;
  }
  MONITOR.with(|monitor| {
    if let Some(installed) = monitor.borrow_mut().take() {
      // Safety: the monitor was returned by `addLocalMonitor`.
      unsafe { NSEvent::removeMonitor(&installed) };
    }
  });
  let center = NSNotificationCenter::defaultCenter();
  for observer in OBSERVERS.with(|kept| std::mem::take(&mut *kept.borrow_mut())) {
    // Safety: each observer was returned by `addObserverForName`.
    unsafe { center.removeObserver(observer.as_ref()) };
  }
  CAPTURED.with(|captured| captured.set(None));
}

pub(super) fn release_pointer() {
  CAPTURED.with(|captured| captured.set(None));
}

/// The held keys as the engine packs them: bit 0 Command, 1 Control, 2
/// Option, 3 Shift.
pub(super) fn key_modifiers(flags: NSEventModifierFlags) -> u32 {
  u32::from(flags.contains(NSEventModifierFlags::Command))
    | (u32::from(flags.contains(NSEventModifierFlags::Control)) << 1)
    | (u32::from(flags.contains(NSEventModifierFlags::Option)) << 2)
    | (u32::from(flags.contains(NSEventModifierFlags::Shift)) << 3)
}

/// Hands `event` to the engine. Answers whether it was the engine's.
fn handle(event: &NSEvent) -> bool {
  let kind = event.r#type();
  match kind {
    NSEventType::KeyDown | NSEventType::KeyUp | NSEventType::FlagsChanged => keyboard(event, kind),
    _ => pointer::handle(event, kind),
  }
}

fn keyboard(event: &NSEvent, kind: NSEventType) -> bool {
  let Some(target) = KEYBOARD.with(Cell::get) else {
    return false;
  };
  match kind {
    NSEventType::KeyDown => engine::key_down(
      target,
      event.keyCode(),
      key_modifiers(event.modifierFlags()),
      event.isARepeat(),
    ),
    NSEventType::KeyUp => engine::key_up(target, event.keyCode()),
    _ => {
      let alt = event.modifierFlags().contains(NSEventModifierFlags::Option);
      engine::alt_changed(target, alt);
      // Modifier changes always reach the rest of the app too.
      false
    }
  }
}
