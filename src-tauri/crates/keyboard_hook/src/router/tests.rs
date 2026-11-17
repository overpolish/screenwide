// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::{Arc, Mutex};

use super::*;

const S: u32 = 0x53;
const ESCAPE: u32 = 0x1B;
const LEFT_SHIFT: u32 = 0xA0;
const LEFT_WIN: u32 = 0x5B;

type Heard = Arc<Mutex<Vec<(&'static str, u32, Phase, bool)>>>;

fn key(key: u32, pressed: bool) -> RawKey {
  RawKey {
    key,
    pressed,
    time: 0,
    extended: false,
    injected: false,
  }
}

/// A taker that logs what it hears under `name` and takes the keys `takes`
/// answers yes for.
fn taker(
  heard: &Heard,
  name: &'static str,
  takes: impl Fn(&KeyEvent) -> bool + Send + 'static,
) -> Taker {
  let heard = Arc::clone(heard);
  Box::new(move |event| {
    heard
      .lock()
      .unwrap()
      .push((name, event.key, event.phase, event.owned));
    if takes(event) {
      Route::Take
    } else {
      Route::Pass
    }
  })
}

fn names(heard: &Heard) -> Vec<&'static str> {
  heard
    .lock()
    .unwrap()
    .iter()
    .map(|(name, ..)| *name)
    .collect()
}

#[test]
fn a_key_goes_to_the_first_handler_in_priority_order_that_takes_it() {
  let heard = Heard::default();
  let mut router = Router::default();
  // Registered out of order: the priority decides, not the registration.
  router.add_taker(Priority::Glide, taker(&heard, "glide", |_| true));
  router.add_taker(Priority::SystemKeys, taker(&heard, "system", |_| true));
  router.add_taker(Priority::Overlay, taker(&heard, "overlay", |_| false));

  assert!(router.route(key(S, true)).take);
  assert_eq!(names(&heard), ["overlay", "system"]);
}

#[test]
fn a_taken_key_keeps_its_repeats_and_release_with_its_handler() {
  let heard = Heard::default();
  let mut router = Router::default();
  router.add_taker(
    Priority::SystemKeys,
    taker(&heard, "system", |event| event.phase == Phase::Press),
  );
  router.add_taker(Priority::Glide, taker(&heard, "glide", |_| true));

  assert!(router.route(key(S, true)).take);
  assert!(router.route(key(S, true)).take);
  assert!(router.route(key(S, false)).take);
  assert_eq!(
    *heard.lock().unwrap(),
    [
      ("system", S, Phase::Press, false),
      ("system", S, Phase::Repeat, true),
      ("system", S, Phase::Release, true),
    ]
  );

  // Released, the key is free again.
  heard.lock().unwrap().clear();
  router.route(key(S, true));
  assert_eq!(names(&heard), ["system"]);
}

#[test]
fn every_handler_hears_a_release_nobody_owns_and_it_passes() {
  let heard = Heard::default();
  let mut router = Router::default();
  router.add_taker(Priority::SystemKeys, taker(&heard, "system", |_| false));
  router.add_taker(Priority::Glide, taker(&heard, "glide", |_| false));

  router.route(key(ESCAPE, true));
  heard.lock().unwrap().clear();
  assert!(!router.route(key(ESCAPE, false)).take);
  assert_eq!(names(&heard), ["system", "glide"]);
}

#[test]
fn a_held_key_can_be_taken_on_a_repeat() {
  let heard = Heard::default();
  let mut router = Router::default();
  router.add_taker(
    Priority::Glide,
    taker(&heard, "glide", |event| event.phase == Phase::Repeat),
  );

  assert!(!router.route(key(S, true)).take);
  assert!(router.route(key(S, true)).take);
  assert!(router.route(key(S, false)).take);
}

#[test]
fn observers_hear_every_key_including_taken_ones() {
  let heard = Heard::default();
  let mut router = Router::default();
  let observed = Arc::new(Mutex::new(Vec::new()));
  let log = Arc::clone(&observed);
  router.add_observer(Box::new(move |event: &KeyEvent| {
    log.lock().unwrap().push((event.key, event.phase));
  }));
  router.add_taker(Priority::SystemKeys, taker(&heard, "system", |_| true));

  router.route(key(S, true));
  router.route(key(S, false));
  assert_eq!(
    *observed.lock().unwrap(),
    [(S, Phase::Press), (S, Phase::Release)]
  );
}

#[test]
fn nothing_is_taken_while_taking_is_paused() {
  let heard = Heard::default();
  let mut router = Router::default();
  let told = Arc::new(Mutex::new(Vec::new()));
  let log = Arc::clone(&told);
  router.add_taker(
    Priority::SystemKeys,
    Box::new(move |event: &KeyEvent| {
      log.lock().unwrap().push(event.may_take);
      Route::Take
    }),
  );
  router.add_taker(Priority::Glide, taker(&heard, "glide", |_| true));
  router.set_paused(true);

  assert!(!router.route(key(S, true)).take);
  // Every handler was still told about the key, and told it may not take it.
  assert_eq!(*told.lock().unwrap(), [false]);
  assert_eq!(names(&heard), ["glide"]);
}

#[test]
fn modifiers_follow_the_key_stream_and_a_lost_release_is_forgotten() {
  let mut router = Router::default();
  let seen = Arc::new(Mutex::new(Vec::new()));
  let log = Arc::clone(&seen);
  router.add_observer(Box::new(move |event: &KeyEvent| {
    log.lock().unwrap().push(event.modifiers);
  }));

  router.route(key(LEFT_WIN, true));
  router.route(key(LEFT_SHIFT, true));
  router.route(key(S, true));
  let win_shift = Modifiers {
    shift: true,
    win: true,
    ..Modifiers::default()
  };
  assert_eq!(seen.lock().unwrap()[2], win_shift);

  // Windows was released where the hook could not see it, on the lock
  // screen: the system says it is up, so the next key is not Windows+S.
  router.forget_released(S, |key| key != LEFT_WIN);
  router.route(key(S, false));
  router.route(key(S, true));
  assert_eq!(
    seen.lock().unwrap()[4],
    Modifiers {
      shift: true,
      ..Modifiers::default()
    }
  );
}

#[test]
fn a_taken_key_is_not_forgotten_though_the_system_never_saw_it() {
  let heard = Heard::default();
  let mut router = Router::default();
  router.add_taker(Priority::Overlay, taker(&heard, "overlay", |_| true));

  router.route(key(S, true));
  router.forget_released(LEFT_SHIFT, |_| false);
  heard.lock().unwrap().clear();
  assert!(router.route(key(S, true)).take);
  assert_eq!(
    *heard.lock().unwrap(),
    [("overlay", S, Phase::Repeat, true)]
  );
}

#[test]
fn taking_a_key_with_windows_held_asks_for_the_mask_key() {
  let heard = Heard::default();
  let mut router = Router::default();
  router.add_taker(
    Priority::SystemKeys,
    taker(&heard, "system", |event| event.key == S),
  );

  router.route(key(LEFT_WIN, true));
  let outcome = router.route(key(S, true));
  assert_eq!(
    outcome,
    Outcome {
      take: true,
      mask: true
    }
  );
  router.route(key(S, false));
  router.route(key(LEFT_WIN, false));
  assert!(!router.route(key(S, true)).mask);
}

#[test]
fn a_removed_handler_gives_its_held_key_back() {
  let heard = Heard::default();
  let mut router = Router::default();
  let id = router.add_taker(Priority::SystemKeys, taker(&heard, "system", |_| true));

  router.route(key(S, true));
  router.remove(id);
  assert!(!router.route(key(S, false)).take);
}
