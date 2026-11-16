// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The call played through the speakers taken out of the microphone. Its
//! voices reach the microphone quieter than the computer played them, a few
//! frames late, and the voice activity model takes them for speech, so a
//! stretch of the microphone runs on through the others' turns and its text
//! carries their words. Each frame the microphone heard while the computer
//! played louder than it is silenced before the stretch is written, as a
//! speakerphone hands the line to whoever is louder. The note taker's own
//! voice, close to the microphone, is louder than the call reaching it.

/// The voice activity model's frame, 32 ms: what is silenced or kept.
const FRAME: usize = 512;
/// How late the call may reach the microphone, in frames: 128 ms, room for
/// the sound's way through the air and the system's buffers.
const LATE_FRAMES: usize = 4;
/// The computer's audio counts as playing above this level, about -46 dBFS.
const PLAYING: f32 = 0.005;
/// A microphone frame under this share of the computer's level meanwhile is
/// the call reaching it.
const BLEED: f32 = 0.5;
/// The least of the note taker's own speech worth writing, 288 ms: under it a
/// stretch is the call's echo with a word or two of breath.
const LEAST_KEPT_FRAMES: usize = 9;

/// Silences the frames of `microphone`, starting at sample `from` of the
/// notes, where `system`, the computer's audio from the notes' start, played
/// louder than the microphone heard. False when too little of the
/// microphone is left to be worth writing.
pub fn without_bleed(microphone: &mut [f32], system: &[i16], from: usize) -> bool {
  let mut kept = 0;
  for (index, frame) in microphone.chunks_mut(FRAME).enumerate() {
    let at = from + index * FRAME;
    let first = at.saturating_sub(LATE_FRAMES * FRAME);
    let end = (at + FRAME).min(system.len());
    let played = system
      .get(first.min(end)..end)
      .unwrap_or_default()
      .chunks(FRAME)
      .map(|frame| rms(frame.iter().map(|&sample| f32::from(sample) / 32_768.0)))
      .fold(0.0, f32::max);
    let heard = rms(frame.iter().copied());
    if played > PLAYING && heard < BLEED * played {
      frame.fill(0.0);
    } else if heard > PLAYING {
      kept += 1;
    }
  }
  kept >= LEAST_KEPT_FRAMES
}

fn rms(samples: impl Iterator<Item = f32>) -> f32 {
  let (sum, count) = samples.fold((0.0, 0_usize), |(sum, count), sample| {
    (sum + sample * sample, count + 1)
  });
  if count == 0 {
    0.0
  } else {
    (sum / count as f32).sqrt()
  }
}

#[cfg(test)]
mod tests {
  use super::{without_bleed, FRAME};

  fn tone(level: f32, frames: usize) -> Vec<f32> {
    (0..frames * FRAME)
      .map(|at| level * (at as f32 * 0.3).sin())
      .collect()
  }

  #[test]
  fn the_call_reaching_the_microphone_is_silenced() {
    let system: Vec<i16> = tone(0.2, 40)
      .iter()
      .map(|&s| (s * 32_767.0) as i16)
      .collect();
    // The call at a quarter of its level, the note taker silent.
    let mut microphone = tone(0.05, 40);
    assert!(!without_bleed(&mut microphone, &system, 0));
    assert!(microphone.iter().all(|&sample| sample == 0.0));
  }

  #[test]
  fn the_note_takers_own_voice_is_kept() {
    let system = vec![0_i16; 40 * FRAME];
    let mut microphone = tone(0.1, 40);
    assert!(without_bleed(&mut microphone, &system, 0));
    assert!(microphone.iter().any(|&sample| sample != 0.0));
    // Louder than the call, over it, it is kept too.
    let system: Vec<i16> = tone(0.05, 40)
      .iter()
      .map(|&s| (s * 32_767.0) as i16)
      .collect();
    let mut microphone = tone(0.2, 40);
    assert!(without_bleed(&mut microphone, &system, 0));
  }
}
