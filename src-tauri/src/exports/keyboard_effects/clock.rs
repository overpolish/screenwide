// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Maps recorded source time onto the edited output clock used by animations.

use crate::exports::timeline_edit::{source_to_output_us, TimelineRange};

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct AnimationClock<'a> {
  ranges: Option<&'a [TimelineRange]>,
}

impl<'a> AnimationClock<'a> {
  pub(super) const fn source() -> Self {
    Self { ranges: None }
  }

  pub(super) const fn edited(ranges: &'a [TimelineRange]) -> Self {
    Self {
      ranges: Some(ranges),
    }
  }

  pub(super) fn elapsed_us(self, start_us: u64, now_us: u64) -> u64 {
    if now_us <= start_us {
      return 0;
    }
    let Some(ranges) = self.ranges else {
      return now_us - start_us;
    };
    let Some(now_output_us) = source_to_output_us(ranges, now_us) else {
      return 0;
    };
    let Some(start_output_us) = source_to_output_us(ranges, start_us) else {
      // The animation began in content that was cut. Any state which survives
      // the cut must already be settled at the next retained frame.
      return u64::MAX;
    };
    now_output_us.saturating_sub(start_output_us)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn range(
    output_start_us: u64,
    source_start_us: u64,
    source_end_us: u64,
    playback_rate: f64,
  ) -> TimelineRange {
    TimelineRange {
      output_start_us,
      source_start_us,
      source_end_us,
      playback_rate,
    }
  }

  #[test]
  fn edited_elapsed_integrates_rates_across_retained_ranges() {
    let ranges = [
      range(0, 0, 1_000_000, 2.0),
      range(500_000, 1_000_000, 2_000_000, 0.5),
    ];
    let clock = AnimationClock::edited(&ranges);
    assert_eq!(clock.elapsed_us(500_000, 1_250_000), 750_000);
  }

  #[test]
  fn an_anchor_removed_by_a_cut_is_already_settled() {
    let ranges = [range(0, 1_000_000, 2_000_000, 1.0)];
    assert_eq!(
      AnimationClock::edited(&ranges).elapsed_us(500_000, 1_000_000),
      u64::MAX
    );
  }
}
