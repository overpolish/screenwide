// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The recording's video file, encoded by the GPU's H.264 encoder.

use screenwide_h264::{FileEncoder, RateControl, Settings};

use super::*;

/// Half-second fragments align with the half-second keyframes, bounding what
/// a crash loses while keeping seeks cheap in the export scrubber.
const FRAGMENT_100NS: u64 = 5_000_000;

impl Sink {
  pub(super) fn new(config: &WriterConfig) -> Result<Self, String> {
    let fps = config.fps.max(1);
    let settings = Settings {
      width: config.width,
      height: config.height,
      fps,
      bitrate: u32::try_from(bitrate_bps(config.width, config.height, fps)).unwrap_or(u32::MAX),
      rate_control: RateControl::Average,
      // The export scrubber seeks a working recording keyframe to keyframe.
      keyframe_interval: (fps / 2).max(1),
      low_latency: true,
      purpose: "Recording",
    };
    let file = FileEncoder::create(
      &config.device,
      settings,
      None,
      &config.path,
      Some(FRAGMENT_100NS),
    )?;
    Ok(Self { file: Some(file) })
  }

  pub(super) fn write(
    &mut self,
    frame: &Frame,
    pts_100ns: i64,
    duration_100ns: i64,
  ) -> Result<(), String> {
    self
      .file
      .as_mut()
      .ok_or_else(|| "The recording has already been finalized".to_owned())?
      .write(&frame.texture, pts_100ns, duration_100ns)
  }

  pub(super) fn finish(&mut self) -> Result<(), String> {
    self
      .file
      .take()
      .ok_or_else(|| "The recording has already been finalized".to_owned())?
      .finish()
  }
}
