// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// Where a package manager puts FFmpeg and its siblings. A bundled app
/// inherits its environment from launchd rather than from a shell, so `PATH`
/// alone finds nothing that was installed by Homebrew or MacPorts.
#[cfg(not(windows))]
const TOOL_SEARCH_DIRECTORIES: &[&str] = &[
  "/opt/homebrew/bin",
  "/usr/local/bin",
  "/opt/local/bin",
  "/usr/bin",
];

#[cfg(windows)]
const TOOL_SEARCH_DIRECTORIES: &[&str] = &[];

pub(super) fn tool_path(name: &str) -> PathBuf {
  let executable = if cfg!(windows) {
    format!("{name}.exe")
  } else {
    name.to_owned()
  };
  // Alongside the app first: a bundled copy is the one this build was tested
  // against.
  if let Ok(current) = std::env::current_exe() {
    if let Some(directory) = current.parent() {
      let bundled = directory.join(&executable);
      if bundled.is_file() {
        return bundled;
      }
    }
  }
  for directory in TOOL_SEARCH_DIRECTORIES {
    let path = Path::new(directory).join(&executable);
    if path.is_file() {
      return path;
    }
  }

  PathBuf::from(executable)
}

pub(crate) fn ffmpeg_path() -> PathBuf {
  tool_path("ffmpeg")
}

/// FFprobe is only ever used to *check* a file this module just wrote, never
/// to produce one. Everything here still works without it - see
/// [`plays_from_start_to_end`] for what is given up when it is missing.
pub(super) fn ffprobe_path() -> PathBuf {
  tool_path("ffprobe")
}

pub fn inspect_audio_tracks(source: &Path) -> Result<Vec<RecordingAudioTrack>, String> {
  // FFmpeg prints container metadata before it complains that no output was
  // supplied. That gives recovery everything it needs without shipping a
  // second, almost equally large FFprobe executable.
  let output = Command::new(ffmpeg_path())
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
        let (kind, label) = inferred_audio_track(line, stream_index, streams.len());
        RecordingAudioTrack {
          kind,
          label: match kind {
            AudioTrackKind::Unknown => format!("Audio {}", stream_index + 1),
            _ => label.to_owned(),
          },
          stream_index,
        }
      })
      .collect(),
  )
}

/// Names a stream from what the recorder always writes: system audio first
/// and in stereo, the microphone after it and in mono. A recording from
/// before the metadata sidecar has nothing else to say which is which.
fn inferred_audio_track(
  line: &str,
  stream_index: usize,
  stream_count: usize,
) -> (AudioTrackKind, &'static str) {
  let stereo = line.contains("stereo");
  let mono = line.contains("mono");
  match (stream_count, stream_index) {
    (2, 0) => (AudioTrackKind::SystemAudio, "System audio"),
    (2, 1) => (AudioTrackKind::Microphone, "Microphone"),
    (1, 0) if stereo => (AudioTrackKind::SystemAudio, "System audio"),
    (1, 0) if mono => (AudioTrackKind::Microphone, "Microphone"),
    _ => (AudioTrackKind::Unknown, "Audio"),
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
      inferred_audio_track(stereo, 0, 2).0,
      AudioTrackKind::SystemAudio
    );
    assert_eq!(
      inferred_audio_track(mono, 1, 2).0,
      AudioTrackKind::Microphone
    );
  }

  #[test]
  fn a_lone_stream_is_named_by_its_channel_layout() {
    let stereo = "Stream #0:1: Audio: aac, 48000 Hz, stereo, fltp";
    let mono = "Stream #0:1: Audio: aac, 48000 Hz, mono, fltp";
    assert_eq!(
      inferred_audio_track(stereo, 0, 1).0,
      AudioTrackKind::SystemAudio
    );
    assert_eq!(
      inferred_audio_track(mono, 0, 1).0,
      AudioTrackKind::Microphone
    );
  }

  #[test]
  fn anything_else_stays_unknown() {
    let line = "Stream #0:1: Audio: aac, 48000 Hz, 5.1, fltp";
    assert_eq!(inferred_audio_track(line, 2, 3).0, AudioTrackKind::Unknown);
  }
}
