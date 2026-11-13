// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How loud a clip plays by default: the stretch of its sound it plays,
//! brought a set distance under the voice as the recording is heard, a song
//! further under than a sound effect. Only the sound as a whole is measured
//! to the standard (LUFS); a stretch of it is put against that by the levels
//! kept for its waveform, ten a second, which is rough but follows a quiet
//! intro or a loud chorus.

use crate::clip::{AudioClipKind, RecordingAudioClip};

use super::SoundFile;

/// The loudness a clip is brought to with no voice to play under, in LUFS:
/// the loudness voices are commonly published at.
const NO_VOICE_LUFS: f64 = -16.0;
/// How far under the voice a song plays by default, in dB. Studio ducking
/// takes it further down while the voice speaks.
const SONG_UNDER_VOICE_DB: f64 = 6.0;
/// How far under the voice a sound effect plays by default, in dB.
const EFFECT_UNDER_VOICE_DB: f64 = 3.0;
/// The most a quiet sound is lifted to reach its level, in dB.
const MOST_LIFT_DB: f64 = 20.0;
/// The most a stretch is taken to be louder or quieter than its sound as a
/// whole, in dB: its levels are rough.
const MOST_STRETCH_DB: f64 = 12.0;
/// How many levels a sound keeps a second.
const LEVELS_PER_SECOND: u64 = 10;

/// How much a sound measured at `loudness_lufs` is lifted or lowered to play
/// at its kind's level: a song further under the voice, heard at
/// `voice_lufs`, than a sound effect, and either at a published voice's
/// loudness where the recording has no microphone to play under.
pub fn default_gain_db(
  kind: AudioClipKind,
  loudness_lufs: Option<f64>,
  voice_lufs: Option<f64>,
) -> f64 {
  let Some(loudness) = loudness_lufs else {
    return 0.0;
  };
  let target = match (voice_lufs, kind) {
    (None, _) => NO_VOICE_LUFS,
    (Some(voice), AudioClipKind::Music) => voice - SONG_UNDER_VOICE_DB,
    (Some(voice), AudioClipKind::Effect) => voice - EFFECT_UNDER_VOICE_DB,
  };
  (target - loudness).min(MOST_LIFT_DB)
}

/// How loud the stretch of its sound `clip` plays is, in LUFS. A song is the
/// sound's loudness over its whole length, moved by how much louder or
/// quieter its levels are there than along the whole sound; a sound effect
/// is heard at its peak, so it is the sound's loudest moment, moved by how
/// much quieter the loudest level of the stretch is than the sound's.
pub(super) fn played_loudness(clip: &RecordingAudioClip, file: &SoundFile) -> Option<f64> {
  let is_song = clip.kind == AudioClipKind::Music;
  let loudness = if is_song {
    file.loudness_lufs
  } else {
    file.loudest_lufs
  }?;
  let offset = clip.offset_ms.min(file.duration_ms);
  let length = clip.end_ms.saturating_sub(clip.start_ms);
  // A looped clip that comes round again plays all of its sound.
  let (from_ms, to_ms) = if clip.is_looped && offset + length > file.duration_ms {
    (0, file.duration_ms)
  } else {
    (offset, (offset + length).min(file.duration_ms))
  };
  let power = |levels: &[f32]| {
    let powers = levels.iter().map(|level| f64::from(*level).powi(2));
    #[expect(clippy::cast_precision_loss, reason = "a count of levels")]
    let count = levels.len() as f64;
    let power = if is_song {
      powers.sum::<f64>() / count
    } else {
      powers.fold(0.0, f64::max)
    };
    (count > 0.0 && power > 0.0).then_some(power)
  };
  let at = |ms: u64| usize::try_from(ms * LEVELS_PER_SECOND / 1_000).unwrap_or(usize::MAX);
  let levels = &file.levels;
  let stretch = levels.get(at(from_ms).min(levels.len())..at(to_ms).min(levels.len()));
  let shift = stretch
    .and_then(power)
    .zip(power(levels))
    .map_or(0.0, |(stretch, whole)| {
      (10.0 * (stretch / whole).log10()).clamp(-MOST_STRETCH_DB, MOST_STRETCH_DB)
    });
  Some(loudness + shift)
}
