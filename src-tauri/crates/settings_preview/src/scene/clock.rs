// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A recording scene's time, counted in the display's refreshes rather than
//! read off the wall clock. The surface hands out a frame once a refresh
//! has taken the last one, so each frame drawn is shown for one refresh;
//! counting them gives every frame the same step of scene time. Read off
//! the wall clock between a timer's wake-ups, a 60 fps recording's frames
//! landed on the refreshes unevenly, some held for two and the next one
//! skipped, which is the judder a 60 fps picture must not have.

use std::time::{Duration, Instant};

/// Refresh rates displays run at, which a measured interval is taken as
/// when it is near one: a measurement is never exact, and a refresh counted
/// a little short or long would drift the picture off its frames.
const RATES: [f64; 14] = [
  24.0, 30.0, 48.0, 50.0, 60.0, 72.0, 75.0, 90.0, 100.0, 120.0, 144.0, 165.0, 180.0, 240.0,
];
/// How near a measured rate has to be to a display's to be taken as it.
const NEAR: f64 = 0.03;
/// The rate assumed until enough refreshes have been seen.
const ASSUMED: f64 = 60.0;
/// How many intervals the rate is measured over, and how many it needs.
const MEASURED: usize = 32;
const ENOUGH: usize = 8;
/// Shorter intervals are frames handed out at once while the surface still
/// had free ones, not refreshes.
const SHORTEST: Duration = Duration::from_millis(2);
/// An interval this many refreshes long or longer missed some: the display
/// showed the last frame for each of them.
const LATE: f64 = 1.75;

#[derive(Default)]
pub(crate) struct FrameClock {
  last: Option<Instant>,
  /// Recent intervals between frames, in seconds, oldest first.
  intervals: Vec<f64>,
  /// Scene time, in seconds.
  elapsed: f64,
}

impl FrameClock {
  /// The scene time of a frame handed out at `now`: none for the first, then
  /// one refresh more for each, or as many as went by when frames were
  /// missed.
  pub(crate) fn tick(&mut self, now: Instant) -> Duration {
    if let Some(last) = self.last.replace(now) {
      let interval = now.saturating_duration_since(last);
      if interval >= SHORTEST {
        if self.intervals.len() == MEASURED {
          self.intervals.remove(0);
        }
        self.intervals.push(interval.as_secs_f64());
      }
      let refresh = self.refresh();
      let refreshes = interval.as_secs_f64() / refresh;
      self.elapsed += refresh
        * if refreshes >= LATE {
          refreshes.round()
        } else {
          1.0
        };
    }
    Duration::from_secs_f64(self.elapsed)
  }

  /// How long one of the display's refreshes is, in seconds.
  pub(crate) fn refresh(&self) -> f64 {
    if self.intervals.len() < ENOUGH {
      return ASSUMED.recip();
    }
    // The middle half, so a missed refresh or a late wake-up does not count.
    let mut sorted = self.intervals.clone();
    sorted.sort_by(f64::total_cmp);
    let quarter = sorted.len() / 4;
    let middle = &sorted[quarter..sorted.len() - quarter];
    let measured = (middle.len() as f64 / middle.iter().sum::<f64>()).max(1.0);
    let rate = RATES
      .into_iter()
      .find(|rate| (measured / rate - 1.0).abs() <= NEAR)
      .unwrap_or(measured);
    rate.recip()
  }
}

/// Which of a recording's frames at `fps` is shown `elapsed` into it. A
/// refresh can fall right on the edge between two frames, as every other
/// one does for 30 fps at 60 Hz; it takes the later one every time rather
/// than whichever rounding gives, which held some frames for one refresh
/// and others for three.
pub(crate) fn frame_index(elapsed: Duration, fps: u32) -> u64 {
  (elapsed.as_secs_f64() * f64::from(fps) + 1e-6).floor() as u64
}

#[cfg(test)]
mod tests {
  use super::*;

  /// When the surface hands out `count` frames on a display refreshing
  /// `hz` times a second: the first three at once, as it has three free,
  /// then one a refresh, each a little early or late as threads wake.
  fn handed_out(hz: f64, count: usize) -> Vec<Instant> {
    let start = Instant::now();
    let refresh = hz.recip();
    (0..count)
      .map(|frame| {
        let at = if frame < 3 {
          frame as f64 * 0.0002
        } else {
          let wobble = [0.0013, -0.0011, 0.0004, -0.0017, 0.0009][frame % 5];
          (frame - 2) as f64 * refresh + wobble
        };
        start + Duration::from_secs_f64(at)
      })
      .collect()
  }

  /// How many refreshes each of the recording's frames is shown for, past
  /// the first second the rate is learnt in, as a sorted list of distinct
  /// counts.
  fn holds(hz: f64, fps: u32, times: &[Instant]) -> Vec<usize> {
    let mut clock = FrameClock::default();
    let shown: Vec<u64> = times
      .iter()
      .map(|&at| frame_index(clock.tick(at), fps))
      .skip(hz as usize)
      .collect();
    let mut runs = Vec::new();
    let mut run = 1;
    for pair in shown.windows(2) {
      if pair[1] == pair[0] {
        run += 1;
      } else {
        assert_eq!(pair[1], pair[0] + 1, "a frame was skipped at {fps} fps");
        runs.push(run);
        run = 1;
      }
    }
    // The first and last runs are cut by the window.
    let mut distinct: Vec<usize> = runs[1..].to_vec();
    distinct.sort_unstable();
    distinct.dedup();
    distinct
  }

  #[test]
  fn sixty_fps_shows_a_new_frame_every_refresh_at_60_hz() {
    let times = handed_out(60.0, 600);
    assert_eq!(holds(60.0, 60, &times), vec![1]);
    assert_eq!(holds(60.0, 30, &times), vec![2]);
  }

  #[test]
  fn frames_are_held_evenly_at_120_hz() {
    let times = handed_out(120.0, 1_200);
    assert_eq!(holds(120.0, 60, &times), vec![2]);
    assert_eq!(holds(120.0, 30, &times), vec![4]);
  }

  #[test]
  fn the_rate_is_learnt_from_the_refreshes() {
    for hz in [60.0, 120.0, 144.0] {
      let mut clock = FrameClock::default();
      for at in handed_out(hz, 60) {
        clock.tick(at);
      }
      assert!((clock.refresh() - 1.0 / hz).abs() < 1e-9, "{hz} Hz");
    }
  }

  #[test]
  fn a_missed_refresh_moves_the_scene_on_by_it() {
    let mut clock = FrameClock::default();
    let start = Instant::now();
    let refresh = Duration::from_secs_f64(1.0 / 60.0);
    for frame in 0..30 {
      clock.tick(start + refresh * frame);
    }
    let before = clock.tick(start + refresh * 30);
    let after = clock.tick(start + refresh * 33);
    assert_eq!(frame_index(after, 60), frame_index(before, 60) + 3);
  }
}
