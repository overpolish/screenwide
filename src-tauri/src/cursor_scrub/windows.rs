// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::{mpsc, Mutex};

use screenwide_keyboard_hook::Phase;
use tauri::ipc::Channel;
use windows::Win32::{
  Foundation::POINT,
  UI::{
    Input::KeyboardAndMouse::{GetAsyncKeyState, VK_ESCAPE, VK_MENU, VK_SHIFT},
    WindowsAndMessaging::{GetCursorPos, SetCursorPos},
  },
};

use super::CursorScrubEvent;
use crate::raw_mouse::{MouseReport, Subscription};

struct ActiveScrub {
  anchor: POINT,
  events: mpsc::Sender<CursorScrubEvent>,
  /// Set once the scrub has reported its end, so no move follows it.
  finished: bool,
  /// Mouse travel and the left button's release, from the process's one raw
  /// mouse registration: a registration of the scrub's own would take the
  /// input from Glide and, removed, leave it with none.
  _mouse: Subscription,
  /// Escape ends the scrub. It comes through the process's keyboard hook,
  /// not raw input: while the process registers raw keyboard input, Windows
  /// calls none of its low-level keyboard hooks for keys typed into its own
  /// windows.
  _escape: screenwide_keyboard_hook::Registration,
}

static ACTIVE_SCRUB: Mutex<Option<ActiveScrub>> = Mutex::new(None);

pub(super) fn begin(channel: Channel<CursorScrubEvent>) -> Result<(), String> {
  if ACTIVE_SCRUB
    .lock()
    .map_err(|_| "The cursor scrub state is unavailable".to_owned())?
    .is_some()
  {
    return Err("A cursor scrub is already active".to_owned());
  }

  let mut anchor = POINT::default();
  unsafe { GetCursorPos(&mut anchor) }
    .map_err(|error| format!("Could not read the cursor position: {error}"))?;
  let mouse = crate::raw_mouse::subscribe(handle_mouse)?;
  let escape = screenwide_keyboard_hook::observe(|event| {
    if event.key == u32::from(VK_ESCAPE.0) && event.phase == Phase::Press {
      finish_from_input();
    }
  })?;
  let (event_tx, event_rx) = mpsc::channel();
  std::thread::Builder::new()
    .name("cursor-scrub-events".to_owned())
    .spawn(move || {
      for event in event_rx {
        if channel.send(event).is_err() {
          break;
        }
      }
    })
    .map_err(|error| format!("Could not start the cursor scrub event stream: {error}"))?;

  *ACTIVE_SCRUB
    .lock()
    .map_err(|_| "The cursor scrub state is unavailable".to_owned())? = Some(ActiveScrub {
    anchor,
    events: event_tx,
    finished: false,
    _mouse: mouse,
    _escape: escape,
  });
  Ok(())
}

pub(super) fn end(offset_x: f64) -> Result<(), String> {
  let scrub = ACTIVE_SCRUB
    .lock()
    .map_err(|_| "The cursor scrub state is unavailable".to_owned())?
    .take();
  if let Some(scrub) = scrub {
    unsafe { SetCursorPos(scrub.anchor.x + offset_x.round() as i32, scrub.anchor.y) }
      .map_err(|error| format!("Could not restore the cursor position: {error}"))?;
  }
  Ok(())
}

fn handle_mouse(report: &MouseReport) {
  if report.left_released {
    finish_from_input();
  } else if report.delta_x != 0 || report.delta_y != 0 {
    emit_movement(report.delta_x, report.delta_y);
  }
}

fn emit_movement(delta_x: i32, delta_y: i32) {
  let Ok(active) = ACTIVE_SCRUB.lock() else {
    return;
  };
  let Some(scrub) = active.as_ref().filter(|scrub| !scrub.finished) else {
    return;
  };
  let _ = scrub.events.send(CursorScrubEvent::Move {
    alt_key: unsafe { GetAsyncKeyState(VK_MENU.0 as i32) } < 0,
    delta_x,
    delta_y,
    shift_key: unsafe { GetAsyncKeyState(VK_SHIFT.0 as i32) } < 0,
  });
  let anchor = scrub.anchor;
  drop(active);
  let _ = unsafe { SetCursorPos(anchor.x, anchor.y) };
}

fn finish_from_input() {
  let Ok(mut active) = ACTIVE_SCRUB.lock() else {
    return;
  };
  let Some(scrub) = active.as_mut().filter(|scrub| !scrub.finished) else {
    return;
  };
  scrub.finished = true;
  let _ = scrub.events.send(CursorScrubEvent::End);
  let anchor = scrub.anchor;
  drop(active);
  let _ = unsafe { SetCursorPos(anchor.x, anchor.y) };
}
