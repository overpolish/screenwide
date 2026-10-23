// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The island's window on macOS: placed at the very top of its screen, so
//! the island can sit in the menu bar under the notch, and
//! letting clicks through everywhere but the island. Two event monitors
//! answer the pointer moving at once, a global one for while it is over other
//! apps and a local one for while it is over Screenwide's windows, and its
//! position is also read on a timer while the island is up: macOS sends the
//! app in front its pointer moves only through its focused window, so with
//! another of Screenwide's windows focused, such as the region being drawn,
//! neither monitor sees the pointer arrive on the island.

use std::cell::RefCell;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::MainThreadMarker;
use objc2_app_kit::{NSEvent, NSEventMask, NSScreen};
use objc2_foundation::{ns_string, NSNumber, NSPoint, NSRect, NSSize};
use tauri::{AppHandle, Manager};
use tauri_nspanel::PanelHandle;

use super::island::{self, IslandNotch, WINDOW_SIZE};
use super::{platform, webview_visibility, WindowLabel};

/// How often the pointer is looked for while it is on the island, quickly
/// enough to let clicks through as soon as it leaves onto the shadow.
const POLL_ON_ISLAND: Duration = Duration::from_millis(16);
/// How often it is looked for elsewhere: well inside the time between
/// reaching the island and clicking it.
const POLL_ELSEWHERE: Duration = Duration::from_millis(50);

/// Which showing of the island the pointer is being looked for in; none
/// when 0. A thread looks only while its own showing lasts, so hiding and
/// showing again quickly never leaves two looking.
static WATCHING: AtomicU64 = AtomicU64::new(0);
static NEXT_WATCH: AtomicU64 = AtomicU64::new(1);
/// The pointer is on the island, which sets how often it is looked for.
static OVER: AtomicBool = AtomicBool::new(false);

/// The island while its window is up: its panel and frame, whether the
/// pointer is on it, whether it is tucking away, and the monitors watching
/// the pointer.
struct Watch {
  frame: NSRect,
  is_hiding: bool,
  is_over: bool,
  monitors: Vec<Retained<AnyObject>>,
  panel: PanelHandle<tauri::Wry>,
}

thread_local! {
  static WATCH: RefCell<Option<Watch>> = const { RefCell::new(None) };
}

/// The notch, from the screen's safe area and the menu bar either side of
/// it: present only where both sides are.
fn notch_of(screen: &NSScreen) -> Option<IslandNotch> {
  let top = screen.safeAreaInsets().top;
  let left = screen.auxiliaryTopLeftArea();
  let right = screen.auxiliaryTopRightArea();
  if top <= 0.0 || left.size.width <= 0.0 || right.size.width <= 0.0 {
    return None;
  }
  Some(IslandNotch {
    height: top,
    width: screen.frame().size.width - left.size.width - right.size.width,
  })
}

/// The screen the island sits on: the display set for it while connected,
/// otherwise the primary display, the one with the menu bar, which AppKit
/// lists first.
fn island_screen(mtm: MainThreadMarker, preferred: Option<u32>) -> Option<Retained<NSScreen>> {
  let screens = NSScreen::screens(mtm);
  preferred
    .and_then(|id| screens.iter().find(|screen| display_id(screen) == Some(id)))
    .or_else(|| screens.firstObject())
}

/// A screen's CoreGraphics display id, the id displays are known by
/// elsewhere.
fn display_id(screen: &NSScreen) -> Option<u32> {
  let number = screen
    .deviceDescription()
    .objectForKey(ns_string!("NSScreenNumber"))?;
  number
    .downcast::<NSNumber>()
    .ok()
    .map(|number| number.unsignedIntValue())
}

/// The display the island belongs on now.
pub(super) fn target_display(app: &AppHandle) -> Option<u32> {
  let preferred = island::preferred_display(app);
  dispatch2::run_on_main(move |mtm| {
    island_screen(mtm, preferred)
      .as_deref()
      .and_then(display_id)
  })
}

/// Whether the pointer is on one of the island's parts. AppKit measures up
/// from the screen's bottom; the parts are measured down from the window's
/// top.
fn is_over(watch: &Watch) -> bool {
  let pointer = NSEvent::mouseLocation();
  let x = pointer.x - watch.frame.origin.x;
  let y = watch.frame.origin.y + watch.frame.size.height - pointer.y;
  !watch.is_hiding && island::is_on_island(&island::rects(), x, y)
}

