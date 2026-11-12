// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::spectrum::Spectrum;
use super::{chunked, plausible, CHUNK, SIDES};

/// Interleaved stereo frames, each side different and no two frames alike.
fn audio(frames: usize) -> Vec<f32> {
  (0..frames * SIDES)
    .map(|index| ((index as f32) * 0.000_37).sin() * if index % 2 == 0 { 0.8 } else { -0.5 })
    .collect()
}

#[test]
fn a_chunk_comes_back_from_its_own_spectrum() {
  // Planar: the left side, then the right, with sound up to both edges,
  // where the frames reach past the chunk and reflect.
  let planar: Vec<f32> = (0..SIDES * CHUNK)
    .map(|index| ((index as f32) * 0.013).sin() * 0.7 + ((index as f32) * 0.000_91).cos() * 0.2)
    .collect();
  let mut spectrum = Spectrum::new();
  let analysed = spectrum.analyse(&planar);
  let back = spectrum.synthesise(&analysed);
  let worst = back
    .iter()
    .zip(&planar)
    .map(|(made, sample)| (made - sample).abs())
    .fold(0.0_f32, f32::max);
  assert!(worst < 1e-4, "off by {worst}");
}

#[test]
fn a_model_giving_back_what_it_hears_gives_back_the_audio_at_any_length() {
  for frames in [1, 1_000, CHUNK - 1, CHUNK, CHUNK + 1, CHUNK * 3 + 17] {
    let samples = audio(frames);
    let bytes: Vec<u8> = samples
      .iter()
      .flat_map(|sample| sample.to_le_bytes())
      .collect();
    let mut written = Vec::new();
    let mut chunks = 0;
    let mut last_progress = 0.0;
    chunked(
      &mut bytes.as_slice(),
      &mut written,
      frames,
      &mut |planar| {
        chunks += 1;
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
    assert_eq!(out, samples, "{frames} frames");
    assert_eq!(chunks, frames.div_ceil(CHUNK), "{frames} frames");
    assert_eq!(last_progress, 1.0, "{frames} frames");
  }
}

#[test]
fn a_gpu_answer_is_kept_only_when_singing_could_be_it() {
  // Up to twice the mix's peak, with a margin for near silence.
  assert!(plausible(vec![0.0, -0.6, 0.6], 0.3).is_ok());
  assert!(plausible(vec![0.009], 0.0).is_ok());
  assert!(plausible(vec![0.0, 0.62], 0.3).is_err());
  assert!(plausible(vec![0.011], 0.0).is_err());
  // A broken GPU's answers.
  assert!(plausible(vec![0.1, f32::NAN], 0.3).is_err());
  assert!(plausible(vec![f32::INFINITY], 0.3).is_err());
  assert!(plausible(vec![-1_800.0], 0.3).is_err());
}
