// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The clips laid out as pieces: each a stretch of one sound file placed on
//! the timeline, with the level and fades it plays at. A clip is one piece,
//! or one for each time it repeats when it loops, each repeat crossfading
//! into the next. Clips have no fades of their own to set: a song fades in
//! where it starts partway into its sound and out where it stops short of
//! its end, and a sound effect cut short fades out quickly; every edge
//! fades a little, so none clicks. Songs never overlap; those that touch
//! crossfade into each other on their own: the crossfade is centred on the
//! join where the files hold sound past it, and happens inside the clip on
//! a side whose file ends there. Nothing is heard past the timeline's end.

mod piece;

pub use piece::{Curve, Fade, Piece, SoundFile};

use crate::clip::{AudioClipKind, RecordingAudioClip};

/// How long songs that touch crossfade for, at most, in ms.
pub const CROSSFADE_MS: u64 = 3_000;
/// How long a looped clip's repeats crossfade for, at most, in ms.
pub const LOOP_CROSSFADE_MS: u64 = 500;
/// The loudness voices are brought to by Auto volume, in LUFS.
const VOICE_LUFS: f64 = -16.0;
/// How far under the voice a song plays by default, in dB.
const SONG_UNDER_VOICE_DB: f64 = 14.0;
/// How far under the voice a sound effect plays by default, in dB.
const EFFECT_UNDER_VOICE_DB: f64 = 6.0;
/// The most a quiet sound is lifted to reach its level, in dB.
const MOST_LIFT_DB: f64 = 20.0;
/// How far apart, in ms, a song's end and the next one's start may be and
/// still touch: the editor works in whole milliseconds.
const TOUCHING_MS: u64 = 1;
/// The most repeats a looped clip plays, so a tiny sound looped over a long
/// timeline stays a graph FFmpeg can run.
const MOST_REPEATS: usize = 2_000;
/// How long a song that starts partway into its sound fades in, in ms.
const SONG_FADE_IN_MS: u64 = 1_000;
/// How long a song that stops short of its sound's end fades out, in ms.
const SONG_FADE_OUT_MS: u64 = 2_000;
/// How long a sound effect that stops short of its sound's end fades out,
/// in ms.
const EFFECT_FADE_OUT_MS: u64 = 100;
/// How long every clip's edges fade at least, in ms, so none clicks.
const EDGE_FADE_MS: u64 = 5;

/// How much a sound measured at `loudness_lufs` is lifted or lowered to play
/// at its kind's level: a song well under the voice, a sound effect a little
/// under it, and either at the voice's own level where the recording has no
/// microphone to play under.
pub fn default_gain_db(kind: AudioClipKind, loudness_lufs: Option<f64>, has_voice: bool) -> f64 {
  let Some(loudness) = loudness_lufs else {
    return 0.0;
  };
  let under = match (has_voice, kind) {
    (false, _) => 0.0,
    (true, AudioClipKind::Music) => SONG_UNDER_VOICE_DB,
    (true, AudioClipKind::Effect) => EFFECT_UNDER_VOICE_DB,
  };
  (VOICE_LUFS - under - loudness).min(MOST_LIFT_DB)
}

/// A clip as it is heard: where it plays on the timeline and its pieces.
struct Laid<'a> {
  clip: &'a RecordingAudioClip,
  file: &'a SoundFile,
  start_ms: u64,
  end_ms: u64,
  pieces: Vec<Piece>,
}

/// The pieces `clips` play on a timeline `timeline_ms` long, their sounds
/// found by `files`. A clip whose sound is missing is silent.
pub fn pieces<'a>(
  clips: &'a [RecordingAudioClip],
  files: impl Fn(&str) -> Option<&'a SoundFile>,
  timeline_ms: u64,
  has_voice: bool,
) -> Vec<Piece> {
  let mut laid: Vec<Laid<'a>> = clips
    .iter()
    .filter_map(|clip| {
      let file = files(&clip.sound).filter(|file| file.duration_ms > 0)?;
      let end_ms = clip.end_ms.min(timeline_ms);
      if end_ms <= clip.start_ms {
        return None;
      }
      let gain_db =
        default_gain_db(clip.kind, file.loudness_lufs, has_voice) + f64::from(clip.volume_db);
      let mut pieces = if clip.is_looped {
        repeats(clip, file, end_ms, gain_db)
      } else {
        once(clip, file, end_ms, gain_db)
      };
      let (fade_in, fade_out) = edge_fades(clip, file, &pieces);
      if let Some(first) = pieces.first_mut() {
        first.fade_in = fade_in;
      }
      if let Some(last) = pieces.last_mut() {
        last.fade_out = fade_out;
      }
      (!pieces.is_empty()).then_some(Laid {
        clip,
        file,
        start_ms: clip.start_ms,
        end_ms,
        pieces,
      })
    })
    .collect();
  crossfade_songs(&mut laid);
  laid
    .into_iter()
    .flat_map(|laid| laid.pieces)
    .map(|mut piece| {
      piece.fit_fades();
      piece
    })
    .collect()
}

