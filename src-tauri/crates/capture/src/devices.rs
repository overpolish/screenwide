// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Naming and opening the microphone and camera a recording asked for.

use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, SampleFormat, StreamConfig};
use nokhwa::utils::CameraInfo;

/// The microphone `device_id` names, or the default one, with the format it
/// records in.
pub fn resolve_microphone(
  device_id: Option<&str>,
) -> Result<(Device, StreamConfig, SampleFormat), String> {
  let host = cpal::default_host();
  let device = match device_id {
    Some(device_id) => host
      .input_devices()
      .map_err(|error| error.to_string())?
      .find(|device| {
        device
          .id()
          .is_ok_and(|candidate| candidate.to_string() == device_id)
      })
      .ok_or_else(|| "The selected microphone is no longer available".to_owned())?,
    None => host
      .default_input_device()
      .ok_or_else(|| "No default microphone is available".to_owned())?,
  };
  let config = device
    .default_input_config()
    .map_err(|error| error.to_string())?;
  let sample_format = config.sample_format();
  Ok((device, config.into(), sample_format))
}

/// A camera's stable id: its backend's, or its index where it has none.
pub fn camera_id(camera: &CameraInfo) -> String {
  let backend_id = camera.misc();
  if backend_id.trim().is_empty() {
    camera.index().as_string()
  } else {
    backend_id
  }
}
