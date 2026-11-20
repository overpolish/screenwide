// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The restored voice given a studio microphone's finish, as a broadcast
//! chain gives it: warmth in the chest, a touch of body, less of the boxy
//! middle, and air on top; then its "s" sounds softened, a gentle 2:1
//! compression, and its peaks held below clipping. The chain follows
//! CrispVoice's (Francium Tech, MIT), fitted against a commercial enhancer
//! and kept after listening tests on headset recordings. The compression
//! takes the voice down a few decibels, so the finished voice is brought
//! back to the level it was recorded at, and the switch changes how it
//! sounds, not how loud it is; loudness itself is Auto volume's.

use std::path::Path;

use screenwide_transcriber::CLEAN_SAMPLE_RATE as RATE;

use super::level;
use crate::runner::samples::Samples;

/// The limiter, written once for [`CHAIN`] and [`LIMITER`], since `concat!`
/// takes only literals.
macro_rules! limiter {
  () => {
    "alimiter=limit=-1dB:attack=5:release=50:level=0:latency=1"
  };
}

/// What FFmpeg does to the voice, in order. The limiter's lookahead is made
/// up for, so the voice stays lined up with the picture.
const CHAIN: &str = concat!(
  "bass=g=3.5:f=160,",
  "equalizer=f=400:t=q:w=1:g=1.5,",
  "equalizer=f=280:t=q:w=1:g=-1.5,",
  "equalizer=f=9000:t=q:w=1:g=2.5,",
  "equalizer=f=10000:t=q:w=1:g=1,",
  "deesser,",
  "acompressor=threshold=-22dB:ratio=2:attack=10:release=150,",
  limiter!(),
);
/// Peaks held below clipping, here and after the level is brought back.
const LIMITER: &str = limiter!();

/// Gives the voice in `voice`, at the cleaning rate, its finish, and brings
/// it to speak at `recorded_level` (see `level::voiced`), written as FLAC to
/// `output`.
pub(super) fn master(voice: &Samples, recorded_level: f32, output: &Path) -> Result<(), String> {
  let finished = Samples::scratch();
  ffmpeg(voice, CHAIN, &["-f", "f32le"], finished.path())?;
  let level = level::voiced(finished.path(), RATE)?;
  let gain = if level > 0.0 && recorded_level > 0.0 {
    recorded_level / level
  } else {
    1.0
  };
  ffmpeg(
    &finished,
    &format!("volume={gain},{LIMITER}"),
    &["-c:a", "flac", "-sample_fmt", "s16"],
    output,
  )
}

/// Runs the mono samples in `input`, at the cleaning rate, through
/// `filters` into `output`, written as `format` says.
fn ffmpeg(input: &Samples, filters: &str, format: &[&str], output: &Path) -> Result<(), String> {
  let finished = screenwide_sidecar::ffmpeg_command()
    .args(["-nostdin", "-loglevel", "error", "-y", "-f", "f32le", "-ar"])
    .arg(RATE.to_string())
    .args(["-ac", "1", "-i"])
    .arg(input.path())
    .args(["-af", filters])
    .args(format)
    .arg(output)
    .output()
    .map_err(|error| format!("Could not start FFmpeg: {error}"))?;
  if finished.status.success() {
    Ok(())
  } else {
    Err(format!(
      "Could not finish the restored voice: {}",
      String::from_utf8_lossy(&finished.stderr).trim()
    ))
  }
}
