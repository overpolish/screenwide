// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Gaps: time inserted into the timeline with no recording behind it. A gap
//! is a range whose source lies past the recording's end, each in a stretch
//! of that empty source of its own, so every moment of the timeline still
//! has a source moment of its own and the mapping both ways stays one to
//! one. Nothing was recorded there: no picture, no sound, no cursor.

use crate::TimelineRange;

impl TimelineRange {
  /// Whether the range is a gap on a recording `source_duration_us` long.
  pub fn is_gap(&self, source_duration_us: u64) -> bool {
    self.source_start_us >= source_duration_us
  }

  /// Where the range ends on the output.
  pub fn output_end_us(&self) -> u64 {
    self.output_start_us.saturating_add(
      ((self.source_end_us.saturating_sub(self.source_start_us)) as f64 / self.playback_rate)
        .round() as u64,
    )
  }
}
