// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use screenwide_sidecar::ffmpeg_command;

pub fn inspect_audio_tracks(source: &Path) -> Result<Vec<RecordingAudioTrack>, String> {
  // FFmpeg prints container metadata before it complains that no output was
  // supplied. That gives a recording's audio tracks without shipping a
  // second, almost equally large FFprobe executable.
  let output = ffmpeg_command()
    .args(["-hide_banner", "-nostdin", "-i"])
    .arg(source)
    .output()
    .map_err(|error| format!("FFmpeg could not be started: {error}"))?;
  let metadata = String::from_utf8_lossy(&output.stderr);
  let streams: Vec<&str> = metadata
    .lines()
    .filter(|line| line.contains("Stream #") && line.contains(" Audio:"))
    .collect();

  Ok(
    streams
      .iter()
      .enumerate()
      .map(|(stream_index, line)| {
        let kind = inferred_audio_track(line, stream_index, streams.len());
        RecordingAudioTrack {
          kind,
          label: kind.label(stream_index),
          stream_index,
        }
      })
      .collect(),
  )
}

/// Names a stream from what the recorder always writes: system audio first
/// and in stereo, the microphone after it and in mono. A recording from
/// before the metadata sidecar has nothing else to say which is which.
fn inferred_audio_track(line: &str, stream_index: usize, stream_count: usize) -> AudioTrackKind {
  let stereo = line.contains("stereo");
  let mono = line.contains("mono");
  match (stream_count, stream_index) {
    (2, 0) => AudioTrackKind::SystemAudio,
    (2, 1) => AudioTrackKind::Microphone,
    (1, 0) if stereo => AudioTrackKind::SystemAudio,
    (1, 0) if mono => AudioTrackKind::Microphone,
    _ => AudioTrackKind::Unknown,
  }
}

#[cfg(test)]
mod inferred_audio_track_tests {
  use super::*;

  #[test]
  fn two_streams_are_system_audio_then_microphone() {
    let stereo = "Stream #0:1: Audio: aac, 48000 Hz, stereo, fltp";
    let mono = "Stream #0:2: Audio: aac, 48000 Hz, mono, fltp";
    assert_eq!(
      inferred_audio_track(stereo, 0, 2),
      AudioTrackKind::SystemAudio
    );
    assert_eq!(inferred_audio_track(mono, 1, 2), AudioTrackKind::Microphone);
  }

  #[test]
  fn a_lone_stream_is_named_by_its_channel_layout() {
    let stereo = "Stream #0:1: Audio: aac, 48000 Hz, stereo, fltp";
    let mono = "Stream #0:1: Audio: aac, 48000 Hz, mono, fltp";
    assert_eq!(
      inferred_audio_track(stereo, 0, 1),
      AudioTrackKind::SystemAudio
    );
    assert_eq!(inferred_audio_track(mono, 0, 1), AudioTrackKind::Microphone);
  }

  #[test]
  fn anything_else_stays_unknown() {
    let line = "Stream #0:1: Audio: aac, 48000 Hz, 5.1, fltp";
    assert_eq!(inferred_audio_track(line, 2, 3), AudioTrackKind::Unknown);
  }
}
