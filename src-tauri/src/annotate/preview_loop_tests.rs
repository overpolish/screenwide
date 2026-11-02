// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The settings scene (`annotate-scene.ts`) played across the end of one
//! pass and the start of the next, a frame a sixtieth of a second, as the
//! page sends it (`use-tool-pointer.ts`) and the settings window's thread
//! fades it (`AnnotatePreview::frame`).

use super::tests::{counter_centre, counter_tip, drawn, glass, keep_frame, point, HEAD, TAIL};
use super::tests::{DOWN, DRAG, UP};
use super::*;

/// The scene's timeline, in seconds into a pass.
const DURATION: f64 = 6.4;
const PICK_ARROW: f64 = 0.1;
const PICK_COUNTER: f64 = 1.4;
const EXIT: f64 = 3.4;
const ENTER: f64 = 4.4;
/// The next pass's first stroke begins to show at its first drag.
const FIRST_DRAG: f64 = 0.35;
const FRAME_STEP: f64 = 1.0 / 60.0;

const COUNTER_SIZE: f64 = 80.0;
const COUNTER_ANGLE: f64 = 0.0;

fn travel(at: f64, (start, end): (f64, f64), from: (f64, f64), to: (f64, f64)) -> (f64, f64) {
  let amount = ((at - start) / (end - start)).clamp(0.0, 1.0);
  let eased = amount * amount * (3.0 - 2.0 * amount);
  (
    from.0 + (to.0 - from.0) * eased,
    from.1 + (to.1 - from.1) * eased,
  )
}

/// Where the scene's pointer is `at` seconds in, and whether it is pressed.
fn pointer_at(at: f64) -> Option<(bool, (f64, f64))> {
  let counter = counter_centre(counter_tip(COUNTER_ANGLE), COUNTER_SIZE, COUNTER_ANGLE);
  match at {
    at if at < 0.3 => Some((false, TAIL)),
    at if at < 1.2 => Some((true, travel(at, (0.35, 1.1), TAIL, HEAD))),
    at if at < 1.6 => Some((false, travel(at, (1.2, 1.55), HEAD, counter))),
    at if at < 1.75 => Some((true, counter)),
    _ => None,
  }
}

/// The page's player and the window's fade over the glass.
struct Player {
  glass: Glass,
  pass: i64,
  previous: f64,
  last: Option<(bool, (f64, f64))>,
  leaving_since: Option<f64>,
}

impl Player {
  fn new(keep: bool) -> Self {
    Self {
      glass: glass(COUNTER_SIZE, COUNTER_ANGLE, keep),
      pass: -1,
      previous: -1.0,
      last: None,
      leaving_since: None,
    }
  }

  /// Plays the frame at `seconds` since the scene started.
  fn tick(&mut self, seconds: f64) {
    let at = seconds % DURATION;
    let pass = (seconds / DURATION).floor() as i64;
    if pass != self.pass {
      (self.pass, self.previous, self.last) = (pass, -1.0, None);
      self.glass.reset();
      self.leaving_since = Some(seconds);
    }
    let cues = [PICK_ARROW, PICK_COUNTER, EXIT, ENTER];
    for cue in cues
      .into_iter()
      .filter(|&cue| self.previous < cue && cue <= at)
    {
      match cue {
        PICK_ARROW => self.glass.shape = AnnotationKind::Arrow,
        PICK_COUNTER => self.glass.shape = AnnotationKind::Counter,
        EXIT => self.glass.exit(),
        _ => self.glass.session = true,
      }
    }
    self.previous = at;
    let now = pointer_at(at);
    let was_pressed = self.last.is_some_and(|(pressed, _)| pressed);
    match now {
      Some((true, spot)) if !was_pressed => self.glass.pointer(DOWN, point(spot)),
      Some((false, spot)) if was_pressed => self.glass.pointer(UP, point(spot)),
      Some((true, spot)) => self.glass.pointer(DRAG, point(spot)),
      Some((false, _)) => {}
      None => {
        if let Some((true, spot)) = self.last {
          self.glass.pointer(UP, point(spot));
        }
      }
    }
    self.last = now;
    if let Some(since) = self.leaving_since {
      let amount = (seconds - since) / FADE.as_secs_f64();
      if self.glass.fade(amount as f32) {
        self.leaving_since = None;
      }
    }
  }
}

