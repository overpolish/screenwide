// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

mod classifier;

use core_foundation::runloop::{kCFRunLoopDefaultMode, CFRunLoop};
use core_graphics::event::{
  CGEvent, CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
  CGEventType, CallbackResult, EventField,
};

use super::{EventSink, FocusContext, RawKeyboardEvent};

const EVENT_QUEUE_CAPACITY: usize = 128;
const RUN_LOOP_POLL: Duration = Duration::from_millis(25);
const START_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Clone, Copy)]
struct PendingKeyDown {
  at: Instant,
  flags: CGEventFlags,
  focus: FocusContext,
  is_repeat: bool,
  key_code: u16,
  kind: classifier::PendingKind,
}

fn run(stop: &AtomicBool, sink: &EventSink, ready: mpsc::Sender<Result<(), String>>) {
  let ready = RefCell::new(Some(ready));
  let (events, pending) = mpsc::sync_channel::<PendingKeyDown>(EVENT_QUEUE_CAPACITY);
  let cached_focus = Rc::new(Cell::new(FocusContext::Unknown));
  let event_focus = Rc::clone(&cached_focus);
  let result = CGEventTap::with_enabled(
    CGEventTapLocation::HID,
    CGEventTapPlacement::HeadInsertEventTap,
    CGEventTapOptions::ListenOnly,
    vec![
      CGEventType::KeyDown,
      CGEventType::KeyUp,
      CGEventType::FlagsChanged,
    ],
    move |_, event_type, event: &CGEvent| {
      let key_code = event
        .get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE)
        .clamp(0, i64::from(u16::MAX)) as u16;
      let _ = events.try_send(PendingKeyDown {
        at: Instant::now(),
        flags: event.get_flags(),
        focus: event_focus.get(),
        is_repeat: event.get_integer_value_field(EventField::KEYBOARD_EVENT_AUTOREPEAT) != 0,
        key_code,
        kind: match event_type {
          CGEventType::KeyUp => classifier::PendingKind::KeyUp,
          CGEventType::FlagsChanged => classifier::PendingKind::FlagsChanged,
          _ => classifier::PendingKind::KeyDown,
        },
      });
      CallbackResult::Keep
    },
    || {
      if let Some(ready) = ready.borrow_mut().take() {
        let _ = ready.send(Ok(()));
      }
      while !stop.load(Ordering::Acquire) {
        cached_focus.set(classifier::focus_context());
        // SAFETY: Core Foundation owns this process-global constant.
        unsafe {
          CFRunLoop::run_in_mode(kCFRunLoopDefaultMode, RUN_LOOP_POLL, false);
        }
        while let Ok(event) = pending.try_recv() {
          let focus = FocusContext::conservative(event.focus, classifier::focus_context());
          let Some(event_kind) =
            classifier::event_kind(event.kind, event.key_code, event.flags, event.is_repeat)
          else {
            continue;
          };
          sink(RawKeyboardEvent {
            at: event.at,
            focus,
            kind: event_kind,
            key_code: event.key_code,
            modifiers: classifier::modifiers(event.flags),
          });
        }
      }
    },
  );
  if result.is_err() {
    if let Some(ready) = ready.borrow_mut().take() {
      let _ = ready.send(Err(
        "Could not listen for keyboard events; Accessibility access is required".to_owned(),
      ));
    }
  }
}

pub(super) fn start(stop: Arc<AtomicBool>, sink: EventSink) -> Result<JoinHandle<()>, String> {
  let (ready, did_start) = mpsc::channel();
  let worker_stop = Arc::clone(&stop);
  let worker = std::thread::Builder::new()
    .name("screenwide-keyboard-recorder".to_owned())
    .spawn(move || run(&worker_stop, &sink, ready))
    .map_err(|error| error.to_string())?;
  match did_start.recv_timeout(START_TIMEOUT) {
    Ok(Ok(())) => Ok(worker),
    Ok(Err(error)) => {
      let _ = worker.join();
      Err(error)
    }
    Err(_) => {
      stop.store(true, Ordering::Release);
      let _ = worker.join();
      Err("Keyboard shortcut recording did not start in time".to_owned())
    }
  }
}
