// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! On macOS a press on the overlay picks the window before the page under it
//! sees the press: AppKit spends the first press on a window that is not key
//! making it key, whatever the page asks, so a pick made by the page alone
//! would take two. Presses anywhere else, the island's among them, go on as
//! usual.

use std::cell::RefCell;
use std::ptr::NonNull;

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_app_kit::{NSEvent, NSEventMask, NSWindow};
use tauri::{AppHandle, Manager};

use crate::app_windows::WindowLabel;

thread_local! {
  static MONITOR: RefCell<Option<Retained<AnyObject>>> = const { RefCell::new(None) };
}

pub(super) fn catch(app: &AppHandle) {
  let Some(overlay) = app.get_webview_window(WindowLabel::RegionSelector.as_str()) else {
    return;
  };
  let app = app.clone();
  let _ = overlay.clone().run_on_main_thread(move || {
    let Ok(ns_window) = overlay.ns_window() else {
      return;
    };
    // Safety: Tauri hands out the window's live NSWindow, used here on the
    // main thread it belongs to.
    let overlay_number = unsafe { &*ns_window.cast::<NSWindow>() }.windowNumber();
    let handler = RcBlock::new(move |event: NonNull<NSEvent>| -> *mut NSEvent {
      // Safety: AppKit hands the monitor a live event.
      if unsafe { event.as_ref() }.windowNumber() != overlay_number {
        return event.as_ptr();
      }
      let app = app.clone();
      // Off the event's turn: ending the pick emits and takes locks.
      std::thread::spawn(move || super::pick_hovered(&app));
      std::ptr::null_mut()
    });
    // Safety: the handler returns the event it was given, or none to
    // swallow it.
    let monitor = unsafe {
      NSEvent::addLocalMonitorForEventsMatchingMask_handler(NSEventMask::LeftMouseDown, &handler)
    };
    MONITOR.with(|slot| {
      if let Some(old) = slot.borrow_mut().replace(monitor?) {
        // Safety: returned by `addLocalMonitor...`.
        unsafe { NSEvent::removeMonitor(&old) };
      }
      Some(())
    });
  });
}

pub(super) fn release(app: &AppHandle) {
  let _ = app.run_on_main_thread(|| {
    MONITOR.with(|slot| {
      if let Some(monitor) = slot.borrow_mut().take() {
        // Safety: returned by `addLocalMonitor...`.
        unsafe { NSEvent::removeMonitor(&monitor) };
      }
    });
  });
}
