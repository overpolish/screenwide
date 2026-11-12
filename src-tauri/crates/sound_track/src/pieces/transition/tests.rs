// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::{Path, PathBuf};

use crate::clip::{AudioClipKind, RecordingAudioClip};
use crate::pieces::{pieces, Piece, SoundFile, Tone};
use crate::store::music::{Beat, Music};
use crate::store::Separated;

/// The earlier song's beat: every 500 ms from 100 ms, 120 BPM.
const OURS: Beat = Beat {
  period_ms: 500.0,
  first_ms: 100.0,
};

fn file(name: &str, beat: Option<Beat>) -> SoundFile {
  SoundFile {
    path: PathBuf::from(name),
    duration_ms: 60_000,
    loudness_lufs: Some(-14.0),
    levels: Vec::new(),
    separated: None,
    music: Music { beat },
  }
}

fn song(sound: &str, start_ms: u64, end_ms: u64, offset_ms: u64) -> RecordingAudioClip {
  RecordingAudioClip {
    id: sound.to_owned(),
    sound: sound.to_owned(),
    kind: AudioClipKind::Music,
    lane: 0,
    start_ms,
    end_ms,
    offset_ms,
    volume_db: 0,
    is_looped: false,
    is_instrumental: false,
  }
}

/// Song `a`, its singing separated, from `earlier_from` up to a join at
/// 30 s, then song `b`, its beat `theirs`, from 10 s into it.
fn joined(earlier_from: u64, theirs: Option<Beat>, later_is_instrumental: bool) -> Vec<Piece> {
  let mut earlier = file("a", Some(OURS));
  earlier.separated = Some(Separated::with_pieces(Path::new("a"), 0..8));
  let mut second = song("b", 30_000, 60_000, 10_000);
  second.is_instrumental = later_is_instrumental;
  let clips = [song("a", earlier_from, 30_000, earlier_from), second];
  let files = [("a", earlier), ("b", file("b", theirs))];
  pieces(
    &clips,
    |name| {
      files
        .iter()
        .find(|(each, _)| *each == name)
        .map(|(_, file)| file)
    },
    60_000,
    (None, false),
  )
}

fn beat(period_ms: f64) -> Option<Beat> {
  Some(Beat {
    period_ms,
    first_ms: 30.0,
  })
}

fn of<'a>(pieces: &'a [Piece], name: &str, tone: Tone) -> Vec<&'a Piece> {
  pieces
    .iter()
    .filter(|piece| piece.path == Path::new(name) && piece.tone == tone)
    .collect()
}

/// Where in its file `piece` is at `at_ms` of the timeline.
fn file_at(piece: &Piece, at_ms: u64) -> f64 {
  piece.file_start_ms as f64 + (at_ms as f64 - piece.start_ms as f64) * piece.speed
}

/// How far `file_ms` is from the nearest beat of `beat`, in ms.
fn off_beat(beat: Beat, file_ms: f64) -> f64 {
  (beat.nearest(file_ms) - file_ms).abs()
}

#[test]
fn close_tempos_ramp_then_blend_band_by_band_on_the_beat() {
  // 133 BPM after 120: 11% apart.
  let theirs = beat(450.0).unwrap();
  let pieces = joined(0, Some(theirs), false);
  let speed = 500.0 / 450.0;
  // The earlier song ramps from its own speed to the later's tempo, a step
  // a beat, then plays its bands at it.
  let ramp: Vec<f64> = of(&pieces, "a", Tone::Whole)
    .iter()
    .map(|piece| piece.speed)
    .filter(|speed| *speed != 1.0)
    .collect();
  assert_eq!(ramp.len(), 8);
  assert!(ramp.windows(2).all(|pair| pair[0] < pair[1]));
  assert!(1.0 < ramp[0] && ramp[7] < speed);
  let [bass, mids, highs] =
    [Tone::Bass, Tone::Mids, Tone::Highs].map(|tone| of(&pieces, "a", tone)[0]);
  for band in [bass, mids, highs] {
    assert_eq!(band.speed, speed);
  }
  // The bass swaps on a beat of both songs, about the join.
  let swap = bass.end_ms - 450 / 2;
  assert!(swap.abs_diff(30_000) <= 450, "swap at {swap}");
  assert!(off_beat(OURS, file_at(bass, swap)) < 2.0);
  let [their_bass, their_mids, their_highs] =
    [Tone::Bass, Tone::Mids, Tone::Highs].map(|tone| of(&pieces, "b", tone)[0]);
  assert!(off_beat(theirs, file_at(their_bass, swap)) < 2.0);
  // The later song's highs come first and its bass last; the earlier's
  // bass goes first and its highs last.
  assert!(their_highs.start_ms < their_mids.start_ms && their_mids.start_ms < their_bass.start_ms);
  assert!(bass.end_ms < mids.end_ms && mids.end_ms < highs.end_ms);
  // Four beats each side of the swap, then the later song as it is, from
  // where its bands were.
  assert_eq!(highs.end_ms - swap, 1_800);
  let whole = of(&pieces, "b", Tone::Whole)[0];
  assert_eq!(whole.start_ms, highs.end_ms - 10);
  assert!((file_at(whole, highs.end_ms) - file_at(their_highs, highs.end_ms)).abs() <= 1.0);
}

#[test]
fn songs_at_one_tempo_blend_without_a_ramp() {
  let pieces = joined(0, beat(500.0), false);
  assert!(pieces.iter().all(|piece| piece.speed == 1.0));
  assert_eq!(of(&pieces, "b", Tone::Bass).len(), 1);
}

#[test]
fn tempos_too_far_apart_crossfade() {
  // 167 BPM after 120.
  let pieces = joined(0, beat(360.0), false);
  assert!(pieces
    .iter()
    .all(|piece| piece.tone == Tone::Whole && piece.speed == 1.0));
  assert_eq!(of(&pieces, "b", Tone::Whole)[0].start_ms, 28_500);
}

#[test]
fn a_short_song_blends_over_fewer_beats() {
  // Three seconds of the earlier song: room for a ramp of two of its beats
  // and two beats each side of the swap.
  let pieces = joined(27_000, beat(450.0), false);
  let highs = of(&pieces, "b", Tone::Highs)[0];
  assert_eq!(highs.fade_in.ms, 900);
  // One second: no room for any blend.
  let pieces = joined(29_000, beat(450.0), false);
  assert!(pieces.iter().all(|piece| piece.tone == Tone::Whole));
}

#[test]
fn the_earlier_songs_singing_goes_only_under_a_song_that_sings() {
  let sings = |later_is_instrumental| {
    let pieces = joined(0, beat(450.0), later_is_instrumental);
    of(&pieces, "a", Tone::Highs)[0].instrumental
  };
  assert!(sings(false));
  assert!(!sings(true));
}

#[test]
fn a_song_with_no_steady_beat_crossfades() {
  let pieces = joined(0, None, false);
  assert!(pieces.iter().all(|piece| piece.tone == Tone::Whole));
  assert_eq!(of(&pieces, "b", Tone::Whole)[0].start_ms, 28_500);
}
