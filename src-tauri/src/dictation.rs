// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Dictation, wired into the app. The Dictate shortcut starts it; held, it
//! listens until let go, and tapped, until tapped again, or until the
//! island's Stop. Escape or the island's Cancel drops it. It listens in the
//! language transcription is set to (`setup`). The island shows the words
//! as they are heard (`island`), its spoken commands carried out, words
//! added to or removed from the word list by voice confirmed (`taught`),
//! and when it stops the text is pasted where the caret is
//! (`screenwide_paste`).
//!
//! Dictation's helper is kept for a few minutes after each dictation, with
//! its model loaded, so the next one starts at once.

pub(crate) mod island;
mod put_in;
mod setup;
mod taught;

use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use screenwide_dictation::{shape, DictatedText, Ended, Event, Session};
use screenwide_paste::Target;
use screenwide_speech::runner::Transcriber;
use tauri::AppHandle;

use self::island::{DictationIslandState, DictationProblem};
use self::setup::Spoken;

/// A press held at least this long is a hold, and letting go stops
/// listening; a shorter one is a tap, and listening goes on.
const HOLD: Duration = Duration::from_millis(350);
/// How long dictation's helper is kept after a dictation.
const KEEP_HELPER: Duration = Duration::from_secs(180);

#[derive(Default)]
struct Dictation {
  listening: Option<Listening>,
  /// Between stopping and the paste finishing.
  finishing: bool,
  helper: Option<Transcriber>,
  /// Which dictation last used the helper, for the timer that ends it.
  used: u64,
  /// The microphone chosen on the recording island, by id; the system's
  /// default while none is.
  microphone: Option<String>,
  /// The Dictate shortcut is down. Windows sends its press again while the
  /// keys are held, which would otherwise read as a second tap.
  held: bool,
}

/// The recording island's microphone changed: dictation listens to the same
/// one. The choice is kept by the island's page, which says it as it loads
/// and whenever it changes.
#[tauri::command]
pub fn set_dictation_microphone(microphone: Option<String>) {
  with(|dictation| dictation.microphone = microphone);
}

struct Listening {
  session: Session,
  pressed_at: Instant,
  /// The app dictated into: the one in front as listening started.
  target: Option<Target>,
  spoken: Spoken,
}

static DICTATION: Mutex<Option<Dictation>> = Mutex::new(None);

fn state() -> MutexGuard<'static, Option<Dictation>> {
  DICTATION.lock().unwrap_or_else(PoisonError::into_inner)
}

fn with<T>(work: impl FnOnce(&mut Dictation) -> T) -> T {
  work(state().get_or_insert_with(Dictation::default))
}

/// Whether a dictation is listening, which takes Escape.
pub(crate) fn is_listening() -> bool {
  state()
    .as_ref()
    .is_some_and(|dictation| dictation.listening.is_some())
}

/// The Dictate shortcut's keys went down. False for a press Windows repeats
/// while they are held, which is not a second tap.
pub(crate) fn key_down() -> bool {
  !with(|dictation| std::mem::replace(&mut dictation.held, true))
}

/// The Dictate shortcut was pressed: listening starts, or a tapped
/// dictation stops.
pub(crate) fn pressed(app: &AppHandle) {
  let (listening, finishing) =
    with(|dictation| (dictation.listening.is_some(), dictation.finishing));
  if finishing {
    return;
  }
  if listening {
    finish(app);
    return;
  }
  let started = problem(app).map_or_else(
    || setup::setup(app, with(|dictation| dictation.microphone.clone())),
    Err,
  );
  let (setup, spoken) = match started {
    Ok(started) => started,
    Err(problem) => {
      island::show_problem(app, problem);
      return;
    }
  };
  let events = app.clone();
  taught::reset();
  with(|dictation| {
    let helper = dictation.helper.take();
    dictation.used += 1;
    dictation.listening = Some(Listening {
      session: Session::start(setup, helper, move |event| match event {
        Event::Level(decibels) => island::level(&events, decibels),
        // Words still arriving as listening stops would show over the
        // island's Finishing.
        Event::Heard(heard) if is_listening() => heard_so_far(&events, &heard, spoken),
        Event::Heard(_) => {}
        // Said as a problem once the session's end says why.
        Event::Failed => {
          let app = events.clone();
          let _ = events.run_on_main_thread(move || finish(&app));
        }
      }),
      pressed_at: Instant::now(),
      target: Target::current(),
      spoken,
    });
  });
  island::tell(
    app,
    DictationIslandState::Listening {
      settled: String::new(),
      unsettled: String::new(),
      taught: Vec::new(),
      at_pauses: spoken.at_pauses,
    },
  );
  sync_escape(app);
}

