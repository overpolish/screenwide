// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The start's check that every input it was asked for is still there.
//!
//! A start never goes ahead without an input the user chose. Recording an hour
//! without the camera, or at a size it was not set to, would ruin what the
//! recording was for, so a missing microphone or camera mode fails the start
//! and the user turns it off or chooses again.

use crate::fault::{self, Fault};
use crate::recording::StartRecordingOptions;

pub(in crate::recording) fn check_inputs(options: &StartRecordingOptions) -> Result<(), String> {
  if let Some(microphone_id) = &options.microphone_id {
    if fault::active(Fault::Microphone)
      || crate::recording_inputs::resolve_microphone(Some(microphone_id)).is_err()
    {
      return Err(
        "The selected microphone is no longer connected. Choose another microphone in the recording bar, or turn the microphone off."
          .to_owned(),
      );
    }
  }
  if let (Some(camera_id), Some(width), Some(height), Some(fps)) = (
    &options.camera_id,
    options.camera_width,
    options.camera_height,
    options.camera_fps,
  ) {
    crate::recording_inputs::require_camera_mode(camera_id, width, height, fps)?;
  }
  Ok(())
}
