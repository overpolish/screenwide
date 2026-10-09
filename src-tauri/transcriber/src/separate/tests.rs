// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{overlap_add, SEGMENT, SIDES, STRIDE};

/// Interleaved stereo frames, each side different and no two frames alike.
fn audio(frames: usize) -> Vec<f32> {
  (0..frames * SIDES)
    .map(|index| ((index as f32) * 0.000_37).sin() * if index % 2 == 0 { 0.8 } else { -0.5 })
    .collect()
}

#[test]
fn a_model_giving_back_what_it_hears_gives_back_the_audio_at_any_length() {
  for frames in [1, 1_000, SEGMENT - 1, SEGMENT, SEGMENT + 1, STRIDE * 3 + 17] {
    let samples = audio(frames);
    let bytes: Vec<u8> = samples
      .iter()
      .flat_map(|sample| sample.to_le_bytes())
      .collect();
    let mut written = Vec::new();
    let mut segments = 0;
    let mut last_progress = 0.0;
    overlap_add(
      &mut bytes.as_slice(),
      &mut written,
      frames,
      &mut |planar| {
        segments += 1;
        Ok(planar)
      },
      &mut |fraction| last_progress = fraction,
    )
    .unwrap();
    let out: Vec<f32> = written
      .as_chunks::<4>()
      .0
      .iter()
      .map(|&sample| f32::from_le_bytes(sample))
      .collect();
    assert_eq!(out.len(), samples.len(), "{frames} frames");
    let worst = out
      .iter()
      .zip(&samples)
      .map(|(made, sample)| (made - sample).abs())
      .fold(0.0_f32, f32::max);
    assert!(worst < 1e-6, "{frames} frames: off by {worst}");
    assert_eq!(
      segments,
      frames.saturating_sub(SEGMENT).div_ceil(STRIDE) + 1,
      "{frames} frames"
    );
    assert_eq!(last_progress, 1.0, "{frames} frames");
  }
}