/// Shows what has been heard so far, its commands carried out, and carries
/// out word list changes as they settle.
fn heard_so_far(app: &AppHandle, heard: &DictatedText, spoken: Spoken) {
  let shaped = shape(heard, spoken.phrases);
  island::tell(
    app,
    DictationIslandState::Listening {
      taught: taught::apply(app, &shaped.changes),
      settled: shaped.settled,
      unsettled: shaped.unsettled,
      at_pauses: spoken.at_pauses,
    },
  );
}

/// The Dictate shortcut was let go: a hold stops listening.
pub(crate) fn released(app: &AppHandle) {
  let held = with(|dictation| {
    dictation.held = false;
    dictation
      .listening
      .as_ref()
      .is_some_and(|listening| listening.pressed_at.elapsed() >= HOLD)
  });
  if held {
    finish(app);
  }
}

/// Stops listening and pastes what was heard.
pub(crate) fn finish(app: &AppHandle) {
  end(app, false);
}

/// Stops listening and drops what was heard.
pub(crate) fn cancel(app: &AppHandle) {
  end(app, true);
}

fn end(app: &AppHandle, cancelled: bool) {
  let Some(listening) = with(|dictation| {
    let listening = dictation.listening.take();
    dictation.finishing = listening.is_some();
    listening
  }) else {
    return;
  };
  sync_escape(app);
  if cancelled {
    island::withdraw(app);
  } else {
    island::tell(app, DictationIslandState::Finishing);
  }
  let app = app.clone();
  std::thread::spawn(move || {
    let Ended { heard, helper } = if cancelled {
      listening.session.cancel()
    } else {
      listening.session.finish()
    };
    keep_helper(helper);
    if cancelled {
      with(|dictation| dictation.finishing = false);
      return;
    }
    let outcome = put_in::put_in(&app, heard, (listening.target, listening.spoken.phrases));
    with(|dictation| dictation.finishing = false);
    match outcome {
      Ok(taught) if taught.is_empty() => island::withdraw(&app),
      Ok(taught) => island::show_taught(&app, taught),
      Err(problem) => island::show_problem(&app, problem),
    }
  });
}

/// What stands in the way of dictating, before the model is looked for.
fn problem(app: &AppHandle) -> Option<DictationProblem> {
  let permissions = crate::permissions::cached_snapshot(app);
  if !permissions.microphone.granted {
    return Some(DictationProblem::NoMicrophone);
  }
  if !permissions.accessibility.granted {
    return Some(DictationProblem::NoAccessibility);
  }
  None
}

/// Keeps the helper for the next dictation, and ends it once none has come
/// for `KEEP_HELPER`, so its model's memory is given back.
fn keep_helper(helper: Option<Transcriber>) {
  let used = with(|dictation| {
    dictation.helper = helper;
    dictation.used
  });
  std::thread::spawn(move || {
    std::thread::sleep(KEEP_HELPER);
    let idle = with(|dictation| {
      (dictation.used == used && dictation.listening.is_none())
        .then(|| dictation.helper.take())
        .flatten()
    });
    drop(idle);
  });
}

fn sync_escape(app: &AppHandle) {
  crate::app_windows::sync_recording_ui_escape(app, crate::ruler::is_active(app));
}
