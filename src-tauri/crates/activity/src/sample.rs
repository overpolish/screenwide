// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Sampling a recording's picture: FFmpeg decodes it and shrinks each tenth
//! of a second to a small grey frame, streamed so a recording of any length
//! is held a few seconds at a time.
//!
//! Decoding is what costs: every frame of the recording is decoded to reach
//! the ones sampled, since all but the key frames depend on the one before.
//! FFmpeg's own decoder spread over the cores ran far ahead of VideoToolbox
//! here: a 4.5-minute 3494×2256 H.264 recording in 6 s against 150 s, the
//! hardware path stalling on handing each full-size frame back.

use std::io::Read;
use std::path::Path;
use std::process::Stdio;

use crate::measure::{Measure, PictureReading, SAMPLE_SIZE};

/// How many frames a second are measured.
pub const SAMPLES_PER_SECOND: u32 = 10;

/// The readings of the picture of the video at `path`, one per sample.
pub fn read_picture(path: &Path) -> Result<Vec<PictureReading>, String> {
  let (width, height) = SAMPLE_SIZE;
  let mut child = screenwide_sidecar::ffmpeg_command()
    .args(["-hide_banner", "-nostdin", "-loglevel", "error", "-i"])
    .arg(path)
    .args(["-map", "0:v:0", "-an", "-vf"])
    .arg(format!(
      "fps={SAMPLES_PER_SECOND},scale={width}:{height}:flags=area,format=gray"
    ))
    .args(["-f", "rawvideo", "-"])
    .stdin(Stdio::null())
    .stdout(Stdio::piped())
    .stderr(Stdio::null())
    .spawn()
    .map_err(|error| format!("Could not read the picture: {error}"))?;
  let mut stdout = child
    .stdout
    .take()
    .ok_or_else(|| "Could not read the picture".to_owned())?;
  let mut measure = Measure::new(f64::from(SAMPLES_PER_SECOND));
  let mut readings = Vec::new();
  loop {
    let mut frame = vec![0; width * height];
    match stdout.read_exact(&mut frame) {
      Ok(()) => readings.push(measure.next(frame)),
      Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => break,
      Err(error) => return Err(format!("Could not read the picture: {error}")),
    }
  }
  let status = child
    .wait()
    .map_err(|error| format!("Could not read the picture: {error}"))?;
  if !status.success() {
    return Err("FFmpeg could not read the picture".to_owned());
  }
  Ok(readings)
}
