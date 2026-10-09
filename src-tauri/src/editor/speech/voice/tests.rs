// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::f32::consts::TAU;

use super::declick::Declicker;
use super::deess::Deesser;
use super::level::{self, Frames};

pub(super) const RATE: u32 = 48_000;

/// Repeatable white noise at `amplitude`.
pub(super) fn noise(length: usize, amplitude: f32, mut seed: u32) -> Vec<f32> {
  (0..length)
    .map(|_| {
      seed ^= seed << 13;
      seed ^= seed >> 17;
      seed ^= seed << 5;
      (seed as f32 / u32::MAX as f32 * 2.0 - 1.0) * amplitude
    })
    .collect()
}

/// A voiced sound: a 150 Hz voice and its harmonics, rising and falling as
/// syllables do, over a quiet room.
pub(super) fn voice(length: usize, amplitude: f32) -> Vec<f32> {
  let room = noise(length, amplitude * 0.01, 7);
  (0..length)
    .map(|at| {
      let time = at as f32 / RATE as f32;
      let syllables = 0.6 + 0.4 * (TAU * 4.0 * time).sin();
      let harmonics: f32 = (1..=12)
        .map(|harmonic| (TAU * 150.0 * harmonic as f32 * time).sin() / harmonic as f32)
        .sum();
      amplitude * syllables * harmonics / 3.0 + room[at]
    })
    .collect()
}

/// A mouth click: a burst of 5 kHz a millisecond long, gone as fast.
fn click(peak: f32) -> Vec<f32> {
  let length = RATE as usize / 1_000;
  (0..length)
    .map(|at| {
      let time = at as f32 / RATE as f32;
      peak * (TAU * 5_000.0 * time).sin() * (-(at as f32) / (length as f32 / 4.0)).exp()
    })
    .collect()
}

pub(super) fn add(samples: &mut [f32], at: usize, sound: &[f32]) {
  for (sample, added) in samples[at..].iter_mut().zip(sound) {
    *sample += added;
  }
}

pub(super) fn energy(samples: &[f32]) -> f32 {
  samples.iter().map(|sample| sample * sample).sum::<f32>() / samples.len() as f32
}

pub(super) fn decibels(ratio: f32) -> f32 {
  10.0 * ratio.log10()
}

/// How far a sound added to `clean` at `at`, `length` samples long, was
/// taken down: what is left of it in `cleaned`, against the sound itself.
fn taken_down_db(
  clean: &[f32],
  added: &[f32],
  cleaned: &[f32],
  (at, length): (usize, usize),
) -> f32 {
  let window = at - 48..at + length + 96;
  let left: Vec<f32> = window.clone().map(|i| cleaned[i] - clean[i]).collect();
  let added: Vec<f32> = window.map(|i| added[i] - clean[i]).collect();
  decibels(energy(&added) / energy(&left))
}

fn click_taken_down_db(clean: &[f32], clicked: &[f32], cleaned: &[f32], at: usize) -> f32 {
  taken_down_db(clean, clicked, cleaned, (at, 48))
}

#[test]
fn takes_a_click_out_of_a_pause() {
  let clean = noise(RATE as usize, 0.001, 3);
  let mut clicked = clean.clone();
  add(&mut clicked, 24_000, &click(0.2));
  let mut cleaned = clicked.clone();
  Declicker::new(RATE).apply(&mut cleaned);
  let taken = click_taken_down_db(&clean, &clicked, &cleaned, 24_000);
  assert!(taken > 15.0, "taken down by {taken} dB");
}

#[test]
fn takes_a_click_out_of_a_held_vowel() {
  let clean = voice(RATE as usize, 0.1);
  let mut clicked = clean.clone();
  // Where the syllable is at its loudest and steady.
  add(&mut clicked, 15_000, &click(0.1));
  let mut cleaned = clicked.clone();
  Declicker::new(RATE).apply(&mut cleaned);
  let taken = click_taken_down_db(&clean, &clicked, &cleaned, 15_000);
  assert!(taken > 10.0, "taken down by {taken} dB");
}

#[test]
fn takes_out_a_wet_click() {
  // A smack 12 ms long: three ticks and the hiss of lips parting.
  let clean = noise(RATE as usize, 0.001, 3);
  let mut smacked = clean.clone();
  for (offset, peak) in [(0, 0.15), (200, 0.1), (430, 0.12)] {
    add(&mut smacked, 24_000 + offset, &click(peak));
  }
  let hiss: Vec<f32> = noise(576, 0.05, 23)
    .windows(2)
    .enumerate()
    .map(|(at, pair)| (pair[1] - pair[0]) * (-(at as f32) / 200.0).exp())
    .collect();
  add(&mut smacked, 24_000, &hiss);
  let mut cleaned = smacked.clone();
  Declicker::new(RATE).apply(&mut cleaned);
  let taken = taken_down_db(&clean, &smacked, &cleaned, (24_000, 576));
  assert!(taken > 10.0, "taken down by {taken} dB");
}

#[test]
fn leaves_a_voice_without_clicks_as_it_was() {
  let clean = voice(RATE as usize * 2, 0.1);
  let mut cleaned = clean.clone();
  Declicker::new(RATE).apply(&mut cleaned);
  assert_eq!(cleaned, clean);
}

