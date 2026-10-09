// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::editor::recording_preview_player::layout::PreviewPaneKind;
use crate::editor::recording_preview_player::PreviewProcessing;
use crate::editor::speech::auto_volume::Leveling;
use crate::editor::{AudioTrackKind, RecordingAudioTrack};
use crate::recording::PrimaryRecordingKind;

#[test]
fn decodes_tracks_into_independent_pcm_channels() {
  let sources = test_sources();
  let config = StreamConfig {
    channels: 2,
    sample_rate: 48_000,
    buffer_size: cpal::BufferSize::Default,
  };
  let rendered = args(
    &sources,
    &channels(&sources),
    &[RecordingPreviewPlaybackRange {
      source_start_ms: 250,
      source_end_ms: 1_000,
      playback_rate: 1.0,
    }],
    &config,
    1.0,
  )
  .join(" ");

  assert!(rendered.contains("[0:a:0]atrim"));
  assert!(rendered.contains("[0:a:1]atrim"));
  assert!(rendered.contains("apad=whole_dur=0.750"));
  assert!(rendered.contains("amerge=inputs=2[tracks]"));
  assert!(rendered.contains("-ac 2"));
}

#[test]
fn concatenates_retained_ranges_before_opening_the_output_stream() {
  let mut sources = test_sources();
  sources.audio_tracks.truncate(1);
  let config = StreamConfig {
    channels: 2,
    sample_rate: 48_000,
    buffer_size: cpal::BufferSize::Default,
  };
  let rendered = args(
    &sources,
    &channels(&sources),
    &[
      RecordingPreviewPlaybackRange {
        source_start_ms: 250,
        source_end_ms: 500,
        playback_rate: 1.0,
      },
      RecordingPreviewPlaybackRange {
        source_start_ms: 750,
        source_end_ms: 1_000,
        playback_rate: 1.0,
      },
    ],
    &config,
    1.0,
  )
  .join(" ");

  assert!(rendered.contains("asplit=2"));
  assert!(rendered.contains("atrim=start=0.000:end=0.250"));
  assert!(rendered.contains("atrim=start=0.500:end=0.750"));
  assert!(rendered.contains("afade=t=out:st=0.247:d=0.003"));
  assert!(rendered.contains("afade=t=in:st=0:d=0.003"));
  assert!(rendered.contains("concat=n=2:v=0:a=1"));
  assert!(rendered.contains("-t 0.500"));
}

#[test]
fn applies_atempo_and_scales_audio_duration() {
  let sources = test_sources();
  let config = StreamConfig {
    channels: 2,
    sample_rate: 48_000,
    buffer_size: cpal::BufferSize::Default,
  };
  let rendered = args(
    &sources,
    &channels(&sources),
    &[RecordingPreviewPlaybackRange {
      source_start_ms: 0,
      source_end_ms: 1_000,
      playback_rate: 1.0,
    }],
    &config,
    2.0,
  )
  .join(" ");
  assert!(rendered.contains("atempo=2.0"));
  assert!(rendered.contains("apad=whole_dur=0.500"));
  assert!(rendered.contains("-t 0.500"));
}

#[test]
fn scales_cut_fades_by_the_effective_playback_rate() {
  let mut sources = test_sources();
  sources.audio_tracks.truncate(1);
  let config = StreamConfig {
    channels: 1,
    sample_rate: 48_000,
    buffer_size: cpal::BufferSize::Default,
  };
  let ranges = [
    RecordingPreviewPlaybackRange {
      source_start_ms: 0,
      source_end_ms: 500,
      playback_rate: 2.0,
    },
    RecordingPreviewPlaybackRange {
      source_start_ms: 500,
      source_end_ms: 1_000,
      playback_rate: 1.0,
    },
  ];
  let rendered = args(&sources, &channels(&sources), &ranges, &config, 2.0).join(" ");

  assert!(rendered.contains("afade=t=out:st=0.122:d=0.003"));
  assert!(rendered.contains("apad=whole_dur=0.375"));
}

#[test]
fn plays_a_processed_track_from_its_own_file_at_the_same_place() {
  let sources = test_sources();
  let noise = Processing {
    noise: true,
    voice: false,
  };
  let both = Processing {
    noise: true,
    voice: true,
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
    channels
      .iter()
      .map(|channel| mix.hears(channel, &[0, 1], (heard, leveled)))
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
    noise: false,
    voice: true,
  };
  assert_eq!(heard(&[(1, voice)], &[]), [true, true, false, false, false]);
}

fn test_sources() -> PlayerSources {
  let layout =
    super::super::preview_layout(Some((1_920, 1_080, PreviewPaneKind::Screen)), None, 720);
  PlayerSources {
    annotation_clips: Arc::new(RwLock::new(Vec::new())),
    scenes: Default::default(),
    audio_tracks: vec![
      RecordingAudioTrack {
        kind: AudioTrackKind::SystemAudio,
        label: "System audio".to_owned(),
        stream_index: 0,
      },
      RecordingAudioTrack {
        kind: AudioTrackKind::Microphone,
        label: "Microphone".to_owned(),
        stream_index: 1,
      },
    ],
    camera_duration_ms: None,
    camera_path: None,
    captures: Default::default(),
    composition_settings: None,
    cursor: None,
    #[cfg(target_os = "macos")]
    cursor_artworks: None,
    cursor_settings: Default::default(),
    keyboard: None,
    animation_ranges: Default::default(),
    keyboard_settings: Default::default(),
    duration_ms: 1_000,
    frames_per_second: Some(60.0),
    held_fills: [None, None],
    layout: layout.clone(),
    playback_layout: layout,
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pins: None,
    playing: Default::default(),
    preview_surface: None,
    primary_kind: PrimaryRecordingKind::Screen,
    screen_path: "/tmp/recording.mov".into(),
    processing: Default::default(),
    video_muted: Default::default(),
  }
}

#[test]
fn audio_scrubs_acknowledge_each_position_without_a_video_decoder() {
  use super::super::{worker::PlaybackMode, PreviewPlayerManager};
  use tauri::ipc::{Channel, InvokeResponseBody};

  for hidden_video in [false, true] {
    let mut sources = test_sources();
    if hidden_video {
      sources.video_muted.store(true, Ordering::Release);
    } else {
      sources.layout.panes.clear();
    }
    let messages = Arc::new(Mutex::new(Vec::new()));
    let received = Arc::clone(&messages);
    let mut manager = PreviewPlayerManager {
      sources: Some(sources),
      event_channel: Some(Channel::new(move |message| {
        if let InvokeResponseBody::Json(json) = message {
          received
            .lock()
            .unwrap()
            .push(serde_json::from_str::<serde_json::Value>(&json).unwrap());
        }
        Ok(())
      })),
      ..Default::default()
    };
    for (request, position) in [(1, 750), (2, 250), (3, 900)] {
      manager.latest_seek_request = request;
      manager.position_ms = position;
      manager.rough_seek = true;
      manager.restart(PlaybackMode::InteractiveStill).unwrap();
      assert!(manager.worker.is_none());
      assert!(manager.still_decoder.is_none());
      assert!(!manager.rough_seek);
      let messages = messages.lock().unwrap();
      let last = messages.last().unwrap();
      assert_eq!(last["event"], "ready");
      assert_eq!(last["data"]["positionMs"], position);
      assert_eq!(last["data"]["requestId"], request);
    }
  }
}
