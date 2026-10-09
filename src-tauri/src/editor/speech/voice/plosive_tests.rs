// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::f32::consts::{PI, TAU};

use super::deplosive::Deplosive;
use super::tests::{add, decibels, energy, noise, voice, RATE};
use super::tick::Ticks;

/// The breath of a "p" on the microphone: a push of air 30 ms long, and the
/// capsule ringing at 60 Hz as it settles.
fn thump(peak: f32) -> Vec<f32> {
  let push = RATE as usize * 30 / 1_000;
  (0..RATE as usize * 120 / 1_000)
    .map(|at| {
      let time = at as f32 / RATE as f32;
      let air = if at < push {
        (PI * at as f32 / push as f32).sin()
      } else {
        0.0
      };
      peak * (air + 0.6 * (TAU * 60.0 * time).sin() * (-time / 0.03).exp())
    })
    .collect()
}

/// A deep voice: 100 Hz and its harmonics, rising and falling as syllables
/// do.
fn deep_voice(length: usize, amplitude: f32) -> Vec<f32> {
  (0..length)
    .map(|at| {
      let time = at as f32 / RATE as f32;
      let syllables = 0.6 + 0.4 * (TAU * 4.0 * time).sin();
      let harmonics: f32 = (1..80)
        .map(|harmonic| (TAU * 100.0 * harmonic as f32 * time).sin() / harmonic as f32)
        .sum();
      amplitude * syllables * harmonics / 3.0
    })
    .collect()
}

/// What lies below 250 Hz in `samples`, roughly: a moving average 2 ms wide
/// keeps the thump and drops the room's hiss.
fn low(samples: &[f32]) -> Vec<f32> {
  let width = RATE as usize / 500;
  samples
    .windows(width)
    .map(|window| window.iter().sum::<f32>() / width as f32)
    .collect()
}

#[test]
fn softens_a_plosive_thump() {
  let clean = noise(RATE as usize, 0.001, 3);
  let mut popped = clean.clone();
  let pop = thump(0.3);
  add(&mut popped, 24_000, &pop);
  let mut softened = popped.clone();
  Deplosive::new(RATE).apply(&mut softened);
  let window = 24_000..24_000 + pop.len();
  let (popped, softened, clean) = (low(&popped), low(&softened), low(&clean));
  let left: Vec<f32> = window.clone().map(|at| softened[at] - clean[at]).collect();
  let added: Vec<f32> = window.map(|at| popped[at] - clean[at]).collect();
  let taken = decibels(energy(&added) / energy(&left));
  assert!(taken > 10.0, "taken down by {taken} dB");
}

#[test]
fn leaves_a_deep_voice_alone_from_its_first_word() {
  let mut sound = noise(RATE as usize * 2, 0.0001, 5);
  add(&mut sound, 30_000, &deep_voice(RATE as usize, 0.1));
  let mut softened = sound.clone();
  Deplosive::new(RATE).apply(&mut softened);
  assert_eq!(softened, sound);
}

fn peak(samples: &[f32]) -> f32 {
  samples
    .iter()
    .fold(0.0, |peak, sample| peak.max(sample.abs()))
}

/// A word starting out of quiet: a `release` loud and `release_ms` long,
/// then a vowel peaking near 0.2.
fn word_with_release(release: f32, release_ms: usize) -> Vec<f32> {
  let mut sound = noise(RATE as usize, 0.0001, 5);
  let length = RATE as usize * release_ms / 1_000;
  add(&mut sound, 20_000, &noise(length, release, 11));
  add(&mut sound, 20_000 + length, &voice(9_600, 0.2));
  sound
}

#[test]
fn takes_a_tick_down_to_its_word() {
  let sound = word_with_release(0.8, 1);
  let vowel = peak(&sound[20_400..23_000]);
  let mut softened = sound.clone();
  Ticks::new(RATE).apply(&mut softened);
  let tick = peak(&softened[20_000..20_048]);
  assert!(tick < vowel, "tick {tick} against vowel {vowel}");
  assert_eq!(softened[20_400..], sound[20_400..]);
}

/// A tick no louder than the vowel after it is still a click to the ear,
/// and Auto volume's lift makes it more of one.
#[test]
fn takes_a_tick_at_its_words_level_down() {
  let sound = word_with_release(0.2, 1);
  let vowel = peak(&sound[20_400..23_000]);
  let mut softened = sound.clone();
  Ticks::new(RATE).apply(&mut softened);
  let tick = peak(&softened[20_000..20_048]);
  assert!(tick < vowel * 0.5, "tick {tick} against vowel {vowel}");
}

/// A deep, pressed voice spikes above 2 kHz on every pulse of the vocal
/// cords, 40 dB over the level between them; the pulses stand out from
/// that level, but not from each other.
#[test]
fn leaves_a_deep_voices_pulses_alone() {
  let mut sound = noise(RATE as usize, 0.0001, 5);
  add(&mut sound, 20_000, &deep_voice(9_600, 0.2));
  let mut softened = sound.clone();
  Ticks::new(RATE).apply(&mut softened);
  assert_eq!(softened, sound);
}

/// The release of a "t": a hiss 15 ms long, at the vowel's level.
#[test]
fn leaves_a_consonant_release_alone() {
  let sound = word_with_release(0.12, 15);
  let mut softened = sound.clone();
  Ticks::new(RATE).apply(&mut softened);
  assert_eq!(softened, sound);
}

/// The crack of a "k" out of its closure rings past a click, loud against
/// the vowel after it; it is softened by 12 dB rather than taken out, so the
/// consonant is still heard.
#[test]
fn softens_a_consonants_crack_without_taking_it_out() {
  let sound = word_with_release(0.8, 10);
  let mut softened = sound.clone();
  Ticks::new(RATE).apply(&mut softened);
  let crack = 20_000..20_480;
  let cut = 20.0 * (peak(&sound[crack.clone()]) / peak(&softened[crack])).log10();
  assert!((cut - 12.0).abs() < 1.5, "{cut} dB");
}

/// Reduce noise leaves a pause all but silent; what it leaves there is
/// nothing to hear, however it stands out from the rest.
#[test]
fn leaves_a_tick_in_the_noise_alone() {
  let mut sound = noise(RATE as usize, 0.000_001, 5);
  add(&mut sound, 20_000, &noise(48, 0.000_3, 11));
  let mut softened = sound.clone();
  Ticks::new(RATE).apply(&mut softened);
  assert_eq!(softened, sound);
}