#[test]
fn takes_out_both_of_two_clicks_in_quick_succession() {
  // A wet smack: two clicks 4 ms apart, the second as loud as the first.
  let clean = noise(RATE as usize, 0.001, 3);
  let mut clicked = clean.clone();
  add(&mut clicked, 24_000, &click(0.2));
  add(&mut clicked, 24_192, &click(0.2));
  let mut cleaned = clicked.clone();
  Declicker::new(RATE).apply(&mut cleaned);
  for at in [24_000, 24_192] {
    let taken = click_taken_down_db(&clean, &clicked, &cleaned, at);
    assert!(taken > 15.0, "click at {at} taken down by {taken} dB");
  }
}

#[test]
fn leaves_a_consonant_alone() {
  // A "t": a sharp release, then breath that fades over 40 ms.
  let mut sound = noise(RATE as usize, 0.0001, 5);
  let release = noise(96, 0.3, 11);
  let breath: Vec<f32> = noise(1_920, 0.12, 13)
    .into_iter()
    .enumerate()
    .map(|(at, sample)| sample * (-(at as f32) / 640.0).exp())
    .collect();
  add(&mut sound, 20_000, &release);
  add(&mut sound, 20_096, &breath);
  let mut cleaned = sound.clone();
  Declicker::new(RATE).apply(&mut cleaned);
  assert_eq!(cleaned, sound);
}

#[test]
fn leaves_a_consonant_leading_into_a_vowel_alone() {
  // "ta": the tongue's release, 30 ms of breath, then the vowel.
  let mut sound = noise(RATE as usize, 0.0001, 5);
  let release = noise(96, 0.3, 11);
  let breath: Vec<f32> = noise(1_440, 0.12, 13)
    .into_iter()
    .enumerate()
    .map(|(at, sample)| sample * (-(at as f32) / 480.0).exp())
    .collect();
  add(&mut sound, 20_000, &release);
  add(&mut sound, 20_096, &breath);
  add(&mut sound, 21_536, &voice(9_600, 0.1));
  let mut cleaned = sound.clone();
  Declicker::new(RATE).apply(&mut cleaned);
  assert_eq!(cleaned, sound);
}

#[test]
fn a_click_across_two_runs_is_taken_out_the_same_as_in_one() {
  let clean = noise(RATE as usize, 0.001, 3);
  let mut clicked = clean.clone();
  let mut declicker = Declicker::new(RATE);
  // Runs that meet in the middle of the click, each reading 340 ms past its
  // edge, as the cleanup reads a long track; both start on a frame.
  let (edge, context) = (24_064, 1 << 14);
  add(&mut clicked, edge - 20, &click(0.2));
  let mut whole = clicked.clone();
  declicker.apply(&mut whole);
  let mut first = clicked[..edge + context].to_vec();
  declicker.apply(&mut first);
  let mut second = clicked[edge - context..].to_vec();
  declicker.apply(&mut second);
  let joined: Vec<f32> = first[..edge]
    .iter()
    .chain(&second[context..])
    .copied()
    .collect();
  assert_ne!(whole, clicked);
  for (joined, whole) in joined.iter().zip(&whole) {
    assert!((joined - whole).abs() < 1e-6);
  }
}

/// A voice that turns into a hard "s" for its middle fifth: noise above
/// 5 kHz far louder than the voice beneath.
fn sibilant_voice() -> Vec<f32> {
  let mut sound = voice(RATE as usize, 0.1);
  let hiss: Vec<f32> = noise(9_600, 0.2, 17)
    .windows(2)
    .map(|pair| (pair[1] - pair[0]) / 2.0)
    .collect();
  add(&mut sound, 19_200, &hiss);
  sound
}

#[test]
fn softens_a_sibilant_but_not_the_voice_before_it() {
  let sound = sibilant_voice();
  let speech = energy(&voice(RATE as usize, 0.1));
  let mut softened = sound.clone();
  Deesser::new(RATE, speech).apply(&mut softened);
  let middle = 21_000..27_000;
  let lowered = decibels(energy(&sound[middle.clone()]) / energy(&softened[middle]));
  assert!((2.0..=6.5).contains(&lowered), "lowered by {lowered} dB");
  assert_eq!(softened[..18_000], sound[..18_000]);
}

#[test]
fn leaves_hiss_between_words_alone() {
  let hiss: Vec<f32> = noise(RATE as usize, 0.002, 19)
    .windows(2)
    .map(|pair| pair[1] - pair[0])
    .collect();
  let speech = energy(&voice(RATE as usize, 0.1));
  let mut softened = hiss.clone();
  Deesser::new(RATE, speech).apply(&mut softened);
  assert_eq!(softened, hiss);
}

fn tone(length: usize, amplitude: f32) -> Vec<f32> {
  (0..length)
    .map(|at| amplitude * (TAU * 220.0 * at as f32 / RATE as f32).sin())
    .collect()
}

#[test]
fn finds_the_speech_level_from_the_speech_alone() {
  let mut sound = tone(RATE as usize, 0.1);
  sound.extend(vec![0.0; RATE as usize * 3]);
  sound.extend(tone(RATE as usize, 0.1));
  let mut frames = Frames::new(RATE);
  frames.add(&sound);
  let energies = frames.energies();
  let speaking = level::speaking(&energies).expect("there is speech");
  assert_eq!(speaking.iter().filter(|&&speaking| speaking).count(), 40);
  let speech = level::speech_energy(&energies, &speaking);
  assert!((decibels(speech / energy(&tone(RATE as usize, 0.1)))).abs() < 0.1);
}

#[test]
fn a_silent_track_has_no_speech_level() {
  let mut frames = Frames::new(RATE);
  frames.add(&vec![0.0; RATE as usize * 2]);
  assert!(level::speaking(&frames.energies()).is_none());
}
