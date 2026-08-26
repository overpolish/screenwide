// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::exports::recording_preview_player::layout::PreviewPaneKind;
use crate::exports::{AudioTrackKind, RecordingAudioTrack};
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
    &[RecordingPreviewPlaybackRange {
      source_start_ms: 250,
      source_end_ms: 1_000,
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
    &[
      RecordingPreviewPlaybackRange {
        source_start_ms: 250,
        source_end_ms: 500,
      },
      RecordingPreviewPlaybackRange {
        source_start_ms: 750,
        source_end_ms: 1_000,
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

fn test_sources() -> PlayerSources {
  let layout =
    super::super::preview_layout(Some((1_920, 1_080, PreviewPaneKind::Screen)), None, 720);
  PlayerSources {
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
    composition_settings: None,
    cursor: None,
    #[cfg(target_os = "macos")]
    cursor_artworks: None,
    cursor_settings: Default::default(),
    keyboard: None,
    keyboard_settings: Default::default(),
    duration_ms: 1_000,
    frames_per_second: Some(60.0),
    layout: layout.clone(),
    playback_layout: layout,
    playing: Default::default(),
    preview_surface: None,
    primary_kind: PrimaryRecordingKind::Screen,
    screen_path: "/tmp/recording.mov".into(),
  }
}
