// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The region engine's screen space on macOS. AppKit puts the origin at the
//! primary display's bottom left, in points; the engine works from the top
//! left in physical pixels, as Tauri reports monitors: each display's points
//! times its own backing scale. Converting needs the displays, which only
//! the main thread may read, so they are cached here for every thread.

use std::collections::HashMap;
use std::sync::Mutex;

use objc2::MainThreadMarker;
use objc2_app_kit::NSScreen;
use objc2_foundation::{NSPoint, NSRect};
use screenwide_osc::geometry::Rect;

use super::super::region_osc::host::ScreenPoint;

#[derive(Clone, Copy)]
struct Display {
  /// The display's frame in AppKit's global points.
  frame: NSRect,
  scale: f64,
}

#[derive(Default)]
struct Screens {
  primary_height: f64,
  displays: Vec<Display>,
}

/// Where each overlay window is, as the engine reads it.
#[derive(Clone, Copy)]
pub(super) struct Placement {
  /// Physical pixels from the global top left.
  pub frame: Rect,
  pub scale: f64,
}

static SCREENS: Mutex<Screens> = Mutex::new(Screens {
  primary_height: 0.0,
  displays: Vec::new(),
});
static PLACEMENTS: Mutex<Option<HashMap<isize, Placement>>> = Mutex::new(None);

/// Reads the displays again, as their arrangement may have changed.
pub(super) fn refresh_screens(mtm: MainThreadMarker) {
  let screens = NSScreen::screens(mtm);
  let displays: Vec<Display> = screens
    .iter()
    .map(|screen| Display {
      frame: screen.frame(),
      scale: screen.backingScaleFactor(),
    })
    .collect();
  let primary_height = displays
    .first()
    .map_or(0.0, |display| display.frame.size.height);
  if let Ok(mut cached) = SCREENS.lock() {
    *cached = Screens {
      primary_height,
      displays,
    };
  }
}

fn contains(frame: NSRect, point: NSPoint) -> bool {
  point.x >= frame.origin.x
    && point.x < frame.origin.x + frame.size.width
    && point.y >= frame.origin.y
    && point.y < frame.origin.y + frame.size.height
}

/// The scale of the display under `point`, in AppKit's global points.
fn scale_at(screens: &Screens, point: NSPoint) -> f64 {
  screens
    .displays
    .iter()
    .find(|display| contains(display.frame, point))
    .or_else(|| screens.displays.first())
    .map_or(1.0, |display| display.scale)
}

/// An AppKit global point as the engine's screen point.
pub(super) fn screen_point(point: NSPoint) -> ScreenPoint {
  let Ok(screens) = SCREENS.lock() else {
    return ScreenPoint::default();
  };
  let scale = scale_at(&screens, point);
  ScreenPoint {
    x: (point.x * scale).round() as i32,
    y: ((screens.primary_height - point.y) * scale).round() as i32,
  }
}

/// An AppKit global frame, at `scale`, as the engine's pixels.
pub(super) fn pixels(frame: NSRect, scale: f64) -> Rect {
  let primary_height = SCREENS.lock().map_or(0.0, |screens| screens.primary_height);
  let top = primary_height - frame.origin.y - frame.size.height;
  Rect::from_xywh(
    frame.origin.x * scale,
    top * scale,
    frame.size.width * scale,
    frame.size.height * scale,
  )
}

/// The engine's pixels as an AppKit global frame, scaled by the display
/// they lie on, and that scale.
pub(super) fn points(rect: Rect) -> (NSRect, f64) {
  let Ok(screens) = SCREENS.lock() else {
    return (NSRect::ZERO, 1.0);
  };
  let centre = (
    rect.origin.x + rect.size.width / 2.0,
    rect.origin.y + rect.size.height / 2.0,
  );
  let display = screens
    .displays
    .iter()
    .find(|display| {
      let pixels_frame = Rect::from_xywh(
        display.frame.origin.x * display.scale,
        (screens.primary_height - display.frame.origin.y - display.frame.size.height)
          * display.scale,
        display.frame.size.width * display.scale,
        display.frame.size.height * display.scale,
      );
      centre.0 >= pixels_frame.origin.x
        && centre.0 < pixels_frame.origin.x + pixels_frame.size.width
        && centre.1 >= pixels_frame.origin.y
        && centre.1 < pixels_frame.origin.y + pixels_frame.size.height
    })
    .copied();
  let scale = display.map_or(1.0, |display| display.scale);
  let (x, top) = (rect.origin.x / scale, rect.origin.y / scale);
  let (width, height) = (rect.size.width / scale, rect.size.height / scale);
  let frame = NSRect::new(
    NSPoint::new(x, screens.primary_height - top - height),
    objc2_foundation::NSSize::new(width, height),
  );
  (frame, scale)
}

pub(super) fn set_placement(window: isize, placement: Placement) {
  if let Ok(mut placements) = PLACEMENTS.lock() {
    placements
      .get_or_insert_with(HashMap::new)
      .insert(window, placement);
  }
}

pub(super) fn forget(window: isize) {
  if let Ok(mut placements) = PLACEMENTS.lock() {
    if let Some(placements) = placements.as_mut() {
      placements.remove(&window);
    }
  }
}

pub(super) fn placement(window: isize) -> Option<Placement> {
  PLACEMENTS.lock().ok()?.as_ref()?.get(&window).copied()
}
