// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! An audio clip: a stretch of a sound from the library, laid on the
//! timeline. It is timed on the timeline rather than in the recording, so
//! cutting, reordering or speeding up the footage neither moves, slices nor
//! speeds it.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::store::is_name;

/// A clip this long or longer is a song; a shorter one a sound effect.
pub const SONG_MS: u64 = 10_000;
/// The quietest and loudest a clip can be set, in dB from the level it
/// plays at by default.
pub const VOLUME_DB: std::ops::RangeInclusive<i16> = -60..=12;
/// The most clips a timeline holds.
const MAX_CLIPS: usize = 2_000;

/// What a clip is, which sets the level it plays at and how it meets its
/// neighbours: a song, or a sound effect.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum AudioClipKind {
  Music,
  Effect,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RecordingAudioClip {
  pub id: String,
  /// The sound it plays, by its name in the library.
  pub sound: String,
  pub kind: AudioClipKind,
  /// Its lane in the stack the strip draws, 0 at the top. Clips may
  /// overlap, in lanes of their own.
  pub lane: u32,
  /// Where it starts and ends on the timeline, in milliseconds of the edited
  /// video. It may end past the timeline, which cuts it off where the
  /// timeline ends and gives the rest back if the timeline grows again.
  pub start_ms: u64,
  pub end_ms: u64,
  /// How far into the sound it starts, in milliseconds.
  pub offset_ms: u64,
  /// How much louder or quieter than its default level it plays, in dB.
  pub volume_db: i16,
  /// Whether it repeats until its end rather than stopping where the sound
  /// ends.
  pub is_looped: bool,
  /// Whether a song plays without its singing, once that is separated.
  /// Songs always make way for the voice; sound effects never do.
  #[serde(default)]
  pub is_instrumental: bool,
}

/// Every clip has an id of its own, a sound named as the library names
/// sounds, a stretch, a volume in range and a lane in the stack, and no
/// two songs play at once.
pub fn validate_clips(clips: &[RecordingAudioClip]) -> Result<(), String> {
  if clips.len() > MAX_CLIPS {
    return Err("The timeline holds too many audio clips".to_owned());
  }
  let mut ids = std::collections::HashSet::with_capacity(clips.len());
  for clip in clips {
    if clip.id.is_empty()
      || !ids.insert(clip.id.as_str())
      || !is_name(&clip.sound)
      || clip.end_ms <= clip.start_ms
      || !VOLUME_DB.contains(&clip.volume_db)
      || clip.lane as usize >= MAX_CLIPS
    {
      return Err("The audio clips are invalid".to_owned());
    }
  }
  let mut songs: Vec<(u64, u64)> = clips
    .iter()
    .filter(|clip| clip.kind == AudioClipKind::Music)
    .map(|clip| (clip.start_ms, clip.end_ms))
    .collect();
  songs.sort_unstable();
  if songs.windows(2).any(|pair| pair[1].0 < pair[0].1) {
    return Err("Two songs play at once".to_owned());
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  fn clip(id: &str, start_ms: u64, end_ms: u64) -> RecordingAudioClip {
    RecordingAudioClip {
      id: id.to_owned(),
      sound: "0123456789abcdef0123456789abcdef.mp3".to_owned(),
      kind: AudioClipKind::Music,
      lane: 0,
      start_ms,
      end_ms,
      offset_ms: 0,
      volume_db: 0,
      is_looped: false,
      is_instrumental: false,
    }
  }

  #[test]
  fn two_clips_with_one_id_are_refused() {
    let effect = |id: &str, start_ms, end_ms| RecordingAudioClip {
      kind: AudioClipKind::Effect,
      ..clip(id, start_ms, end_ms)
    };
    assert!(validate_clips(&[effect("a", 0, 1_000), effect("b", 500, 2_000)]).is_ok());
    assert!(validate_clips(&[effect("a", 0, 1_000), effect("a", 500, 2_000)]).is_err());
  }

  #[test]
  fn songs_may_touch_but_never_overlap() {
    assert!(validate_clips(&[clip("a", 0, 1_000), clip("b", 1_000, 2_000)]).is_ok());
    assert!(validate_clips(&[clip("a", 0, 1_001), clip("b", 1_000, 2_000)]).is_err());
  }

  #[test]
  fn a_clip_naming_no_library_sound_is_refused() {
    let named_outside = RecordingAudioClip {
      sound: "../../secret.mp3".to_owned(),
      ..clip("a", 0, 1_000)
    };
    assert!(validate_clips(&[named_outside]).is_err());
  }
}
