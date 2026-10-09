// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Audio handed to and from the transcriber: scratch files of 32-bit float
//! samples, made from recordings by FFmpeg.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use screenwide_transcriber::SAMPLE_RATE;

/// Audio as the transcriber takes it, in a scratch file removed with this.
pub(crate) struct Samples(pub(super) PathBuf);

impl Drop for Samples {
  fn drop(&mut self) {
    let _ = std::fs::remove_file(&self.0);
  }
}

impl Samples {
  /// A scratch file for the transcriber to write into.
  pub(crate) fn scratch() -> Self {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    Self(std::env::temp_dir().join(format!(
      "screenwide-transcription-{}-{}.f32",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    )))
  }

  pub(crate) fn path(&self) -> &Path {
    &self.0
  }

  /// The samples themselves, to look at before they are transcribed.
  pub(crate) fn read(&self) -> Result<Vec<f32>, String> {
    let bytes = std::fs::read(&self.0)
      .map_err(|error| format!("Could not read the audio to transcribe: {error}"))?;
    Ok(
      bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|bytes| f32::from_le_bytes(*bytes))
        .collect(),
    )
  }
}

/// `source` converted by FFmpeg into what the transcriber reads: mono 32-bit
/// float samples at its rate, in little-endian order, which is the native
/// one on every computer Screenwide runs on.
pub(in crate::transcription) fn decode(source: &Path) -> Result<Samples, String> {
  decode_with(source, &[], (1, SAMPLE_RATE))
}

/// The `stream`th audio track of `source`, converted as [`decode`] converts
/// a voice note but at `rate`, and padded or trimmed to start where the
/// recording starts so a moment in it is the same moment in the picture.
pub(crate) fn decode_audio_stream(
  source: &Path,
  stream: usize,
  rate: u32,
) -> Result<Samples, String> {
  decode_stream(source, stream, (1, rate))
}

/// The `stream`th audio track of `source` as [`decode_audio_stream`] has it,
/// but with both its sides, interleaved, as separating music takes it.
pub(crate) fn decode_stereo_stream(
  source: &Path,
  stream: usize,
  rate: u32,
) -> Result<Samples, String> {
  decode_stream(source, stream, (2, rate))
}

fn decode_stream(source: &Path, stream: usize, form: (u8, u32)) -> Result<Samples, String> {
  decode_with(
    source,
    &[
      "-map",
      &format!("0:a:{stream}"),
      "-af",
      "aresample=async=1:first_pts=0",
    ],
    form,
  )
}

fn decode_with(
  source: &Path,
  map: &[&str],
  (channels, rate): (u8, u32),
) -> Result<Samples, String> {
  let samples = Samples::scratch();
  let output = crate::editor::ffmpeg_command()
    .args(["-nostdin", "-loglevel", "error", "-y", "-i"])
    .arg(source)
    .args(map)
    .args([
      "-ac",
      &channels.to_string(),
      "-ar",
      &rate.to_string(),
      "-f",
      "f32le",
    ])
    .arg(&samples.0)
    .output()
    .map_err(|error| format!("Could not start FFmpeg: {error}"))?;
  if !output.status.success() {
    return Err(format!(
      "Could not read the audio to transcribe: {}",
      String::from_utf8_lossy(&output.stderr).trim()
    ));
  }
  Ok(samples)
}
