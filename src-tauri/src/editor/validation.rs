// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use screenwide_project::PrimaryRecordingKind;

/// The choices the export offers: Original, three quarters and half of the
/// capture's own scale, rounded as the export window rounds them.
pub(super) fn validate_resolution_scale(selected: u16, source: u16) -> Result<(), String> {
  let source = u32::from(source.clamp(100, 400));
  let choices = [source, (source * 75 + 50) / 100, (source * 50 + 50) / 100];
  if !choices.contains(&u32::from(selected)) {
    return Err("The selected output resolution is not available for this recording".to_owned());
  }

  Ok(())
}

pub(super) fn validate_camera_resolution_scale(selected: u16) -> Result<(), String> {
  if ![50, 75, 100].contains(&selected) {
    return Err("The selected camera resolution is not available".to_owned());
  }

  Ok(())
}

pub(super) fn validate_primary_resolution_scale(
  selected: u16,
  source: u16,
  kind: PrimaryRecordingKind,
) -> Result<(), String> {
  match kind {
    PrimaryRecordingKind::Camera => validate_camera_resolution_scale(selected),
    PrimaryRecordingKind::Screen => validate_resolution_scale(selected, source),
    PrimaryRecordingKind::Audio => Ok(()),
  }
}
