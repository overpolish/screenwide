// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Read;
use std::path::PathBuf;
use std::process::Stdio;

use super::*;

const RATE: u64 = 8_000;

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
    makes_way: false,
    singing: None,
  }
}

fn tone(folder: &Path, name: &str, seconds: u32) -> PathBuf {
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
    .arg(format!(
      "sine=frequency=440:duration={seconds}:sample_rate=8000"
    ))
    .arg(&path)
    .status()
    .unwrap();
  assert!(status.success());
  path
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
  let graph = filter(&pieces, (90_000, 180_000), "out", "", None).unwrap();
  assert!(graph.contains("seek_point=90.000,atrim=start=90.000:duration=90.000"));
}

#[test]
fn nothing_in_the_window_makes_no_graph() {
  let pieces = [piece(Path::new("/song.mp3"), 0, 0, 1_000)];
  assert!(filter(&pieces, (1_000, 2_000), "out", "", None).is_none());
}

#[test]
fn pieces_play_where_they_sit_on_the_timeline_from_the_window_on() {
  let folder = scratch("placed");
  let a = tone(&folder, "a,'b [c].wav", 3);
  let pieces = [piece(&a, 0, 0, 1_000), piece(&a, 1_000, 2_000, 3_000)];
  let format = format!(",aresample={RATE},aformat=sample_fmts=flt:channel_layouts=mono");
  let samples = played(&filter(&pieces, (500, 3_000), "out", &format, None).unwrap());
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
  let samples = played(&filter(&[faded], (0, 2_000), "out", &format, None).unwrap());
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
  let graph = filter(&[song], (0, 3_000), "out", &format, Some("[voice]")).unwrap();
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
  // A song that is all singing: nothing of it should be left while the
  // voice speaks.
  let sung = tone(&folder, "song.wav", 3);
  let mut song = piece(&sung, 0, 0, 3_000);
  song.makes_way = true;
  song.singing = Some(sung);
  let ratio = ducked(song, &folder);
  assert!(ratio < 0.15, "came down to {ratio}");
  let _ = std::fs::remove_dir_all(&folder);
}
