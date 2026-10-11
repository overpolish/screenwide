// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Enhance's colour correction: the recording's white balance and exposure,
//! worked out once from frames sampled across it (`balance`), so the camera
//! never flickers between corrections the way one worked out frame by frame
//! would, with the Natural look on top (`look`).

use std::io::Read;
use std::path::Path;
use std::process::Stdio;

use super::balance;
use super::look;

/// Frames sampled across a recording to measure it.
const SAMPLES: u64 = 60;
/// The size each sampled frame is measured at: a third of the 960-wide
/// frames the balance's pixel counts and blur were set on.
const SAMPLE_SIZE: (usize, usize) = (320, 180);

/// The colour correction as the effects pass applies it to every pixel it
/// grades.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Grade {
  /// The recording's correction, in linear light.
  pub gains: [f32; 3],
}

impl Grade {
  /// The grade for a recording measured to need `gains`.
  pub(super) fn new(gains: [f64; 3]) -> Self {
    Self {
      gains: gains.map(|gain| gain as f32),
    }
  }

  /// `rgb`, 0 to 255, graded.
  pub(crate) fn apply(&self, rgb: [u8; 3]) -> [u8; 3] {
    let graded = look::corrected(rgb.map(|value| f32::from(value) / 255.0), self.gains);
    graded.map(|value| (value * 255.0).round() as u8)
  }
}

/// The gains that correct the recording at `camera`, judged with the person
/// where `matte` is given, from frames sampled at the composing `rate`.
pub(super) fn measure(
  camera: &Path,
  matte: Option<&Path>,
  (rate, frames): (f64, u64),
) -> Result<[f64; 3], String> {
  let every = (frames / SAMPLES).max(1);
  let picture = sampled(camera, "rgb24", rate, every)?;
  if picture.is_empty() {
    return Err("The camera recording has no frames to measure".to_owned());
  }
  let alpha = matte
    .map(|matte| sampled(matte, "gray", rate, every))
    .transpose()?;
  Ok(balance::correction(&picture, alpha.as_deref(), SAMPLE_SIZE))
}

/// Every `every`th frame of the video at `path`, read at `rate` as the
/// composing reads it and shrunk to the sample size, as raw `format` bytes
/// back to back.
fn sampled(path: &Path, format: &str, rate: f64, every: u64) -> Result<Vec<u8>, String> {
  let (width, height) = SAMPLE_SIZE;
  let mut child = crate::editor::media_preview::ffmpeg_command()
    .args(["-hide_banner", "-nostdin", "-loglevel", "error", "-i"])
    .arg(path)
    .args(["-map", "0:v:0", "-an", "-vf"])
    .arg(format!(
      "fps={rate},select='not(mod(n\\,{every}))',scale={width}:{height}:flags=area"
    ))
    .args([
      "-fps_mode",
      "passthrough",
      "-f",
      "rawvideo",
      "-pix_fmt",
      format,
      "-",
    ])
    .stdin(Stdio::null())
    .stdout(Stdio::piped())
    .stderr(Stdio::null())
    .spawn()
    .map_err(|error| format!("Could not measure the camera: {error}"))?;
  let mut bytes = Vec::new();
  child
    .stdout
    .take()
    .ok_or_else(|| "Could not measure the camera".to_owned())?
    .read_to_end(&mut bytes)
    .map_err(|error| format!("Could not measure the camera: {error}"))?;
  let status = child
    .wait()
    .map_err(|error| format!("Could not measure the camera: {error}"))?;
  if !status.success() {
    return Err("FFmpeg could not measure the camera".to_owned());
  }
  Ok(bytes)
}
