// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The colour grade: one gain and offset for each channel, the same for the
//! whole recording, so the camera never flickers between grades the way a
//! grade worked out frame by frame would.
//!
//! In front of a picture the person is drawn part of the way towards its
//! colour, brightness and contrast, as light in that place would light
//! them. With no picture the camera is balanced on its own: the room's cast
//! taken out, and the person brought to a steady exposure and contrast. Each
//! move is held to a fraction of the way and to a bound, so a picture of a
//! blue sky tints a face rather than painting it blue.

use std::io::Read;
use std::path::Path;
use std::process::Stdio;

/// Frames sampled across a recording to measure it.
const SAMPLES: u64 = 60;
/// The size each sampled frame is measured at.
const SAMPLE_SIZE: (usize, usize) = (160, 90);
/// Rec. 709 luma weights.
const LUMA: [f64; 3] = [0.2126, 0.7152, 0.0722];

/// Each channel's mean and spread, and the luma's, 0 to 1.
#[derive(Clone, Copy, Debug, serde::Deserialize, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Stats {
  pub mean: [f64; 3],
  pub luma_mean: f64,
  pub luma_spread: f64,
}

#[derive(Default)]
struct Accumulator {
  weight: f64,
  sum: [f64; 3],
  luma: f64,
  luma_squared: f64,
}

impl Accumulator {
  fn add(&mut self, rgb: [u8; 3], weight: f64) {
    if weight <= 0.0 {
      return;
    }
    let values = rgb.map(|value| f64::from(value) / 255.0);
    let luma = values
      .iter()
      .zip(LUMA)
      .map(|(value, weight)| value * weight)
      .sum::<f64>();
    self.weight += weight;
    for (sum, value) in self.sum.iter_mut().zip(values) {
      *sum += value * weight;
    }
    self.luma += luma * weight;
    self.luma_squared += luma * luma * weight;
  }

  /// Nothing where nothing was weighed, as a camera with no person in it.
  fn finish(&self) -> Option<Stats> {
    (self.weight > f64::EPSILON).then(|| {
      let luma_mean = self.luma / self.weight;
      Stats {
        mean: self.sum.map(|sum| sum / self.weight),
        luma_mean,
        luma_spread: (self.luma_squared / self.weight - luma_mean * luma_mean)
          .max(0.0)
          .sqrt(),
      }
    })
  }
}

/// `x * gain + offset` for each channel, 0 to 1, which the effects pass
/// applies to every pixel it grades.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Grade {
  pub gain: [f64; 3],
  pub offset: [f64; 3],
}

impl Grade {
  /// `rgb`, 0 to 255, graded.
  pub(crate) fn apply(&self, rgb: [u8; 3]) -> [u8; 3] {
    let mut graded = [0; 3];
    for (index, value) in graded.iter_mut().enumerate() {
      let x = f64::from(rgb[index]) / 255.0 * self.gain[index] + self.offset[index];
      *value = (x.clamp(0.0, 1.0) * 255.0).round() as u8;
    }
    graded
  }
}

/// Part of the way from 1 to `factor`.
fn towards_one(factor: f64, share: f64) -> f64 {
  1.0 + share * (factor - 1.0)
}

/// The camera balanced on its own: the cast of `frame` taken out, and the
/// `subject` brought to a steady exposure and contrast.
pub(super) fn balanced(frame: &Stats, subject: &Stats) -> Grade {
  let grey = frame.mean.iter().sum::<f64>() / 3.0;
  let balance = frame
    .mean
    .map(|mean| towards_one((grey / mean.max(0.01)).clamp(0.85, 1.15), 0.6));
  let exposure = towards_one((0.48 / subject.luma_mean.max(0.01)).clamp(0.85, 1.35), 0.7);
  let contrast = towards_one((0.2 / subject.luma_spread.max(0.01)).clamp(0.9, 1.2), 0.5);
  let pivot = subject.luma_mean * exposure;
  Grade {
    gain: balance.map(|balance| balance * exposure * contrast),
    offset: [pivot * (1.0 - contrast); 3],
  }
}

/// The `subject` drawn part of the way towards the `picture` behind it:
/// its brightness half the way, its colour a third, and its contrast to the
/// picture's square root.
pub(super) fn towards(subject: &Stats, picture: &Stats) -> Grade {
  let brightness = 0.5 * (picture.luma_mean - subject.luma_mean);
  let contrast = (picture.luma_spread / subject.luma_spread.max(0.01))
    .sqrt()
    .clamp(0.85, 1.2);
  let mut offset = [0.0; 3];
  for (index, offset) in offset.iter_mut().enumerate() {
    let tint = picture.mean[index] - subject.mean[index] - (picture.luma_mean - subject.luma_mean);
    let target = subject.mean[index] + brightness + 0.35 * tint;
    *offset = (target - contrast * subject.mean[index]).clamp(-0.2, 0.2);
  }
  Grade {
    gain: [contrast; 3],
    offset,
  }
}

/// The recording at `camera`, and where `matte` is given the person in it,
/// measured across frames sampled at the composing `rate`: the whole frame,
/// and the person weighed by how much of each pixel is them.
pub(super) fn measure(
  camera: &Path,
  matte: Option<&Path>,
  (rate, frames): (f64, u64),
) -> Result<(Stats, Option<Stats>), String> {
  let every = (frames / SAMPLES).max(1);
  let picture = sampled(camera, "rgb24", rate, every)?;
  let alpha = matte
    .map(|matte| sampled(matte, "gray", rate, every))
    .transpose()?;
  let mut frame = Accumulator::default();
  let mut person = Accumulator::default();
  for (index, pixel) in picture.as_chunks::<3>().0.iter().enumerate() {
    frame.add(*pixel, 1.0);
    if let Some(weight) = alpha.as_ref().and_then(|alpha| alpha.get(index)) {
      person.add(*pixel, f64::from(*weight) / 255.0);
    }
  }
  let frame = frame
    .finish()
    .ok_or_else(|| "The camera recording has no frames to measure".to_owned())?;
  Ok((frame, person.finish()))
}

/// A picture measured whole.
pub(super) fn measure_picture(picture: &::image::RgbaImage) -> Option<Stats> {
  let small = ::image::imageops::thumbnail(picture, SAMPLE_SIZE.0 as u32, SAMPLE_SIZE.1 as u32);
  let mut whole = Accumulator::default();
  for pixel in small.pixels() {
    whole.add([pixel.0[0], pixel.0[1], pixel.0[2]], 1.0);
  }
  whole.finish()
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

#[cfg(test)]
pub(super) fn stats(mean: [f64; 3], luma_spread: f64) -> Stats {
  Stats {
    mean,
    luma_mean: mean
      .iter()
      .zip(LUMA)
      .map(|(mean, weight)| mean * weight)
      .sum(),
    luma_spread,
  }
}
