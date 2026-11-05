// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The desktop as references measure it on macOS: points from the primary
//! display's top left, as Tauri gives a window's logical position, where
//! AppKit counts from the bottom left.

use objc2::rc::Retained;
use objc2::MainThreadMarker;
use objc2_app_kit::NSScreen;
use objc2_foundation::{NSPoint, NSRect, NSSize};
use screenwide_osc::geometry::{Point, Rect, Size};

/// The primary display's height, which turns AppKit's bottom-left screen
/// coordinates into the desktop's top-left ones.
fn primary_height(mtm: MainThreadMarker) -> f64 {
  NSScreen::screens(mtm)
    .firstObject()
    .map_or(0.0, |screen| screen.frame().size.height)
}

pub(super) fn desktop_rect(mtm: MainThreadMarker, rect: NSRect) -> Rect {
  Rect::from_xywh(
    rect.origin.x,
    primary_height(mtm) - rect.origin.y - rect.size.height,
    rect.size.width,
    rect.size.height,
  )
}

pub(super) fn appkit_rect(mtm: MainThreadMarker, rect: Rect) -> NSRect {
  NSRect::new(
    NSPoint::new(
      rect.origin.x,
      primary_height(mtm) - rect.origin.y - rect.size.height,
    ),
    NSSize::new(rect.size.width, rect.size.height),
  )
}

/// The pointer on the desktop.
pub(super) fn pointer(mtm: MainThreadMarker) -> Point {
  let at = objc2_app_kit::NSEvent::mouseLocation();
  Point {
    x: at.x,
    y: primary_height(mtm) - at.y,
  }
}

/// The display `point` is on, else the main one.
fn display_at(mtm: MainThreadMarker, point: Point) -> Option<Retained<NSScreen>> {
  let screens = NSScreen::screens(mtm);
  screens
    .iter()
    .find(|screen| desktop_rect(mtm, screen.frame()).contains(point))
    .or_else(|| NSScreen::mainScreen(mtm))
}

/// The whole of the display `point` is on, else the main one.
pub(super) fn screen_at(mtm: MainThreadMarker, point: Point) -> Rect {
  display_at(mtm, point)
    .map(|screen| desktop_rect(mtm, screen.frame()))
    .unwrap_or_default()
}

/// The room for windows on the display `point` is on, else the main one:
/// clear of the menu bar and the Dock.
pub(super) fn room_at(mtm: MainThreadMarker, point: Point) -> Rect {
  display_at(mtm, point)
    .map(|screen| desktop_rect(mtm, screen.visibleFrame()))
    .unwrap_or_default()
}

/// The picture's corner for a reference with nowhere to return to: the
/// middle of the main display.
pub(super) fn centred(mtm: MainThreadMarker, size: Size) -> Point {
  let screen = NSScreen::mainScreen(mtm)
    .map(|screen| desktop_rect(mtm, screen.visibleFrame()))
    .unwrap_or_default();
  Point {
    x: (screen.origin.x + (screen.size.width - size.width) / 2.0).round(),
    y: (screen.origin.y + (screen.size.height - size.height) / 2.0).round(),
  }
}