/// Lets clicks reach the island while the pointer is on it, and pass through
/// while it is not, its shadow included. Only a change is sent to the window.
fn update() {
  WATCH.with(|watch| {
    if let Some(watch) = watch.borrow_mut().as_mut() {
      let over = is_over(watch);
      if over != watch.is_over {
        watch.is_over = over;
        watch.panel.set_ignores_mouse_events(!over);
      }
      OVER.store(over, Ordering::Relaxed);
    }
  });
}

fn poll(app: AppHandle) {
  let showing = NEXT_WATCH.fetch_add(1, Ordering::Relaxed);
  WATCHING.store(showing, Ordering::Relaxed);
  std::thread::spawn(move || {
    while WATCHING.load(Ordering::Relaxed) == showing {
      std::thread::sleep(if OVER.load(Ordering::Relaxed) {
        POLL_ON_ISLAND
      } else {
        POLL_ELSEWHERE
      });
      if app.run_on_main_thread(update).is_err() {
        break;
      }
    }
  });
}

fn monitors() -> Vec<Retained<AnyObject>> {
  let mask = NSEventMask::MouseMoved | NSEventMask::LeftMouseDragged;
  let global = RcBlock::new(|_: NonNull<NSEvent>| update());
  let local = RcBlock::new(|event: NonNull<NSEvent>| -> *mut NSEvent {
    update();
    event.as_ptr()
  });
  let mut installed = Vec::new();
  installed.extend(NSEvent::addGlobalMonitorForEventsMatchingMask_handler(
    mask, &global,
  ));
  // Safety: the handler returns the event it was given.
  installed.extend(unsafe { NSEvent::addLocalMonitorForEventsMatchingMask_handler(mask, &local) });
  installed
}

pub(super) fn show(app: &AppHandle) -> tauri::Result<()> {
  let window = app
    .get_webview_window(WindowLabel::Island.as_str())
    .ok_or(tauri::Error::WindowNotFound)?;
  let panel = platform::island_panel(&window)?;
  platform::restore_recording_level(&window)?;
  let preferred = island::preferred_display(app);
  let app = app.clone();
  window.clone().run_on_main_thread(move || {
    let Some(mtm) = MainThreadMarker::new() else {
      return;
    };
    let Some(screen) = island_screen(mtm, preferred) else {
      return;
    };
    island::set_placed_display(display_id(&screen));
    let bounds = screen.frame();
    let (width, height) = WINDOW_SIZE;
    let frame = NSRect::new(
      NSPoint::new(
        bounds.origin.x + (bounds.size.width - width) / 2.0,
        bounds.origin.y + bounds.size.height - height,
      ),
      NSSize::new(width, height),
    );
    panel.as_panel().setFrame_display(frame, false);
    island::announce_shown(&app, notch_of(&screen));

    // Shown as the other recording panels are, without taking the keyboard
    // from the app in front (`platform::show`), but passing clicks through
    // from the start.
    let _ = webview_visibility::show_webview(&window);
    panel.set_ignores_mouse_events(true);
    panel.set_alpha_value(1.0);
    panel.show();

    WATCH.with(|watch| {
      let mut watch = watch.borrow_mut();
      let installed = watch
        .take()
        .map(|old| old.monitors)
        .unwrap_or_else(monitors);
      *watch = Some(Watch {
        frame,
        is_hiding: false,
        is_over: false,
        monitors: installed,
        panel: panel.clone(),
      });
    });
    update();
    poll(app);
  })
}

/// Tucking away, the island takes no clicks, so a drag that crosses it as it
/// goes is not caught by it.
pub(super) fn set_hiding(app: &AppHandle, is_hiding: bool) {
  let _ = app.run_on_main_thread(move || {
    WATCH.with(|watch| {
      if let Some(watch) = watch.borrow_mut().as_mut() {
        watch.is_hiding = is_hiding;
      }
    });
    update();
  });
}

pub(super) fn refresh_hit_area(app: &AppHandle) {
  let _ = app.run_on_main_thread(update);
}

pub(super) fn hide(app: &AppHandle) -> tauri::Result<()> {
  let window = app
    .get_webview_window(WindowLabel::Island.as_str())
    .ok_or(tauri::Error::WindowNotFound)?;
  WATCHING.store(0, Ordering::Relaxed);
  OVER.store(false, Ordering::Relaxed);
  app.run_on_main_thread(|| {
    if let Some(watch) = WATCH.with(|watch| watch.borrow_mut().take()) {
      for monitor in watch.monitors {
        // Safety: each was returned by `addGlobal`/`addLocalMonitor...`.
        unsafe { NSEvent::removeMonitor(&monitor) };
      }
    }
  })?;
  platform::hide(&window)
}
