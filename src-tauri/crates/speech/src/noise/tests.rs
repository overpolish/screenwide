// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::f32::consts::TAU;
use std::io::Cursor;

use super::super::analysis::SpeechMap;
use super::profile::{Denoiser, Profile};

const RATE: u32 = 48_000;
const WINDOW_MS: f64 = 32.0;

/// Steady hiss, the same every run.
fn hiss(length: usize) -> Vec<f32> {
  let mut state = 0x2545_f491_u32;
  (0..length)
    .map(|_| {
      state ^= state << 13;
      state ^= state >> 17;
      state ^= state << 5;
      0.01 * (state as f32 / u32::MAX as f32 * 2.0 - 1.0)
    })
    .collect()
}

fn speech(from_ms: f64, to_ms: f64, duration_ms: f64) -> SpeechMap {
  SpeechMap {
    probabilities: (0..(duration_ms / WINDOW_MS) as usize)
      .map(|window| {
        let at = window as f64 * WINDOW_MS;
        if (from_ms..to_ms).contains(&at) {
          1.0
        } else {
          0.0
        }
      })
      .collect(),
    window_ms: WINDOW_MS,
  }
}

fn profile(samples: &[f32], speech: &SpeechMap) -> Option<Profile> {
  let bytes: Vec<u8> = samples
    .iter()
    .flat_map(|sample| sample.to_le_bytes())
    .collect();
  Profile::measure(&mut Cursor::new(bytes), samples.len() as u64, speech, RATE).unwrap()
}

fn denoise(profile: Profile, samples: &[f32]) -> Vec<f32> {
  let mut denoiser = Denoiser::new(profile);
  let mut all = Vec::new();
  let mut out = Vec::new();
  for chunk in samples.chunks(10_000) {
    denoiser.feed(chunk, &mut out);
    all.extend_from_slice(&out);
  }
  denoiser.finish(&mut out);
  all.extend_from_slice(&out);
  all
}

/// The level of `hz` in `samples`, and the level of everything else, in dB.
fn tone_and_rest_db(samples: &[f32], hz: f32) -> (f32, f32) {
  let (mut sin, mut cos) = (0.0_f64, 0.0_f64);
  for (at, &sample) in samples.iter().enumerate() {
    let phase = TAU * hz * at as f32 / RATE as f32;
    sin += f64::from(sample * phase.sin());
    cos += f64::from(sample * phase.cos());
  }
  let length = samples.len() as f64;
  let tone = 2.0 * (sin * sin + cos * cos) / (length * length);
  let all = samples.iter().map(|&s| f64::from(s * s)).sum::<f64>() / length;
  let db = |power: f64| (10.0 * power.max(1e-20).log10()) as f32;
  (db(tone), db(all - tone))
}

#[test]
fn takes_the_hiss_out_from_under_the_voice_and_keeps_the_voice() {
  let length = 4 * RATE as usize;
  let (from, to) = (3 * RATE as usize / 2, 3 * RATE as usize);
  let mut recording = hiss(length);
  for (at, sample) in recording[from..to].iter_mut().enumerate() {
    *sample += 0.1 * (TAU * 440.0 * at as f32 / RATE as f32).sin();
  }
  let map = speech(1_500.0, 3_000.0, 4_000.0);
  let cleaned = denoise(profile(&recording, &map).unwrap(), &recording);
  assert_eq!(cleaned.len(), recording.len());

  // Away from the speech's edges, where the gains have settled.
  let middle = from + RATE as usize / 4..to - RATE as usize / 4;
  let (voice_before, hiss_before) = tone_and_rest_db(&recording[middle.clone()], 440.0);
  let (voice_after, hiss_after) = tone_and_rest_db(&cleaned[middle], 440.0);
  assert!(
    (voice_before - voice_after).abs() < 0.5,
    "voice {voice_before} → {voice_after} dB"
  );
  assert!(
    hiss_before - hiss_after > 9.0,
    "hiss {hiss_before} → {hiss_after} dB"
  );
}

#[test]
fn leaves_a_track_alone_that_is_all_speech() {
  let recording = hiss(4 * RATE as usize);
  assert!(profile(&recording, &speech(0.0, 4_000.0, 4_000.0)).is_none());
}
