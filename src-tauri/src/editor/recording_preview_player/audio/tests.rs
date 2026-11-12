// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::editor::recording_preview_player::layout::PreviewPaneKind;
use crate::editor::recording_preview_player::sound_track::PreviewSoundTrack;
use crate::editor::{AudioTrackKind, RecordingAudioTrack};
use screenwide_project::PrimaryRecordingKind;

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
    None,
    &[RecordingPreviewPlaybackRange {
      source_start_ms: 250,
      source_end_ms: 1_000,
      playback_rate: 1.0,
    }],
    &config,
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
    None,
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

/// Played 750-1000 ms, then 250-500 ms: the decode starts at 250 ms, the
/// earlier of the two, so the first range played is trimmed from later on.
#[test]
fn trims_reordered_ranges_from_the_earliest_source_start() {
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
    None,
    &[
      RecordingPreviewPlaybackRange {
        source_start_ms: 750,
        source_end_ms: 1_000,
        playback_rate: 1.0,
      },
      RecordingPreviewPlaybackRange {
        source_start_ms: 250,
        source_end_ms: 500,
        playback_rate: 1.0,
      },
    ],
    &config,
  )
  .join(" ");

  assert!(rendered.contains("-ss 0.250"));
  assert!(rendered.contains("atrim=start=0.500:end=0.750"));
  assert!(rendered.contains("atrim=start=0.000:end=0.250"));
  assert!(rendered.find("start=0.500").unwrap() < rendered.find("start=0.000").unwrap());
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
    None,
    &[RecordingPreviewPlaybackRange {
      source_start_ms: 0,
      source_end_ms: 1_000,
      playback_rate: 2.0,
    }],
    &config,
  )
  .join(" ");
  assert!(rendered.contains("atempo=2.0"));
  assert!(rendered.contains("apad=whole_dur=0.500"));
  assert!(rendered.contains("-t 0.500"));
}

#[test]
fn scales_cut_fades_by_the_range_playback_rate() {
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
      playback_rate: 4.0,
    },
    RecordingPreviewPlaybackRange {
      source_start_ms: 500,
      source_end_ms: 1_000,
      playback_rate: 2.0,
    },
  ];
  let rendered = args(&sources, &channels(&sources), None, &ranges, &config).join(" ");

  assert!(rendered.contains("afade=t=out:st=0.122:d=0.003"));
  assert!(rendered.contains("apad=whole_dur=0.375"));
}

/// The preview's command for two recorded tracks and a song placed from
/// 0.2 s on, run through FFmpeg: one channel more, as long as the tracks.
#[test]
fn the_sound_track_plays_as_one_channel_more_padded_to_the_tracks() {
  let folder =
    std::env::temp_dir().join(format!("screenwide-preview-sound-{}", std::process::id()));
  let _ = std::fs::remove_dir_all(&folder);
  std::fs::create_dir_all(&folder).unwrap();
  let made = |args: &[&str], path: &std::path::Path| {
    let status = screenwide_sidecar::ffmpeg_command()
      .args(["-hide_banner", "-loglevel", "error", "-nostdin", "-y"])
      .args(args)
      .arg(path)
      .status()
      .unwrap();
    assert!(status.success());
  };
  let mut sources = test_sources();
  sources.screen_path = folder.join("recording.mov");
  let tone = |frequency: u32| format!("sine=frequency={frequency}:duration=1");
  made(
    &[
      "-f",
      "lavfi",
      "-i",
      &tone(220),
      "-f",
      "lavfi",
      "-i",
      &tone(330),
      "-map",
      "0:a",
      "-map",
      "1:a",
    ],
    &sources.screen_path,
  );
  let song = folder.join("song.wav");
  made(&["-f", "lavfi", "-i", &tone(880)], &song);
  let pieces = [screenwide_sound_track::Piece {
    path: song,
    file_start_ms: 0,
    start_ms: 200,
    end_ms: 1_000,
    fade_in: screenwide_sound_track::pieces::Fade {
      ms: 0,
      curve: screenwide_sound_track::pieces::Curve::Linear,
    },
    fade_out: screenwide_sound_track::pieces::Fade {
      ms: 0,
      curve: screenwide_sound_track::pieces::Curve::Linear,
    },
    gain_db: 0.0,
    // Making way, it listens to the microphone, stream 1, split from its
    // channel.
    makes_way: true,
    instrumental: false,
    parts: None,
  }];
  let sound = PreviewSoundTrack::laid_out(pieces.to_vec(), (0, 1_000), Some(1));
  let config = StreamConfig {
    channels: 2,
    sample_rate: 8_000,
    buffer_size: cpal::BufferSize::Default,
  };
  let ranges = [RecordingPreviewPlaybackRange {
    source_start_ms: 0,
    source_end_ms: 500,
    playback_rate: 1.0,
  }];
  let output = screenwide_sidecar::ffmpeg_command()
    .args(args(
      &sources,
      &channels(&sources),
      Some(&sound),
      &ranges,
      &config,
    ))
    .output()
    .unwrap();
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  // Three channels of 0.5 s at 8 kHz, as 32-bit floats.
  let samples: Vec<f32> = output
    .stdout
    .as_chunks::<4>()
    .0
    .iter()
    .map(|chunk| f32::from_le_bytes(*chunk))
    .collect();
  assert_eq!(samples.len(), 3 * 4_000);
  let song_channel = |from: usize, to: usize| {
    samples[from * 3..to * 3]
      .iter()
      .skip(2)
      .step_by(3)
      .fold(0.0_f32, |most, sample| most.max(sample.abs()))
  };
  assert!(song_channel(0, 1_500) < 1e-3);
  // Heard under the microphone, which speaks throughout, so it has come
  // down, but it is there.
  assert!(song_channel(1_700, 4_000) > 0.01);
  let _ = std::fs::remove_dir_all(&folder);
}

pub(super) fn test_sources() -> PlayerSources {
  let layout =
    super::super::preview_layout(Some((1_920, 1_080, PreviewPaneKind::Screen)), None, 720);
  PlayerSources {
    annotation_clips: Arc::new(RwLock::new(Vec::new())),
    annotation_ranges: Default::default(),
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
    cursor_settings: Default::default(),
    keyboard: None,
    animation_ranges: Default::default(),
    keyboard_settings: Default::default(),
    captions: Default::default(),
    duration_ms: 1_000,
    frames_per_second: Some(60.0),
    held_fills: [None, None],
    layout: layout.clone(),
    playback_layout: layout,
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pins: None,
    playing: Default::default(),
    preview_surface: None,
    camera_effects: Default::default(),
    camera_picking: Default::default(),
    primary_kind: PrimaryRecordingKind::Screen,
    project: "/tmp/recording.screenwide".into(),
    screen_path: "/tmp/recording.mov".into(),
    sound_track: Default::default(),
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