/// What is on the glass at each frame from just before the session ends to
/// the next pass's first stroke: the annotations drawn, and how much of the
/// glass they cover. Frames go to `SCREENWIDE_PREVIEW_FRAMES`.
fn play_across_the_loop(keep: bool) -> Vec<(f64, usize, f64)> {
  let mut player = Player::new(keep);
  let name = if keep { "keep" } else { "no-keep" };
  let shown = |seconds: f64| {
    let near = |cue: f64| (seconds - cue).abs() < 0.2;
    near(EXIT) || near(ENTER) || (DURATION - 0.2..DURATION + FIRST_DRAG + 0.2).contains(&seconds)
  };
  let mut seen = Vec::new();
  for frame in 0..((DURATION + 1.0) / FRAME_STEP) as u32 {
    let seconds = f64::from(frame) * FRAME_STEP;
    player.tick(seconds);
    if seconds < EXIT - 0.2 || !shown(seconds) {
      continue;
    }
    let pixels = drawn(&player.glass);
    keep_frame(&player.glass, &pixels, &format!("loop-{name}-{frame:04}"));
    // How much is drawn, in opaque pixels' worth, so a fade reads as less.
    let coverage = pixels
      .as_chunks::<4>()
      .0
      .iter()
      .map(|pixel| f64::from(pixel[3]) / 255.0)
      .sum();
    seen.push((seconds, player.glass.drawn().len(), coverage));
  }
  seen
}

/// Kept annotations stay through the session's end with no frame where they
/// vanish and return, then leave once, fading, before the next stroke.
#[test]
fn kept_annotations_stay_through_the_exit_and_fade_as_the_loop_turns() {
  let seen = play_across_the_loop(true);
  let before_turn = seen.iter().filter(|(seconds, ..)| *seconds < DURATION);
  for &(seconds, count, coverage) in before_turn {
    assert_eq!(
      count, 2,
      "the arrow and counter are on screen at {seconds:.3}s"
    );
    assert!(coverage > 0.0, "nothing is drawn at {seconds:.3}s");
  }
  let turn: Vec<_> = seen
    .iter()
    .filter(|(seconds, ..)| (DURATION..DURATION + FIRST_DRAG).contains(seconds))
    .collect();
  assert!(
    turn.first().is_some_and(|(_, count, _)| *count == 2),
    "the fade starts from everything"
  );
  assert!(
    turn
      .last()
      .is_some_and(|(_, count, coverage)| *count == 0 && *coverage == 0.0),
    "the fade is over before the next stroke"
  );
  for pair in turn.windows(2) {
    let ((at, _, before), (_, _, after)) = (pair[0], pair[1]);
    assert!(
      after <= before,
      "the fade comes back at {at:.3}s: {before} then {after}"
    );
  }
  let (seconds, count, coverage) = *seen.last().expect("frames");
  assert!(
    count == 1 && coverage > 0.0,
    "the next pass's arrow is being drawn at {seconds:.3}s"
  );
}

/// Without keeping, the session's end takes everything at once, and the
/// next pass starts from that empty glass.
#[test]
fn unkept_annotations_go_once_with_the_session() {
  let seen = play_across_the_loop(false);
  for &(seconds, count, coverage) in &seen {
    let empty = count == 0 && coverage == 0.0;
    if seconds < EXIT {
      assert_eq!(
        count, 2,
        "the arrow and counter are on screen at {seconds:.3}s"
      );
    } else if seconds < DURATION + FIRST_DRAG {
      assert!(
        empty,
        "something is drawn at {seconds:.3}s after the session ended"
      );
    }
  }
}
