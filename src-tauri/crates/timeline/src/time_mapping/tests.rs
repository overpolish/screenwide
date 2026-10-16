// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

fn range(output_start_us: u64, source_start_us: u64, source_end_us: u64) -> TimelineRange {
  TimelineRange {
    output_start_us,
    source_end_us,
    source_start_us,
    playback_rate: 1.0,
  }
}

/// Keeps 0-1 s, 2-3 s and 4-5 s of the source and plays them as 4-5 s, 0-1 s,
/// 2-3 s.
fn reordered() -> [TimelineRange; 3] {
  [
    range(0, 4_000_000, 5_000_000),
    range(1_000_000, 0, 1_000_000),
    range(2_000_000, 2_000_000, 3_000_000),
  ]
}

#[test]
fn reordered_ranges_map_both_ways() {
  let ranges = reordered();
  assert_eq!(source_to_output_us(&ranges, 4_500_000), Some(500_000));
  assert_eq!(source_to_output_us(&ranges, 500_000), Some(1_500_000));
  assert_eq!(output_to_source_us(&ranges, 2_500_000), Some(2_500_000));
  assert_eq!(output_to_source_us(&ranges, 500_000), Some(4_500_000));
}

#[test]
fn a_cut_moment_lands_where_its_source_predecessor_ends() {
  let ranges = reordered();
  // 1.5 s follows 0-1 s in the source, which ends 2 s into the output.
  assert_eq!(output_at_us(&ranges, 1_500_000.0), 2_000_000.0);
  // 3.5 s follows 2-3 s, the last range played.
  assert_eq!(output_at_us(&ranges, 3_500_000.0), 3_000_000.0);
  // Past the end of the source, after 4-5 s, played first.
  assert_eq!(output_at_us(&ranges, 6_000_000.0), 1_000_000.0);
}

#[test]
fn a_stretch_ending_where_another_range_starts_ends_with_its_own_range() {
  let ranges = reordered();
  // 1 s ends 0-1 s, which is played second and ends 2 s into the output.
  assert_eq!(output_end_at_us(&ranges, 1_000_000.0), 2_000_000.0);
  // 4 s ends no range: a stretch ending there ends after its source
  // predecessor, 2-3 s, not at the start of 4-5 s.
  assert_eq!(output_end_at_us(&ranges, 4_000_000.0), 3_000_000.0);
  let swapped = [
    range(0, 1_000_000, 2_000_000),
    range(1_000_000, 0, 1_000_000),
  ];
  assert_eq!(output_at_us(&swapped, 1_000_000.0), 0.0);
  assert_eq!(output_end_at_us(&swapped, 1_000_000.0), 2_000_000.0);
}

#[test]
fn a_moment_before_every_range_lands_at_the_earliest_source_range() {
  let ranges = [
    range(0, 3_000_000, 4_000_000),
    range(1_000_000, 1_000_000, 2_000_000),
  ];
  assert_eq!(output_at_us(&ranges, 500_000.0), 1_000_000.0);
}

#[test]
fn ranges_in_source_order_keep_the_cut_join_rule() {
  let ranges = [
    range(0, 0, 1_000_000),
    range(1_000_000, 2_000_000, 3_000_000),
  ];
  assert_eq!(output_at_us(&ranges, 1_500_000.0), 1_000_000.0);
  assert_eq!(output_at_us(&ranges, 9_000_000.0), 2_000_000.0);
}

#[test]
fn plays_through_follows_runs_not_source_order() {
  let ranges = reordered();
  // 0-1 s then 2-3 s play back to back, across an ordinary cut.
  assert!(plays_through(&ranges, 500_000, 2_500_000));
  // 4-5 s plays first, so 0.5 s never carries on into it.
  assert!(!plays_through(&ranges, 500_000, 4_500_000));
  // Cut away.
  assert!(!plays_through(&ranges, 1_500_000, 2_500_000));
}

#[test]
fn plays_through_stops_where_skipped_source_plays_elsewhere() {
  // 0-1 s, then 2-3 s, then 1-2 s: going from 0.5 s to 2.5 s jumps over
  // 1-2 s, which is still played, later.
  let ranges = [
    range(0, 0, 1_000_000),
    range(1_000_000, 2_000_000, 3_000_000),
    range(2_000_000, 1_000_000, 2_000_000),
  ];
  assert!(!plays_through(&ranges, 500_000, 2_500_000));
  assert!(plays_through(&ranges, 1_200_000, 1_800_000));
}

#[test]
fn a_reader_starts_a_new_piece_where_the_timeline_goes_back() {
  // 4-5 s, then 0-1 s, then 2-3 s: back once, then on.
  assert_eq!(
    ascending_source_pieces(&reordered()),
    [[4_000_000, 5_000_000], [0, 3_000_000]]
  );
  let in_order = [
    range(0, 0, 1_000_000),
    range(1_000_000, 2_000_000, 3_000_000),
  ];
  assert_eq!(ascending_source_pieces(&in_order), [[0, 3_000_000]]);
}

#[test]
fn output_windows_keep_their_source_across_reordered_ranges() {
  let kept = within_output_windows(
    &reordered(),
    &[[500_000, 1_500_000], [2_800_000, 3_400_000]],
  );
  assert_eq!(
    kept,
    [
      range(0, 4_500_000, 5_000_000),
      range(500_000, 0, 500_000),
      range(1_000_000, 2_800_000, 3_000_000),
    ]
  );
}

#[test]
fn output_windows_on_a_faster_range_take_more_source() {
  let fast = [TimelineRange {
    playback_rate: 2.0,
    ..range(0, 0, 4_000_000)
  }];
  let kept = within_output_windows(&fast, &[[500_000, 1_000_000]]);
  assert_eq!(
    kept,
    [TimelineRange {
      playback_rate: 2.0,
      ..range(0, 1_000_000, 2_000_000)
    }]
  );
  assert_eq!(output_duration_us(&kept), Some(500_000));
}
