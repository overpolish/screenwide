// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{envelopes, source_ratio, AudioVisualizerTrack, MAX_RIBBON_POINTS, MAX_RIBBON_TRACKS};

fn track(gain_decibels: f64, waveform: Vec<f32>) -> AudioVisualizerTrack {
  AudioVisualizerTrack {
    gain_decibels,
    stream_index: 0,
    waveform,
  }
}

fn numbered(stream_index: usize, waveform: Vec<f32>) -> AudioVisualizerTrack {
  AudioVisualizerTrack {
    gain_decibels: 0.0,
    stream_index,
    waveform,
  }
}

#[test]
fn uploads_one_row_per_track_at_a_shared_point_count() {
  let uploaded = envelopes(&[
    AudioVisualizerTrack {
      gain_decibels: 0.0,
      stream_index: 0,
      waveform: vec![0.25; 8],
    },
    AudioVisualizerTrack {
      gain_decibels: -6.0,
      stream_index: 1,
      waveform: vec![0.5; 4],
    },
  ]);
  assert_eq!(uploaded.tracks, 2);
  assert_eq!(uploaded.points, 8);
  assert_eq!(uploaded.samples.len(), 16);
  // The shorter track is stretched rather than padded with silence.
  assert!(uploaded.samples[8..].iter().all(|value| *value == 0.5));
  assert!((uploaded.gains[0] - 1.0).abs() < 0.000_1);
  assert!((uploaded.gains[1] - 0.501).abs() < 0.001);
}

#[test]
fn clamps_the_track_count_and_the_point_count() {
  let many = (0..6)
    .map(|index| numbered(index, vec![0.5; MAX_RIBBON_POINTS * 2 + index]))
    .collect::<Vec<_>>();
  let uploaded = envelopes(&many);
  assert_eq!(uploaded.tracks, MAX_RIBBON_TRACKS as u32);
  assert_eq!(uploaded.points, MAX_RIBBON_POINTS as u32);
  assert_eq!(
    uploaded.samples.len(),
    MAX_RIBBON_TRACKS * MAX_RIBBON_POINTS
  );
  assert_eq!(uploaded.gains.len(), MAX_RIBBON_TRACKS);
}

#[test]
fn an_empty_or_silent_track_list_clears_the_bars() {
  assert_eq!(envelopes(&[]).tracks, 0);
  assert_eq!(envelopes(&[track(0.0, Vec::new())]).points, 0);
  assert!(envelopes(&[track(0.0, Vec::new())]).samples.is_empty());
}

#[test]
fn a_track_named_twice_takes_one_row() {
  let uploaded = envelopes(&[numbered(3, vec![0.5; 4]), numbered(3, vec![0.5; 4])]);
  assert_eq!(uploaded.tracks, 1);
}

#[test]
fn keeps_every_sample_inside_the_unit_range() {
  let uploaded = envelopes(&[track(0.0, vec![f32::NAN, -3.0, 4.0, 0.5])]);
  assert_eq!(uploaded.samples, vec![0.0, 0.0, 1.0, 0.5]);
}

#[test]
fn ribbon_uses_source_time_after_a_cut_instead_of_compressed_timeline_time() {
  assert_eq!(source_ratio(7_018, 8_125), 0.8637538461538462);
  assert_eq!(source_ratio(9_000, 8_125), 1.0);
  assert_eq!(source_ratio(7_018, 0), 0.0);
}
