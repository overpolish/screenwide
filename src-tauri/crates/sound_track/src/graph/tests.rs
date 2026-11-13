// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Read;
use std::path::PathBuf;
use std::process::Stdio;

use super::*;
use crate::pieces::Parts;

const RATE: u64 = 8_000;

/// The songs and sound effects' filters, mixed into `[out]`, with no
/// recorded tracks to make room in.
fn sound(pieces: &[Piece], window: (u64, u64), then: &str, voice: Option<&str>) -> Option<String> {
  filter(pieces, window, ("out", then), voice, &[]).map(|mix| mix.graph)
}

fn piece(path: &Path, file_start_ms: u64, start_ms: u64, end_ms: u64) -> Piece {
  Piece {
    path: path.to_path_buf(),
    file_start_ms,
    start_ms,
    end_ms,
    fade_in: Fade {
      ms: 0,
      curve: Curve::Linear,
    },
    fade_out: Fade {
      ms: 0,
      curve: Curve::Linear,
    },
    gain_db: 0.0,
    tone: Tone::Whole,
    speed: 1.0,
    makes_way: false,
    is_effect: false,
    instrumental: false,
    parts: None,
  }
}

fn tone(folder: &Path, name: &str, seconds: u32) -> PathBuf {
  made(
    folder,
    name,
    &format!("sine=frequency=440:duration={seconds}:sample_rate=8000"),
  )
}

fn silence(folder: &Path, name: &str, seconds: u32) -> PathBuf {
  made(
    folder,
    name,
    &format!("anullsrc=sample_rate=8000:channel_layout=mono,atrim=duration={seconds}"),
  )
}

fn made(folder: &Path, name: &str, source: &str) -> PathBuf {
  let path = folder.join(name);
  let status = screenwide_sidecar::ffmpeg_command()
    .args([
      "-hide_banner",
      "-loglevel",
      "error",
      "-nostdin",
      "-y",
      "-f",
      "lavfi",
      "-i",
    ])
    .arg(source)
    .arg(&path)
    .status()
    .unwrap();
  assert!(status.success());
  path
}

/// A song that is all singing, separated: its singing is all of it, and the
/// rest silent.
fn all_singing(folder: &Path) -> (PathBuf, Parts) {
  let sung = tone(folder, "song.wav", 3);
  let parts = Parts {
    singing: sung.clone(),
    rest: silence(folder, "song.rest.wav", 3),
  };
  (sung, parts)
}

/// The graph played to mono samples at `RATE`.
fn played(graph: &str) -> Vec<f32> {
  let mut child = screenwide_sidecar::ffmpeg_command()
    .args([
      "-hide_banner",
      "-loglevel",
      "error",
      "-nostdin",
      "-filter_complex",
      graph,
    ])
    .args(["-map", "[out]", "-f", "f32le", "pipe:1"])
    .stdout(Stdio::piped())
    .spawn()
    .unwrap();
  let mut bytes = Vec::new();
  child
    .stdout
    .take()
    .unwrap()
    .read_to_end(&mut bytes)
    .unwrap();
  assert!(child.wait().unwrap().success());
  bytes
    .as_chunks::<4>()
    .0
    .iter()
    .map(|chunk| f32::from_le_bytes(*chunk))
    .collect()
}

fn loudest(samples: &[f32]) -> f32 {
  samples
    .iter()
    .fold(0.0, |most, sample| most.max(sample.abs()))
}

fn scratch(name: &str) -> PathBuf {
  let folder = std::env::temp_dir().join(format!(
    "screenwide-sound-graph-{name}-{}",
    std::process::id()
  ));
  let _ = std::fs::remove_dir_all(&folder);
  std::fs::create_dir_all(&folder).unwrap();
  folder
}

#[test]
fn escapes_a_path_for_both_readings_of_a_filter_graph() {
  assert_eq!(
    filter_path(Path::new("/a b/it's, [x];y:z.flac")),
    r"/a b/it\\\'s\, \[x\]\;y\\:z.flac"
  );
}

#[test]
fn a_window_that_starts_inside_a_long_piece_seeks_to_it() {
  let pieces = [piece(Path::new("/song.mp3"), 0, 0, 180_000)];
  let graph = sound(&pieces, (90_000, 180_000), "", None).unwrap();
  assert!(graph.contains("seek_point=90.000,atrim=start=90.000:duration=90.000"));
}

#[test]
fn nothing_in_the_window_makes_no_graph() {
  let pieces = [piece(Path::new("/song.mp3"), 0, 0, 1_000)];
  assert!(sound(&pieces, (1_000, 2_000), "", None).is_none());
}

