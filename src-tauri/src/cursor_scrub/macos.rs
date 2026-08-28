// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use core_foundation::runloop::{kCFRunLoopDefaultMode, CFRunLoop};
use core_graphics::{
  display::CGDisplay,
  event::{
    CGEvent, CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
    CGEventType, CallbackResult, EventField, KeyCode,
  },
  event_source::{CGEventSource, CGEventSourceStateID},
  geometry::CGPoint,
};
use tauri::ipc::Channel;

use super::CursorScrubEvent;

const RUN_LOOP_POLL: Duration = Duration::from_millis(25);
const START_TIMEOUT: Duration = Duration::from_secs(2);

struct ActiveScrub {
  anchor: (f64, f64),
  stop: Arc<AtomicBool>,
  worker: JoinHandle<()>,
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

  let source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState)
    .map_err(|()| "Could not read the cursor position".to_owned())?;
  let point = CGEvent::new(source)
    .map_err(|()| "Could not read the cursor position".to_owned())?
    .location();

  CGDisplay::associate_mouse_and_mouse_cursor_position(false)
    .map_err(|error| format!("Could not pin the cursor: {error}"))?;
  if let Err(error) = CGDisplay::warp_mouse_cursor_position(point) {
    let _ = CGDisplay::associate_mouse_and_mouse_cursor_position(true);
    return Err(format!("Could not pin the cursor: {error}"));
  }

  let stop = Arc::new(AtomicBool::new(false));
  let worker_stop = Arc::clone(&stop);
  let (ready_tx, ready_rx) = mpsc::sync_channel(1);
  let worker = match std::thread::Builder::new()
    .name("cursor-scrub-input".to_owned())
    .spawn(move || run_input_monitor(&worker_stop, channel, ready_tx, (point.x, point.y)))
  {
    Ok(worker) => worker,
    Err(error) => {
      restore_cursor((point.x, point.y), 0.0);
      return Err(format!("Could not start the cursor scrub monitor: {error}"));
    }
  };

  match ready_rx.recv_timeout(START_TIMEOUT) {
    Ok(Ok(())) => {}
    Ok(Err(error)) => {
      stop.store(true, Ordering::Release);
      let _ = worker.join();
      restore_cursor((point.x, point.y), 0.0);
      return Err(error);
    }
    Err(_) => {
      stop.store(true, Ordering::Release);
      let _ = worker.join();
      restore_cursor((point.x, point.y), 0.0);
      return Err("The cursor scrub monitor did not start in time".to_owned());
    }
  }

  *ACTIVE_SCRUB
    .lock()
    .map_err(|_| "The cursor scrub state is unavailable".to_owned())? = Some(ActiveScrub {
    anchor: (point.x, point.y),
    stop,
    worker,
  });
  Ok(())
}

pub(super) fn end(offset_x: f64) -> Result<(), String> {
  let scrub = ACTIVE_SCRUB
    .lock()
    .map_err(|_| "The cursor scrub state is unavailable".to_owned())?
    .take();
  if let Some(scrub) = scrub {
    scrub.stop.store(true, Ordering::Release);
    let _ = scrub.worker.join();
    restore_cursor(scrub.anchor, offset_x);
  }
  Ok(())
}

fn restore_cursor(anchor: (f64, f64), offset_x: f64) {
  let _ = CGDisplay::warp_mouse_cursor_position(CGPoint::new(anchor.0 + offset_x, anchor.1));
  let _ = CGDisplay::associate_mouse_and_mouse_cursor_position(true);
}

fn run_input_monitor(
  stop: &AtomicBool,
  channel: Channel<CursorScrubEvent>,
  ready: mpsc::SyncSender<Result<(), String>>,
  anchor: (f64, f64),
) {
  let ready = std::cell::RefCell::new(Some(ready));
  let result = CGEventTap::with_enabled(
    CGEventTapLocation::HID,
    CGEventTapPlacement::HeadInsertEventTap,
    CGEventTapOptions::Default,
    vec![
      CGEventType::MouseMoved,
      CGEventType::LeftMouseDragged,
      CGEventType::LeftMouseUp,
      CGEventType::KeyDown,
    ],
    |_, event_type, event| {
      if matches!(
        event_type,
        CGEventType::MouseMoved | CGEventType::LeftMouseDragged | CGEventType::LeftMouseUp
      ) {
        event.set_location(CGPoint::new(anchor.0, anchor.1));
      }
      match event_type {
        CGEventType::MouseMoved | CGEventType::LeftMouseDragged => {
          let delta_x = event
            .get_integer_value_field(EventField::MOUSE_EVENT_DELTA_X)
            .clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32;
          let delta_y = event
            .get_integer_value_field(EventField::MOUSE_EVENT_DELTA_Y)
            .clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32;
          if delta_x != 0 || delta_y != 0 {
            let flags = event.get_flags();
            let _ = channel.send(CursorScrubEvent::Move {
              alt_key: flags.contains(CGEventFlags::CGEventFlagAlternate),
              delta_x,
              delta_y,
              shift_key: flags.contains(CGEventFlags::CGEventFlagShift),
            });
          }
        }
        CGEventType::LeftMouseUp => {
          let _ = channel.send(CursorScrubEvent::End);
        }
        CGEventType::KeyDown
          if event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE)
            == i64::from(KeyCode::ESCAPE) =>
        {
          let _ = channel.send(CursorScrubEvent::End);
        }
        _ => {}
      }
      CallbackResult::Keep
    },
    || {
      if let Some(ready) = ready.borrow_mut().take() {
        let _ = ready.send(Ok(()));
      }
      while !stop.load(Ordering::Acquire) {
        // SAFETY: Core Foundation owns this process-global constant.
        unsafe {
          CFRunLoop::run_in_mode(kCFRunLoopDefaultMode, RUN_LOOP_POLL, false);
        }
      }
    },
  );
  if result.is_err() {
    if let Some(ready) = ready.borrow_mut().take() {
      let _ = ready.send(Err(
        "Could not listen for native cursor scrub events; Accessibility access is required"
          .to_owned(),
      ));
    }
  }
}
