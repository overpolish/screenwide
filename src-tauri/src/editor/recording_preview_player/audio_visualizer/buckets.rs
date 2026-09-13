// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::AudioRibbonEnvelopes;

/// Match the Metal ribbon: average four envelope points per track, apply its
/// gain, then take the loudest track. The shader only places and paints bars.
pub(crate) fn bucket_levels(envelopes: &AudioRibbonEnvelopes) -> Vec<f32> {
  if envelopes.tracks == 0 {
    return Vec::new();
  }
  let points = envelopes.points as usize;
  (0..points)
    .step_by(4)
    .map(|first| {
      let last = (first + 4).min(points);
      (0..envelopes.tracks as usize)
        .map(|track| {
          let sum: f32 = (first..last)
            .map(|point| {
              envelopes
                .samples
                .get(track * points + point)
                .copied()
                .unwrap_or(0.0)
            })
            .sum();
          sum / (last - first) as f32 * envelopes.gains.get(track).copied().unwrap_or(1.0)
        })
        .fold(0.0, f32::max)
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn tracks_are_averaged_before_the_loudest_is_chosen() {
    let input = AudioRibbonEnvelopes {
      gains: vec![1.0, 0.5],
      tracks: 2,
      points: 4,
      samples: vec![1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
    };
    assert_eq!(bucket_levels(&input), vec![0.375]);
  }

  #[test]
  fn partial_buckets_keep_their_actual_mean_and_muting_removes_their_level() {
    let mut input = AudioRibbonEnvelopes {
      gains: vec![1.0],
      tracks: 1,
      points: 5,
      samples: vec![0.0, 0.0, 0.0, 0.0, 0.75],
    };
    assert_eq!(bucket_levels(&input), vec![0.0, 0.75]);
    input.gains[0] = 0.0;
    assert_eq!(bucket_levels(&input), vec![0.0, 0.0]);
    assert!(bucket_levels(&AudioRibbonEnvelopes::default()).is_empty());
  }
}
