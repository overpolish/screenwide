// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where a voice speaks alone, no other voice over it: what its fingerprint
//! is taken from, and the clip of it the notes keep so whoever took them can
//! hear who a speaker is and name them, after which the clip is deleted.

use screenwide_transcriber::{SpeakerTurn, VoiceClip};

/// Samples per millisecond.
pub const RATE: u64 = 16;
/// The least stretch worth hearing a voice from, in milliseconds.
const SHORTEST_MS: u64 = 1_000;
/// How long a voice's clip is at most, in milliseconds.
const CLIP_MS: u64 = 5_000;
/// How far into a stretch its clip starts when there is room, in
/// milliseconds, past where the speaker model may still hear the voice
/// before.
const CLIP_LEAD_MS: u64 = 250;
/// How long a clip fades in and out, in samples, so it starts and stops
/// without a click.
const FADE: usize = 160;

/// Where `speaker` speaks alone in `turns`, the longest stretch first, each
/// at least a second.
pub fn stretches(turns: &[SpeakerTurn], speaker: u32) -> Vec<(u64, u64)> {
  let others: Vec<(u64, u64)> = turns
    .iter()
    .filter(|other| other.speaker != speaker)
    .map(|other| (other.start_ms, other.end_ms))
    .collect();
  let mut alone: Vec<(u64, u64)> = turns
    .iter()
    .filter(|turn| turn.speaker == speaker)
    .flat_map(|turn| without((turn.start_ms, turn.end_ms), &others))
    .filter(|(start, end)| end - start >= SHORTEST_MS)
    .collect();
  alone.sort_by_key(|(start, end)| std::cmp::Reverse(end - start));
  alone
}

/// A clip of each voice in `turns` from its longest stretch alone in
/// `audio`, none for a voice never alone for a second.
pub fn clips(audio: &[i16], turns: &[SpeakerTurn]) -> Vec<VoiceClip> {
  let mut speakers: Vec<u32> = turns.iter().map(|turn| turn.speaker).collect();
  speakers.sort_unstable();
  speakers.dedup();
  speakers
    .into_iter()
    .filter_map(|speaker| {
      let &(start, end) = stretches(turns, speaker).first()?;
      let start = if end - start >= CLIP_MS + CLIP_LEAD_MS {
        start + CLIP_LEAD_MS
      } else {
        start
      };
      let end = end.min(start + CLIP_MS);
      let from = (start * RATE) as usize;
      let to = ((end * RATE) as usize).min(audio.len());
      let mut samples = audio.get(from..to)?.to_vec();
      faded(&mut samples);
      Some(VoiceClip { speaker, samples })
    })
    .collect()
}

fn faded(samples: &mut [i16]) {
  let fade = FADE.min(samples.len() / 2);
  let length = samples.len();
  for i in 0..fade {
    let gain = i as f32 / fade as f32;
    for at in [i, length - 1 - i] {
      samples[at] = (f32::from(samples[at]) * gain) as i16;
    }
  }
}

/// `span` with the `others` taken out of it.
fn without(span: (u64, u64), others: &[(u64, u64)]) -> Vec<(u64, u64)> {
  let mut pieces = vec![span];
  for &(start, end) in others {
    pieces = pieces
      .into_iter()
      .flat_map(|(a, b)| {
        if end <= a || start >= b {
          vec![(a, b)]
        } else {
          [(a, start.max(a)), (end.min(b), b)]
            .into_iter()
            .filter(|(x, y)| y > x)
            .collect()
        }
      })
      .collect();
  }
  pieces
}

#[cfg(test)]
mod tests {
  use super::*;

  fn turn(start_ms: u64, end_ms: u64, speaker: u32) -> SpeakerTurn {
    SpeakerTurn {
      start_ms,
      end_ms,
      speaker,
    }
  }

  #[test]
  fn another_voice_is_cut_out_of_a_stretch() {
    assert_eq!(
      without((0, 10_000), &[(2_000, 3_000), (9_000, 12_000)]),
      vec![(0, 2_000), (3_000, 9_000)]
    );
  }

  #[test]
  fn a_voice_is_clipped_where_it_speaks_longest_alone() {
    let turns = [
      turn(0, 3_000, 1),
      turn(3_000, 12_000, 2),
      turn(10_000, 20_000, 1),
      turn(20_000, 20_500, 3),
    ];
    let audio = vec![1_000i16; 21_000 * RATE as usize];
    let clips = clips(&audio, &turns);
    let lengths: Vec<(u32, usize)> = clips
      .iter()
      .map(|clip| (clip.speaker, clip.samples.len()))
      .collect();
    // Speaker 1 alone longest from 12 s to 20 s: five seconds from 12.25 s.
    // Speaker 2 alone from 3 s to 10 s. Speaker 3 never for a second.
    assert_eq!(
      lengths,
      vec![(1, CLIP_MS as usize * 16), (2, CLIP_MS as usize * 16)]
    );
    assert_eq!(clips[0].samples[0], 0);
    assert_eq!(clips[0].samples[FADE], 1_000);
  }
}