/// How a clip's first piece fades in and its last fades out: a song fades
/// in where it starts partway into its sound, and either kind fades out
/// where it stops short of its sound's end; elsewhere only enough that the
/// edge does not click.
fn edge_fades(clip: &RecordingAudioClip, file: &SoundFile, pieces: &[Piece]) -> (Fade, Fade) {
  let is_song = clip.kind == AudioClipKind::Music;
  let starts_inside = pieces
    .first()
    .is_some_and(|piece| piece.file_start_ms > EDGE_FADE_MS);
  let stops_short = pieces
    .last()
    .is_some_and(|piece| piece.file_start_ms + piece.length_ms() + EDGE_FADE_MS < file.duration_ms);
  let fade_in = if is_song && starts_inside {
    SONG_FADE_IN_MS
  } else {
    EDGE_FADE_MS
  };
  let fade_out = match (stops_short, is_song) {
    (true, true) => SONG_FADE_OUT_MS,
    (true, false) => EFFECT_FADE_OUT_MS,
    (false, _) => EDGE_FADE_MS,
  };
  (Fade::linear(fade_in), Fade::linear(fade_out))
}
/// A clip that plays its sound once, from its offset, until it ends or the
/// sound does.
fn once(clip: &RecordingAudioClip, file: &SoundFile, end_ms: u64, gain_db: f64) -> Vec<Piece> {
  let offset = clip.offset_ms.min(file.duration_ms);
  let end_ms = end_ms.min(clip.start_ms + (file.duration_ms - offset));
  if end_ms <= clip.start_ms {
    return Vec::new();
  }
  vec![Piece {
    path: file.path.clone(),
    file_start_ms: offset,
    start_ms: clip.start_ms,
    end_ms,
    fade_in: Fade::NONE,
    fade_out: Fade::NONE,
    gain_db,
    makes_way: clip.makes_way,
    singing: file.singing.clone(),
  }]
}

/// A looped clip: its sound from its offset, then again from the start each
/// time it ends, each repeat crossfading into the next, until the clip ends.
fn repeats(clip: &RecordingAudioClip, file: &SoundFile, end_ms: u64, gain_db: f64) -> Vec<Piece> {
  let length = file.duration_ms;
  // A sound too short to crossfade into itself for long repeats with a
  // crossfade a quarter of its length.
  let crossfade = LOOP_CROSSFADE_MS.min(length / 4);
  let mut pieces: Vec<Piece> = Vec::new();
  let mut start_ms = clip.start_ms;
  let mut file_start_ms = clip.offset_ms % length;
  while pieces.len() < MOST_REPEATS {
    let piece_end = end_ms.min(start_ms + (length - file_start_ms));
    if piece_end <= start_ms {
      break;
    }
    let fade_in = if pieces.is_empty() {
      Fade::NONE
    } else {
      Fade::equal_power(crossfade)
    };
    pieces.push(Piece {
      path: file.path.clone(),
      file_start_ms,
      start_ms,
      end_ms: piece_end,
      fade_in,
      fade_out: Fade::NONE,
      gain_db,
      makes_way: clip.makes_way,
      singing: file.singing.clone(),
    });
    if piece_end >= end_ms {
      break;
    }
    if let Some(last) = pieces.last_mut() {
      last.fade_out = Fade::equal_power(crossfade);
    }
    start_ms = piece_end - crossfade;
    file_start_ms = 0;
  }
  pieces
}

/// Songs that touch crossfade: the earlier one plays on past the join and
/// the later one starts before it, each by as much as its file holds, up to
/// half the crossfade; a side that holds less leaves the rest to the other,
/// and where neither holds enough the crossfade happens inside the clips.
/// The crossfade takes the place of the two clips' own fades at the join.
fn crossfade_songs(laid: &mut [Laid<'_>]) {
  let mut songs: Vec<usize> = (0..laid.len())
    .filter(|&index| laid[index].clip.kind == AudioClipKind::Music)
    .collect();
  songs.sort_by_key(|&index| laid[index].start_ms);
  for pair in songs.windows(2) {
    let (earlier, later) = (pair[0], pair[1]);
    let join = laid[later].start_ms;
    let heard_to = laid[earlier].pieces.last().map_or(0, |piece| piece.end_ms);
    if laid[earlier].end_ms.abs_diff(join) > TOUCHING_MS || heard_to != laid[earlier].end_ms {
      continue;
    }
    let length = |laid: &Laid<'_>| laid.end_ms - laid.start_ms;
    let crossfade = CROSSFADE_MS
      .min(length(&laid[earlier]) / 2)
      .min(length(&laid[later]) / 2);
    let after = laid[earlier].pieces.last().map_or(0, |piece| {
      laid[earlier]
        .file
        .duration_ms
        .saturating_sub(piece.file_start_ms + piece.length_ms())
    });
    let before = laid[later]
      .pieces
      .first()
      .map_or(0, |piece| piece.file_start_ms);
    let mut early = (crossfade / 2).min(before);
    let late = (crossfade - early).min(after);
    early = (crossfade - late).min(before);
    if let Some(last) = laid[earlier].pieces.last_mut() {
      last.end_ms = join + late;
      last.fade_out = Fade::equal_power(crossfade);
    }
    if let Some(first) = laid[later].pieces.first_mut() {
      first.start_ms = join - early;
      first.file_start_ms -= early;
      first.fade_in = Fade::equal_power(crossfade);
    }
  }
}

#[cfg(test)]
mod tests;
