// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{mem::size_of, thread, time::Duration};

use windows::Win32::{
  Foundation::POINT,
  UI::{
    Input::KeyboardAndMouse::{
      SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_HWHEEL, MOUSEEVENTF_WHEEL, MOUSEINPUT,
    },
    WindowsAndMessaging::{GetCursorPos, SetCursorPos},
  },
};

use super::{Axis, Direction, ScreenPoint};

pub(super) fn place_pointer(point: ScreenPoint) -> Result<ScreenPoint, String> {
  let mut current = POINT::default();
  unsafe { GetCursorPos(&mut current) }
    .map_err(|error| format!("Could not read the cursor position: {error}"))?;
  unsafe { SetCursorPos(point.x.round() as i32, point.y.round() as i32) }
    .map_err(|error| format!("Could not position the cursor over the scrolling region: {error}"))?;
  Ok(ScreenPoint {
    x: f64::from(current.x),
    y: f64::from(current.y),
  })
}

pub(super) fn restore_pointer(point: ScreenPoint) {
  let _ = unsafe { SetCursorPos(point.x.round() as i32, point.y.round() as i32) };
}

pub(super) fn send_scroll(
  _point: ScreenPoint,
  axis: Axis,
  direction: Direction,
  amount: u32,
) -> Result<(), String> {
  let notches = (amount / 80).clamp(2, 12);
  for _ in 0..notches {
    let forward = direction == Direction::Forward;
    let signed_delta = match axis {
      Axis::Vertical => {
        if forward {
          -120_i32
        } else {
          120_i32
        }
      }
      Axis::Horizontal => {
        if forward {
          120_i32
        } else {
          -120_i32
        }
      }
    };
    let flags = match axis {
      Axis::Vertical => MOUSEEVENTF_WHEEL,
      Axis::Horizontal => MOUSEEVENTF_HWHEEL,
    };
    let input = INPUT {
      r#type: INPUT_MOUSE,
      Anonymous: INPUT_0 {
        mi: MOUSEINPUT {
          mouseData: signed_delta as u32,
          dwFlags: flags,
          ..Default::default()
        },
      },
    };
    let sent = unsafe { SendInput(&[input], size_of::<INPUT>() as i32) };
    if sent != 1 {
      return Err("The target application did not accept the scroll event".to_owned());
    }
    thread::sleep(Duration::from_millis(3));
  }
  Ok(())
}
