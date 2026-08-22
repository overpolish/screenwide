// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{thread, time::Duration};

use core_graphics::{
  display::CGDisplay,
  event::{CGEvent, CGEventTapLocation, ScrollEventUnit},
  event_source::{CGEventSource, CGEventSourceStateID},
  geometry::CGPoint,
};

use super::{Axis, Direction, ScreenPoint};

pub(super) fn place_pointer(point: ScreenPoint) -> Result<ScreenPoint, String> {
  let source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState)
    .map_err(|()| "Could not read the cursor position".to_owned())?;
  let current = CGEvent::new(source)
    .map_err(|()| "Could not read the cursor position".to_owned())?
    .location();
  CGDisplay::warp_mouse_cursor_position(CGPoint::new(point.x, point.y))
    .map_err(|error| format!("Could not position the cursor over the scrolling region: {error}"))?;
  Ok(ScreenPoint {
    x: current.x,
    y: current.y,
  })
}

pub(super) fn restore_pointer(point: ScreenPoint) {
  let _ = CGDisplay::warp_mouse_cursor_position(CGPoint::new(point.x, point.y));
}

pub(super) fn send_scroll(
  point: ScreenPoint,
  axis: Axis,
  direction: Direction,
  amount: u32,
) -> Result<(), String> {
  let source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState)
    .map_err(|()| "Could not create a scroll event".to_owned())?;
  let steps = 3_i32;
  let magnitude = ((amount as i32 + steps - 1) / steps).max(1);
  let signed = if direction == Direction::Forward {
    -magnitude
  } else {
    magnitude
  };

  for _ in 0..steps {
    let (vertical, horizontal) = match axis {
      Axis::Vertical => (signed, 0),
      Axis::Horizontal => (0, signed),
    };
    let event = CGEvent::new_scroll_event(
      source.clone(),
      ScrollEventUnit::PIXEL,
      2,
      vertical,
      horizontal,
      0,
    )
    .map_err(|()| "Could not create a scroll event".to_owned())?;
    event.set_location(CGPoint::new(point.x, point.y));
    event.post(CGEventTapLocation::HID);
    thread::sleep(Duration::from_millis(3));
  }
  Ok(())
}