#[test]
fn pieces_play_where_they_sit_on_the_timeline_from_the_window_on() {
  let folder = scratch("placed");
  let a = tone(&folder, "a,'b [c].wav", 3);
  let pieces = [piece(&a, 0, 0, 1_000), piece(&a, 1_000, 2_000, 3_000)];
  let format = format!(",aresample={RATE},aformat=sample_fmts=flt:channel_layouts=mono");
  let samples = played(&sound(&pieces, (500, 3_000), &format, None).unwrap());
  let at = |ms: u64| (ms * RATE / 1_000) as usize;
  assert_eq!(samples.len(), at(2_500));
  // FFmpeg's sine is an eighth of full scale.
  assert!(loudest(&samples[at(0)..at(450)]) > 0.1);
  assert!(loudest(&samples[at(550)..at(1_450)]) < 1e-3);
  assert!(loudest(&samples[at(1_550)..at(2_450)]) > 0.1);
  let _ = std::fs::remove_dir_all(&folder);
}

#[test]
fn a_fade_out_reaches_silence_at_the_piece_end() {
  let folder = scratch("fade");
  let a = tone(&folder, "a.wav", 3);
  let mut faded = piece(&a, 0, 0, 2_000);
  faded.fade_out = Fade {
    ms: 1_000,
    curve: Curve::Linear,
  };
  let format = format!(",aresample={RATE},aformat=sample_fmts=flt:channel_layouts=mono");
  let samples = played(&sound(&[faded], (0, 2_000), &format, None).unwrap());
  let at = |ms: u64| (ms * RATE / 1_000) as usize;
  assert!(loudest(&samples[at(0)..at(900)]) > 0.1);
  assert!(loudest(&samples[at(1_950)..]) < 0.01);
  let _ = std::fs::remove_dir_all(&folder);
}

/// A voice that speaks for the first second of three, opened in the graph
/// as `[voice]`.
fn voice(folder: &Path) -> String {
  let path = folder.join("voice.wav");
  let status = screenwide_sidecar::ffmpeg_command()
    .args([
      "-hide_banner",
      "-loglevel",
      "error",
      "-nostdin",
      "-y",
      "-f",
      "lavfi",
      "-i",
    ])
    .arg("sine=frequency=220:duration=1:sample_rate=8000,volume=4,apad=whole_dur=3")
    .arg(&path)
    .status()
    .unwrap();
  assert!(status.success());
  format!("amovie={}[voice];", filter_path(&path))
}

/// How loud the song is while the voice speaks, against once it has stopped.
fn ducked(song: Piece, folder: &Path) -> f32 {
  let format = format!(",aresample={RATE},aformat=sample_fmts=flt:channel_layouts=mono");
  let graph = sound(&[song], (0, 3_000), &format, Some("[voice]")).unwrap();
  let samples = played(&format!("{}{graph}", voice(folder)));
  let at = |ms: u64| (ms * RATE / 1_000) as usize;
  loudest(&samples[at(300)..at(900)]) / loudest(&samples[at(2_200)..at(2_900)])
}

#[test]
fn a_song_that_makes_way_comes_down_while_the_voice_speaks() {
  let folder = scratch("make-way");
  let mut song = piece(&tone(&folder, "song.wav", 3), 0, 0, 3_000);
  song.makes_way = true;
  let ratio = ducked(song.clone(), &folder);
  assert!(ratio < 0.5, "came down to {ratio}");
  song.makes_way = false;
  assert!(ducked(song, &folder) > 0.9);
  let _ = std::fs::remove_dir_all(&folder);
}

#[test]
fn the_singing_in_a_song_gets_out_of_the_way_entirely() {
  let folder = scratch("singing");
  // Nothing of a song that is all singing should be left while the voice
  // speaks.
  let (sung, parts) = all_singing(&folder);
  let mut song = piece(&sung, 0, 0, 3_000);
  song.makes_way = true;
  song.parts = Some(parts);
  let ratio = ducked(song, &folder);
  assert!(ratio < 0.15, "came down to {ratio}");
  let _ = std::fs::remove_dir_all(&folder);
}

