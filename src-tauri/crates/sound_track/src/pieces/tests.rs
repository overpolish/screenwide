// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::{Path, PathBuf};

use super::*;
use crate::store::Separated;

fn file(name: &str, duration_ms: u64) -> SoundFile {
  SoundFile {
    path: PathBuf::from(name),
    duration_ms,
    loudness_lufs: Some(-14.0),
    levels: Vec::new(),
    separated: None,
  }
}

fn song(id: &str, sound: &str, start_ms: u64, end_ms: u64) -> RecordingAudioClip {
  RecordingAudioClip {
    id: id.to_owned(),
    sound: sound.to_owned(),
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

fn laid_out(
  clips: &[RecordingAudioClip],
  files: &[(&str, SoundFile)],
  timeline_ms: u64,
) -> Vec<Piece> {
  pieces(
    clips,
    |name| {
      files
        .iter()
        .find(|(each, _)| *each == name)
        .map(|(_, file)| file)
    },
    timeline_ms,
    (Some(-16.0), true),
  )
}

#[test]
fn songs_make_way_only_while_ducking_and_sound_effects_never() {
  let effect = RecordingAudioClip {
    kind: AudioClipKind::Effect,
    ..song("b", "click", 0, 1_000)
  };
  let clips = [song("a", "song", 0, 10_000), effect];
  let files = [
    ("song", file("song", 60_000)),
    ("click", file("click", 1_000)),
  ];
  let making_way = |ducks: bool| -> Vec<bool> {
    pieces(
      &clips,
      |name| {
        files
          .iter()
          .find(|(each, _)| *each == name)
          .map(|(_, file)| file)
      },
      60_000,
      (Some(-16.0), ducks),
    )
    .iter()
    .map(|piece| piece.makes_way)
    .collect()
  };
  assert_eq!(making_way(true), [true, false]);
  assert_eq!(making_way(false), [false, false]);
}

#[test]
fn a_piece_reads_the_parts_only_where_all_it_plays_is_separated() {
  let clip = song("a", "song", 0, 10_000);
  let separated = |pieces: &[u64]| {
    let mut song = file("song.mp3", 60_000);
    song.separated = Some(Separated::with_pieces(
      Path::new("song.mp3"),
      pieces.iter().copied(),
    ));
    laid_out(std::slice::from_ref(&clip), &[("song", song)], 60_000)
  };
  // It plays the first 10 s: the first two 8 s pieces.
  assert_eq!(separated(&[0])[0].parts, None);
  assert_eq!(separated(&[1, 2])[0].parts, None);
  assert_eq!(
    separated(&[0, 1])[0].parts,
    Some(Parts {
      singing: PathBuf::from("song.singing.flac"),
      rest: PathBuf::from("song.rest.flac"),
    })
  );
}

#[test]
fn songs_whose_files_end_at_the_join_crossfade_inside_their_clips() {
  let files = [("a", file("a", 60_000)), ("b", file("b", 60_000))];
  let clips = [song("1", "a", 0, 60_000), song("2", "b", 60_000, 120_000)];
  let pieces = laid_out(&clips, &files, 120_000);
  assert_eq!(
    (pieces[0].end_ms, pieces[0].fade_out),
    (60_000, Fade::equal_power(3_000))
  );
  assert_eq!((pieces[1].start_ms, pieces[1].file_start_ms), (60_000, 0));
  assert_eq!(pieces[1].fade_in, Fade::equal_power(3_000));
}

#[test]
fn songs_trimmed_at_the_join_crossfade_centred_on_it() {
  let files = [("a", file("a", 90_000)), ("b", file("b", 90_000))];
  let mut later = song("2", "b", 60_000, 120_000);
  later.offset_ms = 20_000;
  let clips = [song("1", "a", 0, 60_000), later];
  let pieces = laid_out(&clips, &files, 120_000);
  assert_eq!(pieces[0].end_ms, 61_500);
  assert_eq!(
    (pieces[1].start_ms, pieces[1].file_start_ms),
    (58_500, 18_500)
  );
}

#[test]
fn a_crossfade_takes_what_one_side_lacks_from_the_other() {
  let files = [("a", file("a", 60_000)), ("b", file("b", 90_000))];
  let mut later = song("2", "b", 60_000, 120_000);
  later.offset_ms = 20_000;
  let clips = [song("1", "a", 0, 60_000), later];
  let pieces = laid_out(&clips, &files, 120_000);
  assert_eq!(pieces[0].end_ms, 60_000);
  assert_eq!(
    (pieces[1].start_ms, pieces[1].file_start_ms),
    (57_000, 17_000)
  );
}

#[test]
fn a_crossfade_is_never_longer_than_half_the_shorter_song() {
  let files = [("a", file("a", 60_000)), ("b", file("b", 60_000))];
  let clips = [song("1", "a", 0, 60_000), song("2", "b", 60_000, 64_000)];
  let pieces = laid_out(&clips, &files, 120_000);
  assert_eq!(pieces[1].fade_in, Fade::equal_power(2_000));
}

#[test]
fn songs_with_a_gap_between_fade_only_where_they_stop_short() {
  let files = [("a", file("a", 60_000)), ("b", file("b", 60_000))];
  let clips = [song("1", "a", 0, 30_000), song("2", "b", 30_500, 90_500)];
  let pieces = laid_out(&clips, &files, 120_000);
  assert_eq!(pieces[0].fade_out, Fade::linear(SONG_FADE_OUT_MS));
  // Played from its start, the second song needs no fade in, and played to
  // its end, no fade out: only enough that neither edge clicks.
  assert_eq!(pieces[1].fade_in, Fade::linear(EDGE_FADE_MS));
  assert_eq!(pieces[1].fade_out, Fade::linear(EDGE_FADE_MS));
}

#[test]
fn a_song_started_partway_into_its_sound_fades_in() {
  let files = [("a", file("a", 60_000))];
  let mut clip = song("1", "a", 0, 20_000);
  clip.offset_ms = 10_000;
  let pieces = laid_out(&[clip], &files, 60_000);
  assert_eq!(pieces[0].fade_in, Fade::linear(SONG_FADE_IN_MS));
}

#[test]
fn a_sound_effect_cut_short_fades_out_quickly() {
  let files = [("a", file("a", 3_000))];
  let mut clip = song("1", "a", 0, 1_000);
  clip.kind = AudioClipKind::Effect;
  clip.offset_ms = 500;
  let pieces = laid_out(&[clip], &files, 60_000);
  assert_eq!(
    (pieces[0].fade_in, pieces[0].fade_out),
    (Fade::linear(EDGE_FADE_MS), Fade::linear(EFFECT_FADE_OUT_MS))
  );
}

#[test]
fn a_clip_past_the_timeline_is_cut_off_and_fades_out_where_it_ends() {
  let files = [("a", file("a", 60_000))];
  let pieces = laid_out(&[song("1", "a", 10_000, 60_000)], &files, 30_000);
  assert_eq!(
    (pieces[0].end_ms, pieces[0].fade_out),
    (30_000, Fade::linear(SONG_FADE_OUT_MS))
  );
  assert!(laid_out(&[song("1", "a", 30_000, 40_000)], &files, 30_000).is_empty());
}

#[test]
fn a_clip_that_does_not_loop_stops_where_its_sound_ends() {
  let files = [("a", file("a", 5_000))];
  let mut clip = song("1", "a", 0, 20_000);
  clip.offset_ms = 1_000;
  let pieces = laid_out(&[clip], &files, 60_000);
  assert_eq!(pieces.len(), 1);
  assert_eq!(pieces[0].end_ms, 4_000);
}

#[test]
fn a_looped_clip_repeats_with_a_short_crossfade_until_it_ends() {
  let files = [("a", file("a", 4_000))];
  let mut clip = song("1", "a", 0, 10_000);
  clip.is_looped = true;
  let pieces = laid_out(&[clip], &files, 60_000);
  let spans: Vec<_> = pieces
    .iter()
    .map(|piece| (piece.start_ms, piece.end_ms, piece.file_start_ms))
    .collect();
  assert_eq!(
    spans,
    [(0, 4_000, 0), (3_500, 7_500, 0), (7_000, 10_000, 0)]
  );
  assert_eq!(pieces[0].fade_out, Fade::equal_power(LOOP_CROSSFADE_MS));
  assert_eq!(pieces[1].fade_in, Fade::equal_power(LOOP_CROSSFADE_MS));
  // The last repeat stops partway through the sound, so it fades out.
  assert_eq!(pieces[2].fade_out, Fade::linear(SONG_FADE_OUT_MS));
}

#[test]
fn a_clip_whose_sound_is_missing_is_silent() {
  assert!(laid_out(&[song("1", "gone", 0, 1_000)], &[], 60_000).is_empty());
}

#[test]
fn songs_sit_under_the_voice_as_heard_and_at_a_voice_level_without_one() {
  // A voice heard at -20 LUFS: songs 6 dB under it, sound effects 3 dB.
  assert_eq!(
    default_gain_db(AudioClipKind::Music, Some(-14.0), Some(-20.0)),
    -12.0
  );
  assert_eq!(
    default_gain_db(AudioClipKind::Effect, Some(-14.0), Some(-20.0)),
    -9.0
  );
  assert_eq!(
    default_gain_db(AudioClipKind::Music, Some(-14.0), None),
    -2.0
  );
  // A near-silent sound is lifted 20 dB at most.
  assert_eq!(
    default_gain_db(AudioClipKind::Music, Some(-60.0), None),
    20.0
  );
}

#[test]
fn a_clip_is_levelled_by_the_stretch_it_plays() {
  // Ten seconds quiet, then ten seconds 10 dB louder in power.
  let quiet = 0.1_f32;
  let loud = quiet * 10_f32.sqrt();
  let mut sound = file("song", 20_000);
  sound.levels = [vec![quiet; 100], vec![loud; 100]].concat();
  let loudness = |offset_ms: u64| {
    let mut clip = song("1", "song", 0, 10_000);
    clip.offset_ms = offset_ms;
    super::level::played_loudness(&clip, &sound).unwrap()
  };
  // The sound measures -14 LUFS as a whole; its quiet half plays under that
  // and its loud half over it, 10 dB apart.
  let (intro, chorus) = (loudness(0), loudness(10_000));
  assert!(intro < -14.0 && chorus > -14.0, "{intro} {chorus}");
  assert!((chorus - intro - 10.0).abs() < 0.01, "{intro} {chorus}");
}

#[test]
fn fades_that_would_overlap_share_the_clip() {
  let files = [("a", file("a", 60_000))];
  let mut clip = song("1", "a", 0, 1_500);
  clip.offset_ms = 5_000;
  let pieces = laid_out(&[clip], &files, 60_000);
  assert_eq!((pieces[0].fade_in.ms, pieces[0].fade_out.ms), (500, 1_000));
}
