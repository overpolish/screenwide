// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Which of a track's channels the preview mixes, as the switches that make
//! files of it say.

use super::tests::test_sources;
use super::*;
use crate::editor::recording_preview_player::PreviewProcessing;
use crate::editor::speech::auto_volume::Leveling;

#[test]
fn plays_a_processed_track_from_its_own_file_at_the_same_place() {
  let sources = test_sources();
  let noise = Processing {
    noise: true,
    ..Processing::default()
  };
  let both = Processing {
    noise: true,
    voice: true,
    ..Processing::default()
  };
  let ducked = Processing {
    duck: true,
    ..Processing::default()
  };
  let leveling = Leveling {
    threshold_db: -30.0,
    makeup_db: 12.0,
  };
  *sources.processing.write().unwrap() = PreviewProcessing {
    files: vec![
      (1, noise, "/tmp/noise-1.flac".into()),
      (1, both, "/tmp/noise-voice-1.flac".into()),
    ],
    heard: vec![(1, noise)],
    levelings: vec![(1, noise, leveling)],
    leveled: Vec::new(),
    ducked_by: Vec::new(),
  };
  let config = StreamConfig {
    channels: 2,
    sample_rate: 48_000,
    buffer_size: cpal::BufferSize::Default,
  };
  let channels = channels(&sources);
  let rendered = args(
    &sources,
    &channels,
    &[RecordingPreviewPlaybackRange {
      source_start_ms: 250,
      source_end_ms: 1_000,
      playback_rate: 1.0,
    }],
    &config,
    1.0,
  )
  .join(" ");

  assert!(rendered.contains("-ss 0.250 -i /tmp/noise-1.flac"));
  assert!(rendered.contains("-ss 0.250 -i /tmp/noise-voice-1.flac"));
  assert!(rendered.contains("[1:a:0]atrim"));
  assert!(rendered.contains("[2:a:0]atrim"));
  // The noise-reduced file is decoded a second time, leveled.
  assert!(rendered.contains(&format!("[3:a:0]{}[level4];", leveling.filters())));
  assert!(rendered.contains("[level4]atrim"));
  assert!(rendered.contains("amerge=inputs=5[tracks]"));
  // The microphone and all its files are decoded; the switches pick which
  // is heard, and a channel the preview lacks falls back to the recording,
  // or to the unleveled file.
  let mix = Mix {
    audio_volumes: Default::default(),
    channels: channels.clone(),
    processing: Arc::clone(&sources.processing),
    selected_audio: Arc::new(RwLock::new(vec![0, 1])),
  };
  let heard = |heard: &[(usize, Processing)], leveled: &[usize]| {
    let processing = PreviewProcessing {
      heard: heard.to_vec(),
      leveled: leveled.to_vec(),
      ..PreviewProcessing::default()
    };
    channels
      .iter()
      .map(|channel| mix.hears(channel, &[0, 1], &processing))
      .collect::<Vec<_>>()
  };
  assert_eq!(heard(&[(1, noise)], &[]), [true, false, true, false, false]);
  assert_eq!(
    heard(&[(1, noise)], &[1]),
    [true, false, false, false, true]
  );
  assert_eq!(heard(&[(1, both)], &[1]), [true, false, false, true, false]);
  assert_eq!(heard(&[], &[]), [true, true, false, false, false]);
  let voice = Processing {
    voice: true,
    ..Processing::default()
  };
  assert_eq!(heard(&[(1, voice)], &[]), [true, true, false, false, false]);
  // The system audio has no file to make way from here, so it plays as
  // recorded however it is asked for.
  assert_eq!(
    heard(&[(0, ducked)], &[]),
    [true, true, false, false, false]
  );
}

#[test]
fn system_audio_makes_way_only_while_its_microphone_is_on() {
  let sources = test_sources();
  let ducked = Processing {
    duck: true,
    ..Processing::default()
  };
  *sources.processing.write().unwrap() = PreviewProcessing {
    files: vec![(0, ducked, "/tmp/duck-0.flac".into())],
    heard: vec![(0, ducked)],
    ducked_by: vec![(0, 1)],
    ..PreviewProcessing::default()
  };
  let channels = channels(&sources);
  let mix = Mix {
    audio_volumes: Default::default(),
    channels: channels.clone(),
    processing: Arc::clone(&sources.processing),
    selected_audio: Default::default(),
  };
  let processing = sources.processing.read().unwrap().clone();
  let heard = |selected: &[usize]| {
    channels
      .iter()
      .map(|channel| mix.hears(channel, selected, &processing))
      .collect::<Vec<_>>()
  };
  // Recorded system audio, the microphone, and the system audio made way.
  assert_eq!(heard(&[0, 1]), [false, true, true]);
  assert_eq!(heard(&[0]), [true, false, false]);
}
