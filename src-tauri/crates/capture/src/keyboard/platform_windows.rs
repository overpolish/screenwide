// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{mpsc, Arc};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

mod classifier_windows;
mod focus_probe;
use self::classifier_windows as classifier;

use screenwide_keyboard_hook::{Phase, Registration};

use super::{EventSink, FocusContext, RawKeyboardEvent};

const EVENT_QUEUE_CAPACITY: usize = 128;
const POLL: Duration = Duration::from_millis(25);
const START_TIMEOUT: Duration = Duration::from_secs(2);

/// Only one recorder may listen: a second one would double every event.
static LISTENING: AtomicBool = AtomicBool::new(false);

/// Hears every key on the process's keyboard hook. Injected keys count:
/// remote-desktop and streaming hosts (Parsec, Sunshine, RDP) deliver all of
/// the user's real typing that way, and the macOS HID tap records synthetic
/// events too. The hook holds up every keystroke in the system until this
/// returns, so it only queues the key.
fn listen(
  events: mpsc::SyncSender<classifier::PendingEvent>,
  focus: Arc<AtomicU8>,
) -> Result<Registration, String> {
  screenwide_keyboard_hook::observe(move |event| {
    let _ = events.try_send(classifier::PendingEvent {
      at: Instant::now(),
      extended: event.extended,
      focus: focus.load(Ordering::Acquire),
      is_down: event.phase != Phase::Release,
      virtual_key: event.key,
    });
  })
  .map_err(|error| format!("Could not listen for keyboard events: {error}"))
}

fn run(stop: &AtomicBool, sink: &EventSink, ready: mpsc::Sender<Result<(), String>>) {
  if LISTENING.swap(true, Ordering::AcqRel) {
    let _ = ready.send(Err(
      "Keyboard shortcut recording is already running".to_owned(),
    ));
    return;
  }

  let (events, pending) = mpsc::sync_channel::<classifier::PendingEvent>(EVENT_QUEUE_CAPACITY);
  let cached_focus = Arc::new(AtomicU8::new(classifier::encode_focus(
    FocusContext::Unknown,
  )));
  let registration = match listen(events, Arc::clone(&cached_focus)) {
    Ok(registration) => registration,
    Err(error) => {
      LISTENING.store(false, Ordering::Release);
      let _ = ready.send(Err(error));
      return;
    }
  };
  let _ = ready.send(Ok(()));

  let focus = focus_probe::FocusProbe::start(Arc::clone(&cached_focus));
  let mut tracker = classifier::KeyTracker::default();
  let mut deliver = |events: Vec<classifier::PendingEvent>, fresh: FocusContext| {
    for event in events {
      let focus = FocusContext::conservative(classifier::decode_focus(event.focus), fresh);
      let Some(classified) = tracker.classify(&event) else {
        continue;
      };
      sink(RawKeyboardEvent {
        at: event.at,
        focus,
        kind: classified.kind,
        key_code: classified.key_code,
        modifiers: classified.modifiers,
      });
    }
  };
  while !stop.load(Ordering::Acquire) {
    deliver(pending.try_iter().collect(), focus.current());
    std::thread::sleep(POLL);
  }
  drop(registration);
  // What the hook passed on before the stop still belongs to the recording.
  deliver(pending.try_iter().collect(), focus.current());
  LISTENING.store(false, Ordering::Release);
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
