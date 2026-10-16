// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tells deliberate Glide input from everyday typing and pointing. A Glide
//! control can be a letter, which also types in the focused application, and
//! a modifier, which also starts shortcuts; the pointer parked on a title bar
//! says nothing about which of the two is meant.

use std::time::{Duration, Instant};

/// How long after an ordinary keystroke a letter control still types.
const TYPING_PAUSE: Duration = Duration::from_secs(1);

/// How far the pointer must travel to count as reaching for a window: in
/// points on macOS and physical pixels on Windows. Jitter while a hand rests
/// on the mouse stays well inside it.
const DELIBERATE_DISTANCE: f64 = 8.0;

fn distance(from: (f64, f64), to: (f64, f64)) -> f64 {
  (to.0 - from.0).hypot(to.1 - from.1)
}

/// Whether a letter control is pressed to glide or to type. Someone typing
/// leaves the pointer where their last keystroke or click found it; someone
/// gliding has just moved it onto a title bar.
#[derive(Default)]
pub(crate) struct TypingIntent {
  typed_at: Option<Instant>,
  /// Where the pointer was at the last keystroke or click.
  rest: Option<(f64, f64)>,
}

impl TypingIntent {
  pub fn note_key(&mut self, now: Instant, pointer: (f64, f64)) {
    self.typed_at = Some(now);
    self.rest = Some(pointer);
  }

  pub fn note_click(&mut self, pointer: (f64, f64)) {
    self.rest = Some(pointer);
  }

  /// Whether a letter control pressed now, with the pointer here, belongs to
  /// Glide: only after a pause in typing, and once the pointer has moved away
  /// from where the last keystroke or click left it.
  pub fn claims_control(&self, now: Instant, pointer: (f64, f64)) -> bool {
    self
      .typed_at
      .is_none_or(|typed| now.saturating_duration_since(typed) >= TYPING_PAUSE)
      && self
        .rest
        .is_none_or(|rest| distance(rest, pointer) >= DELIBERATE_DISTANCE)
  }
}

/// Where a held control's mouse glide starts. A held modifier is as likely to
/// be the start of a shortcut, so the glide waits until the pointer has
/// travelled from where it was first seen with the control down, and then
/// starts from there.
#[derive(Default)]
pub(crate) struct StartTravel {
  origin: Option<(f64, f64)>,
}

impl StartTravel {
  /// The origin, once `pointer` lies far enough from it.
  pub fn reached(&mut self, pointer: (f64, f64)) -> Option<(f64, f64)> {
    let origin = *self.origin.get_or_insert(pointer);
    (distance(origin, pointer) >= DELIBERATE_DISTANCE).then_some(origin)
  }

  /// Measures again from `pointer`, after an origin that opened no glide.
  pub fn restart(&mut self, pointer: (f64, f64)) {
    self.origin = Some(pointer);
  }

  pub fn reset(&mut self) {
    self.origin = None;
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn a_letter_types_until_typing_pauses_and_the_pointer_moves() {
    let start = Instant::now();
    let mut intent = TypingIntent::default();
    assert!(intent.claims_control(start, (0.0, 0.0)));
    intent.note_key(start, (0.0, 0.0));
    // Mid-sentence, even with the pointer moved.
    assert!(!intent.claims_control(start + Duration::from_millis(400), (40.0, 0.0)));
    // After a pause, with the pointer still where the typing left it.
    assert!(!intent.claims_control(start + Duration::from_secs(5), (3.0, 3.0)));
    assert!(intent.claims_control(start + Duration::from_secs(5), (40.0, 0.0)));
  }

  #[test]
  fn a_click_holds_the_letter_until_the_pointer_moves() {
    let now = Instant::now();
    let mut intent = TypingIntent::default();
    intent.note_click((100.0, 20.0));
    assert!(!intent.claims_control(now, (100.0, 20.0)));
    assert!(intent.claims_control(now, (100.0, 30.0)));
  }

  #[test]
  fn a_mouse_glide_starts_where_the_pointer_set_off_once_it_travels() {
    let mut travel = StartTravel::default();
    assert_eq!(travel.reached((10.0, 10.0)), None);
    // Jitter back and forth never adds up.
    for _ in 0..20 {
      assert_eq!(travel.reached((14.0, 10.0)), None);
      assert_eq!(travel.reached((10.0, 13.0)), None);
    }
    assert_eq!(travel.reached((19.0, 10.0)), Some((10.0, 10.0)));
    travel.restart((19.0, 10.0));
    assert_eq!(travel.reached((22.0, 10.0)), None);
  }
}
