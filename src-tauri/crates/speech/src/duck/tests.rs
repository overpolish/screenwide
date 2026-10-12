// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::f32::consts::TAU;

use realfft::num_complex::Complex32;

use super::super::analysis::SpeechMap;
use super::super::framer::{Framer, HOP, SIZE};
use super::gate::gate;
use super::plan::{Plan, VoiceBands};

const RATE: u32 = 48_000;
const WINDOW_MS: f64 = 32.0;

/// A listen that hears speech over each of `spoken`, in milliseconds.
fn speech(spoken: &[(f64, f64)], duration_ms: f64) -> SpeechMap {
  let windows = (duration_ms / WINDOW_MS) as usize;
  SpeechMap {
    probabilities: (0..windows)
      .map(|window| {
        let at = window as f64 * WINDOW_MS;
        if spoken.iter().any(|&(start, end)| at >= start && at < end) {
          1.0
        } else {
          0.0
        }
      })
      .collect(),
    window_ms: WINDOW_MS,
  }
}

/// The gate every 10 ms over `duration_ms`.
fn gate_every_10_ms(spoken: &[(f64, f64)], duration_ms: f64) -> Vec<f32> {
  let frames = (duration_ms / 10.0) as usize;
  gate(
    &speech(spoken, duration_ms),
    duration_ms,
    frames,
    |frame| frame as f64 * 10.0,
    10.0,
  )
}

#[test]
fn makes_way_while_the_voice_speaks_and_not_while_it_is_quiet() {
  let open = gate_every_10_ms(&[(2_000.0, 4_000.0)], 8_000.0);
  assert_eq!(open[100], 0.0);
  assert!(open[300] > 0.99, "{}", open[300]);
  assert!(open[700] < 0.01, "{}", open[700]);
}

#[test]
fn has_made_way_by_the_time_the_first_word_starts() {
  let open = gate_every_10_ms(&[(2_016.0, 4_000.0)], 8_000.0);
  assert!(open[202] > 0.9, "{}", open[202]);
}

#[test]
fn stays_down_across_a_breath_between_words() {
  // 400 ms between words, less the margins kept either side of speech.
  let open = gate_every_10_ms(&[(1_000.0, 2_000.0), (2_400.0, 3_500.0)], 6_000.0);
  assert!(open[200..240].iter().all(|&open| open > 0.99));
}

#[test]
fn comes_back_up_in_a_long_pause() {
  let open = gate_every_10_ms(&[(1_000.0, 2_000.0), (4_500.0, 5_500.0)], 8_000.0);
  assert!(open[380] < 0.05, "{}", open[380]);
}

/// The spectrum of a frame of `samples`, cut as the framer cuts it.
fn spectra(samples: &[f32]) -> Vec<Vec<Complex32>> {
  let mut framer = Framer::new();
  let mut frames = Vec::new();
  framer.feed(samples, None, &mut |_, spectrum| {
    frames.push(spectrum.to_vec())
  });
  frames
}

fn tone(hz: f32, length: usize) -> Vec<f32> {
  (0..length)
    .map(|at| 0.1 * (TAU * hz * at as f32 / RATE as f32).sin())
    .collect()
}

/// How much frame `frame` of `samples` comes down at `hz`, in decibels.
fn turned_down_db(plan: &mut Plan, samples: &[f32], frame: usize, hz: f32) -> f32 {
  let mut spectrum = spectra(samples)[frame].clone();
  let bin = (hz / RATE as f32 * SIZE as f32).round() as usize;
  let before = spectrum[bin].norm();
  plan.apply(frame, &mut spectrum);
  20.0 * (before / spectrum[bin].norm()).log10()
}

/// A voice: 150 Hz and its harmonics up to 3 kHz.
fn voice(length: usize) -> Vec<f32> {
  (0..length)
    .map(|at| {
      let time = at as f32 / RATE as f32;
      (1..=20)
        .map(|harmonic| 0.02 * (TAU * 150.0 * harmonic as f32 * time).sin())
        .sum()
    })
    .collect()
}

#[test]
fn turns_down_most_where_the_voice_is_and_less_in_the_bass() {
  let length = RATE as usize;
  let mut bands = VoiceBands::default();
  for spectrum in spectra(&voice(length)) {
    bands.add(&spectrum, RATE);
  }
  let frames = length / HOP;
  let mut plan = Plan::new(bands, vec![1.0; frames], RATE);
  let at_voice = turned_down_db(&mut plan, &tone(900.0, length), frames / 2, 900.0);
  let bass = turned_down_db(&mut plan, &tone(60.0, length), frames / 2, 60.0);
  assert!((at_voice - 12.0).abs() < 1.0, "at the voice {at_voice} dB");
  assert!((bass - 6.0).abs() < 0.5, "in the bass {bass} dB");
}

#[test]
fn leaves_the_audio_alone_where_the_voice_is_quiet() {
  let length = RATE as usize;
  let mut voice = VoiceBands::default();
  for spectrum in spectra(&tone(800.0, length)) {
    voice.add(&spectrum, RATE);
  }
  let frames = length / HOP;
  let mut plan = Plan::new(voice, vec![0.0; frames], RATE);
  let untouched = turned_down_db(&mut plan, &tone(800.0, length), frames / 2, 800.0);
  assert!(untouched.abs() < 0.01, "{untouched} dB");
}

#[test]
fn puts_frames_left_alone_back_together_exactly() {
  let samples: Vec<f32> = tone(440.0, RATE as usize);
  let mut framer = Framer::new();
  let mut out = Vec::new();
  framer.feed(&samples, Some(&mut out), &mut |_, _| {});
  framer.feed(&[0.0; SIZE], Some(&mut out), &mut |_, _| {});
  let latency = SIZE - HOP;
  for (made, sample) in out[latency..latency + samples.len()].iter().zip(&samples) {
    assert!((made - sample).abs() < 1e-5);
  }
}
