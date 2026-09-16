// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Recording time, shared by every optional sidecar.
//!
//! One clock per sidecar, all reading the capture's single origin, so a pause
//! removes exactly the same span from cursor, keyboard and annotation
//! timestamps as it removes from the movie. The origin is stamped by the first
//! frame, which is why it arrives as a `OnceLock` rather than as a value.

use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub(crate) struct SidecarClock {
  origin: Arc<OnceLock<Instant>>,
  paused_since: Option<Instant>,
  paused_total: Duration,
  running: bool,
}

impl SidecarClock {
  pub(crate) fn new(origin: Arc<OnceLock<Instant>>) -> Self {
    Self {
      origin,
      paused_since: None,
      paused_total: Duration::ZERO,
      running: true,
    }
  }

  pub(crate) fn pause(&mut self, at: Instant) {
    if self.paused_since.is_none() {
      self.paused_since = Some(at);
    }
  }

  pub(crate) fn resume(&mut self, at: Instant) {
    if let Some(paused_since) = self.paused_since.take() {
      self.paused_total = self
        .paused_total
        .saturating_add(at.saturating_duration_since(paused_since));
    }
  }

  pub(crate) fn stop(&mut self) {
    self.running = false;
  }

  /// Whether events are being timed at all.
  pub(crate) const fn is_live(&self) -> bool {
    self.running && self.paused_since.is_none()
  }

  /// Paused time so far, counting an open pause up to `at`.
  fn paused_total_at(&self, at: Instant) -> Duration {
    match self.paused_since {
      Some(since) => self
        .paused_total
        .saturating_add(at.saturating_duration_since(since)),
      None => self.paused_total,
    }
  }

  /// Recording time at `at`, and nothing at all while the clock is paused or
  /// stopped: a paused recording does not grow, so the event that would have
  /// been timed is dropped instead.
  pub(crate) fn timestamp_us(&self, at: Instant) -> Option<u64> {
    if !self.is_live() {
      return None;
    }
    self.elapsed_us(at)
  }

  /// Recording time at `at` whatever the clock's state, with an open pause
  /// already removed. This is what closes out state that was still on screen
  /// when a paused or stopping recording ended.
  pub(crate) fn elapsed_us(&self, at: Instant) -> Option<u64> {
    let origin = *self.origin.get()?;
    let elapsed = at
      .saturating_duration_since(origin)
      .saturating_sub(self.paused_total_at(at));
    u64::try_from(elapsed.as_micros()).ok()
  }

  /// Time zero, for a state snapshot that belongs at the recording's first
  /// frame rather than at the moment it was taken.
  pub(crate) fn initial_timestamp_us(&self) -> Option<u64> {
    (self.is_live() && self.origin.get().is_some()).then_some(0)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn started() -> (Instant, SidecarClock) {
    let origin = Instant::now();
    let shared = Arc::new(OnceLock::new());
    shared.set(origin).unwrap();
    (origin, SidecarClock::new(shared))
  }

  #[test]
  fn a_pause_is_removed_from_later_timestamps() {
    let (origin, mut clock) = started();

    assert_eq!(
      clock.timestamp_us(origin + Duration::from_secs(2)),
      Some(2_000_000)
    );
    clock.pause(origin + Duration::from_secs(3));
    assert_eq!(clock.timestamp_us(origin + Duration::from_secs(5)), None);
    clock.resume(origin + Duration::from_secs(8));
    assert_eq!(
      clock.timestamp_us(origin + Duration::from_secs(9)),
      Some(4_000_000)
    );
  }

  #[test]
  fn a_stopped_clock_times_nothing() {
    let (origin, mut clock) = started();

    clock.stop();
    assert_eq!(clock.timestamp_us(origin + Duration::from_secs(1)), None);
    assert_eq!(clock.initial_timestamp_us(), None);
  }

  #[test]
  fn nothing_is_timed_before_the_first_frame() {
    let clock = SidecarClock::new(Arc::new(OnceLock::new()));

    assert_eq!(clock.timestamp_us(Instant::now()), None);
    assert_eq!(clock.initial_timestamp_us(), None);
    assert!(clock.is_live());
  }

  #[test]
  fn elapsed_time_counts_an_open_pause_up_to_the_stop() {
    let (origin, mut clock) = started();

    clock.pause(origin + Duration::from_secs(3));
    // Stopped three seconds of wall time into the pause: the recording is
    // still three seconds long.
    assert_eq!(
      clock.elapsed_us(origin + Duration::from_secs(6)),
      Some(3_000_000)
    );
  }

  #[test]
  fn a_snapshot_is_timed_at_zero_once_a_frame_has_landed() {
    let (_, clock) = started();

    assert_eq!(clock.initial_timestamp_us(), Some(0));
  }
}
