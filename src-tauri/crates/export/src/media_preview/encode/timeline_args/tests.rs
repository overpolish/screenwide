// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{atempo, cut_fades, TimelineRange};

const RANGE: TimelineRange = TimelineRange {
  output_start_us: 0,
  source_end_us: 250_000,
  source_start_us: 0,
  playback_rate: 1.0,
};

#[test]
fn smooths_both_sides_of_a_cut_without_changing_range_duration() {
  assert_eq!(
    cut_fades(RANGE, 0, 2),
    ",afade=t=out:st=0.247000:d=0.003000"
  );
  assert_eq!(cut_fades(RANGE, 1, 2), ",afade=t=in:st=0:d=0.003000");
  assert!(cut_fades(RANGE, 0, 1).is_empty());
}

#[test]
fn chains_atempo_filters_at_supported_extremes() {
  assert_eq!(atempo(0.25), "atempo=0.5,atempo=0.500000");
  assert_eq!(atempo(4.0), "atempo=2.0,atempo=2.000000");
}
