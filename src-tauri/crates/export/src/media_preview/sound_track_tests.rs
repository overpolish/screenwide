// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::atomic::AtomicBool;

use screenwide_sidecar::ffmpeg_path;
use screenwide_sound_track::{AudioClipKind, RecordingAudioClip, SoundFile};
use screenwide_timeline_edit::{RecordingTimelineEdit, RecordingTimelineSegment, TimelinePlan};

use super::encode::export_selected_audio;
use super::*;

const RATE: usize = 8_000;

fn directory(name: &str) -> PathBuf {
  let directory = std::env::temp_dir()
    .join("screenwide-tests")
    .join(format!("sound-track-{name}"));
  let _ = std::fs::remove_dir_all(&directory);
  std::fs::create_dir_all(&directory).unwrap();
  directory
}

fn made(args: &[&str], destination: &Path) {
  let status = Command::new(ffmpeg_path())
    .args(["-hide_banner", "-loglevel", "error", "-nostdin", "-y"])
    .args(args)
    .arg(destination)
    .status()
    .unwrap();
  assert!(status.success());
}

/// A three-second recording whose one track is a voice-like tone, and a
/// two-second song, with a timeline that plays the song from 1 s. A song
/// that makes way plays under a voice that stops at 2 s.
fn recording_with_a_song(folder: &Path, makes_way: bool) -> (PathBuf, TimelinePlan) {
  let recording = folder.join("recording.mov");
  let voice = if makes_way {
    "sine=frequency=220:duration=2,volume=4,apad=whole_dur=3"
  } else {
    "sine=frequency=220:duration=3"
  };
  made(&["-f", "lavfi", "-i", voice, "-c:a", "aac"], &recording);
  let song = folder.join("song.wav");
  made(
    &["-f", "lavfi", "-i", "sine=frequency=880:duration=2"],
    &song,
  );
  let clip = RecordingAudioClip {
    id: "song".to_owned(),
    sound: "0123456789abcdef0123456789abcdef.wav".to_owned(),
    kind: AudioClipKind::Music,
    lane: 0,
    start_ms: 1_000,
    end_ms: 3_000,
    offset_ms: 0,
    volume_db: 0,
    is_looped: false,
    is_instrumental: false,
  };
  let mut timeline = TimelinePlan::from_edit(
    &RecordingTimelineEdit {
      audio_clips: vec![clip],
      muted_audio: Vec::new(),
      annotation_clips: Vec::new(),
      artifact_id: 1,
      keyboard_deletions: Box::default(),
      silence_cuts: Vec::new(),
      transcript_cuts: Vec::new(),
      transcript_corrections: Vec::new(),
      transcript_allowed_words: Vec::new(),
      captions: None,
      scene_clips: Vec::new(),
      camera_background_ranges: Vec::new(),
      next_segment_id: 1,
      segments: vec![RecordingTimelineSegment {
        id: 0,
        source_end: 1.0,
        source_start: 0.0,
        playback_rate: 1.0,
      }],
    },
    3_000,
  )
  .expect("a timeline with a song is an edit");
  // The quiet tone only stands for a voice where the song makes way for it,
  // so the plain tests hear the song at the level it has with no voice.
  timeline.find_sounds((makes_way.then_some(0), Some(-16.0), makes_way), |_| {
    Some(SoundFile {
      path: song.clone(),
      duration_ms: 2_000,
      loudness_lufs: Some(-21.0),
      levels: Vec::new(),
      separated: None,
      music: Default::default(),
    })
  });
  (recording, timeline)
}

fn exported(source: &Path, destination: &Path, timeline: &TimelinePlan, file: AudioFile) {
  let tracks = [RecordingAudioTrack {
    kind: AudioTrackKind::Microphone,
    label: "Microphone".to_owned(),
    stream_index: 0,
  }];
  let cancelled = AtomicBool::new(false);
  export_selected_audio(
    source,
    destination,
    &TrackSelection::new(&tracks, &[0]),
    AudioLayout::SeparateTracks,
    file,
    ExportRunOptions {
      cancelled: &cancelled,
      on_progress: &mut |_| {},
      timeline: Some(timeline),
      video: VideoExportOptions {
        compression: 0,
        resolution_scale_percent: 100,
        source_scale_percent: 100,
      },
    },
  )
  .unwrap();
}

/// The `stream`th audio stream of `path`, as mono samples at `RATE`.
fn samples(path: &Path, stream: usize) -> Vec<f32> {
  let mut child = Command::new(ffmpeg_path())
    .args(["-hide_banner", "-loglevel", "error", "-nostdin", "-i"])
    .arg(path)
    .args([
      "-map",
      &format!("0:a:{stream}"),
      "-ac",
      "1",
      "-ar",
      &RATE.to_string(),
    ])
    .args(["-f", "f32le", "pipe:1"])
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

fn loudness(samples: &[f32], (from_ms, to_ms): (usize, usize)) -> f32 {
  let part = &samples[from_ms * RATE / 1_000..to_ms * RATE / 1_000];
  (part.iter().map(|sample| sample * sample).sum::<f32>() / part.len() as f32).sqrt()
}

#[test]
fn a_mixed_down_file_carries_the_song_where_it_sits_on_the_timeline() {
  let folder = directory("mixdown");
  let (recording, timeline) = recording_with_a_song(&folder, false);
  let destination = folder.join("mixed.wav");
  exported(&recording, &destination, &timeline, AudioFile::Wav);
  let mixed = samples(&destination, 0);
  assert!(mixed.len().abs_diff(3 * RATE) < RATE / 20);
  let voice_alone = loudness(&mixed, (100, 900));
  let with_song = loudness(&mixed, (1_100, 2_900));
  assert!(voice_alone > 0.01);
  assert!(with_song > voice_alone * 1.5);
  let _ = std::fs::remove_dir_all(&folder);
}

#[test]
fn kept_apart_the_songs_are_a_stream_after_the_recorded_tracks() {
  let folder = directory("separate");
  let (recording, timeline) = recording_with_a_song(&folder, false);
  let destination = folder.join("separate.m4a");
  exported(&recording, &destination, &timeline, AudioFile::M4a);
  let music = samples(&destination, 1);
  assert!(loudness(&music, (100, 900)) < 1e-3);
  assert!(loudness(&music, (1_100, 2_900)) > 0.05);
  let _ = std::fs::remove_dir_all(&folder);
}

#[test]
fn a_song_that_makes_way_comes_down_under_the_exported_voice() {
  let folder = directory("make-way");
  let (recording, timeline) = recording_with_a_song(&folder, true);
  let destination = folder.join("made-way.m4a");
  exported(&recording, &destination, &timeline, AudioFile::M4a);
  let voice = samples(&destination, 0);
  let music = samples(&destination, 1);
  // The voice is exported whole, though the song listens to it.
  assert!(loudness(&voice, (200, 1_800)) > 0.1);
  let under_the_voice = loudness(&music, (1_300, 1_900));
  let after_it = loudness(&music, (2_400, 2_900));
  assert!(
    under_the_voice < after_it * 0.5,
    "{under_the_voice} against {after_it}"
  );
  let _ = std::fs::remove_dir_all(&folder);
}
