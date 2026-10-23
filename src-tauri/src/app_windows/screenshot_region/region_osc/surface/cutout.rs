// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The cutout round a window being picked fades in as it appears and out as
//! it goes, on the island's state timing, so the overlay settles rather than
//! snaps. From one window to the next it moves at once.

use std::time::{Duration, Instant};

const FADE: Duration = Duration::from_millis(120);

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct CutoutFade {
  from: f32,
  to: f32,
  since: Option<Instant>,
}

impl CutoutFade {
  /// Heads for shown or gone from wherever the fade has got to.
  pub(crate) fn head_for(&mut self, shown: bool, now: Instant) {
    let to = if shown { 1.0 } else { 0.0 };
    if to == self.to {
      return;
    }
    self.from = self.opacity(now);
    self.to = to;
    self.since = Some(now);
  }

  /// How strongly the shade round the window shows, eased out.
  pub(crate) fn opacity(&self, now: Instant) -> f32 {
    let Some(since) = self.since else {
      return self.to;
    };
    let progress = now.saturating_duration_since(since).as_secs_f32() / FADE.as_secs_f32();
    if progress >= 1.0 {
      return self.to;
    }
    let eased = 1.0 - (1.0 - progress).powi(2);
    self.from + (self.to - self.from) * eased
  }

  pub(crate) fn is_fading(&self, now: Instant) -> bool {
    self
      .since
      .is_some_and(|since| now.saturating_duration_since(since) < FADE)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn fades_in_and_out_and_turns_back_from_where_it_is() {
    let start = Instant::now();
    let mut cutout = CutoutFade::default();
    cutout.head_for(true, start);
    assert_eq!(cutout.opacity(start), 0.0);
    assert_eq!(cutout.opacity(start + FADE), 1.0);
    let midway = start + FADE / 2;
    let reached = cutout.opacity(midway);
    assert!(reached > 0.5 && reached < 1.0, "eased out: {reached}");
    cutout.head_for(false, midway);
    assert_eq!(cutout.opacity(midway), reached);
    assert_eq!(cutout.opacity(midway + FADE), 0.0);
    assert!(!cutout.is_fading(midway + FADE));
  }
}
