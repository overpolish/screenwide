// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The qualities the Export island compares and measures, read against
//! the recording they are of.

use super::*;

impl ExportInputs {
  /// The screen's and a separate camera's video at `look`, checked as the
  /// export checks the qualities it is given.
  pub(in crate::editor::preview) fn look_videos(
    &self,
    look: &crate::editor::preview::estimate_command::ExportLookQuality,
  ) -> Result<(VideoExportOptions, VideoExportOptions), String> {
    if look.compression > 4 || look.camera_compression > 4 {
      return Err("Compression must be between 0 and 4".to_owned());
    }
    validate_primary_resolution_scale(
      look.resolution_scale_percent,
      self.source_scale_percent,
      self.primary_kind,
    )?;
    validate_camera_resolution_scale(look.camera_resolution_scale_percent)?;
    Ok((
      VideoExportOptions {
        compression: look.compression,
        resolution_scale_percent: look.resolution_scale_percent,
        source_scale_percent: self.source_scale_percent,
      },
      VideoExportOptions {
        compression: look.camera_compression,
        resolution_scale_percent: look.camera_resolution_scale_percent,
        source_scale_percent: 100,
      },
    ))
  }

  /// The scale Original stands for: the capture's own, or the camera's
  /// where the recording is of the camera alone.
  pub(in crate::editor::preview) fn original_scale_percent(&self) -> u16 {
    match self.primary_kind {
      PrimaryRecordingKind::Camera => 100,
      _ => self.source_scale_percent.clamp(100, 400),
    }
  }
}