#[test]
fn an_instrumental_song_plays_without_its_singing_once_that_is_separated() {
  let folder = scratch("instrumental");
  let (sung, parts) = all_singing(&folder);
  let format = format!(",aresample={RATE},aformat=sample_fmts=flt:channel_layouts=mono");
  let loudness = |song: Piece| {
    let graph = sound(&[song], (0, 3_000), &format, None).unwrap();
    loudest(&played(&graph)[..(2 * RATE) as usize])
  };
  let mut song = piece(&sung, 0, 0, 3_000);
  song.instrumental = true;
  // Until its singing is separated it plays as it is.
  let whole = loudness(song.clone());
  assert!(whole > 0.1, "played at {whole}");
  song.parts = Some(parts);
  let without = loudness(song);
  assert!(without < whole * 0.01, "{without} left of {whole}");
  let _ = std::fs::remove_dir_all(&folder);
}

/// Where each burst starts in `samples` at `RATE`, in ms.
fn bursts(samples: &[f32]) -> Vec<f64> {
  let mut starts = Vec::new();
  let mut quiet_for = usize::MAX;
  for (index, sample) in samples.iter().enumerate() {
    if sample.abs() > 0.3 {
      if quiet_for > 400 {
        starts.push(index as f64 * 1_000.0 / RATE as f64);
      }
      quiet_for = 0;
    } else {
      quiet_for = quiet_for.saturating_add(1);
    }
  }
  starts
}

#[test]
fn stretched_pieces_play_their_files_at_their_own_speeds() {
  let folder = scratch("stretched");
  // A 100 ms note every half second of the file over a steady tone, as
  // music has no silence for `atempo` to fall in step with.
  let path = made(
    &folder,
    "notes.wav",
    "aevalsrc=exprs='0.15*sin(2*PI*220*t)+if(lt(mod(t\\,0.5)\\,0.1)\\,0.7*sin(2*PI*880*t)\\,0)':s=8000:d=8",
  );
  // From a second into the file: one a quarter faster, the other at 0.8,
  // each note where its speed puts it.
  let mut faster = piece(&path, 1_000, 0, 4_000);
  faster.speed = 1.25;
  let mut slower = piece(&path, 1_000, 5_000, 9_000);
  slower.speed = 0.8;
  let format = format!(",aresample={RATE},aformat=sample_fmts=flt:channel_layouts=mono");
  let graph = sound(&[faster, slower], (0, 9_000), &format, None).unwrap();
  let heard = bursts(&played(&graph));
  let mut expected: Vec<f64> = (0..10).map(|burst| f64::from(burst) * 400.0).collect();
  expected.extend((0..7).map(|burst| 5_000.0 + f64::from(burst) * 625.0));
  assert_eq!(heard.len(), expected.len(), "heard {heard:?}");
  for (heard, expected) in heard.iter().zip(&expected) {
    assert!(
      (heard - expected).abs() < 15.0,
      "heard {heard} for {expected}"
    );
  }
  let _ = std::fs::remove_dir_all(&folder);
}

#[test]
fn system_audio_makes_room_only_in_the_bands_a_sound_effect_has() {
  let folder = scratch("carve");
  // A 2 kHz bed, handed in as system audio, under a second of an effect from
  // 1 s: how far the bed comes down while the effect plays, in dB.
  let dip = |effect_hz: u32| {
    let effect = made(
      &folder,
      &format!("effect{effect_hz}.wav"),
      &format!("sine=frequency={effect_hz}:duration=1:sample_rate=8000"),
    );
    let mut hit = piece(&effect, 0, 1_000, 2_000);
    hit.is_effect = true;
    let mix = filter(
      &[hit],
      (0, 3_000),
      ("soundout", ""),
      None,
      &["[bed]".to_owned()],
    )
    .unwrap();
    let graph = format!(
      "sine=frequency=2000:duration=3:sample_rate=8000[bed];{};[soundout]anullsink;\
       {}aresample={RATE},aformat=sample_fmts=flt:channel_layouts=mono[out]",
      mix.graph, mix.beside[0]
    );
    let samples = played(&graph);
    let rms = |from_ms: u64, to_ms: u64| {
      let stretch = &samples[(from_ms * RATE / 1_000) as usize..(to_ms * RATE / 1_000) as usize];
      (stretch.iter().map(|sample| sample * sample).sum::<f32>() / stretch.len() as f32).sqrt()
    };
    20.0 * (rms(1_300, 1_900) / rms(200, 800)).log10()
  };
  // An effect in the bed's band carves a few decibels out of it; one in the
  // bass leaves it be.
  let matching = dip(2_000);
  let bass = dip(100);
  assert!((-8.0..-3.0).contains(&matching), "came down {matching} dB");
  assert!(bass > -1.0, "came down {bass} dB");
  let _ = std::fs::remove_dir_all(&folder);
}
