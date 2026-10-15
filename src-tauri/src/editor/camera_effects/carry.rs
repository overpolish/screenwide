// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A project kept before backgrounds had ranges took the camera's background
//! away for the whole recording, as one choice beside the edit. Opened now,
//! that choice becomes one range over the whole edited video, its picture
//! the range's own.

use super::{image, keep, kept, CameraRecording, Choice};
use crate::editor::timeline_edit;
use crate::editor::EditorArtifact;
use screenwide_export::camera_effects::CameraBackground;
use screenwide_timeline_edit::{CameraBackgroundRange, CameraRangeBackground, TimelinePlan};

/// The id the carried range takes, which names its picture.
const CARRIED_ID: &str = "carried";

/// Carries the whole-recording background of `artifact`'s project, where it
/// has one, into its edit as a range. Done once: the choice no longer holds
/// a background afterwards.
pub(crate) fn carry_background(artifact: &EditorArtifact) {
  let EditorArtifact::Recording { id, .. } = artifact else {
    return;
  };
  let Some(camera) = CameraRecording::of(artifact) else {
    return;
  };
  if let Err(error) = carry(&camera, *id) {
    eprintln!("Could not carry the camera's background onto its timeline: {error}");
  }
}

pub(super) fn carry(camera: &CameraRecording, artifact_id: u64) -> Result<(), String> {
  let folder = camera.folder();
  let kept = kept(&folder);
  let background = match kept.background {
    CameraBackground::Kept => return Ok(()),
    CameraBackground::Transparent => CameraRangeBackground::Transparent,
    CameraBackground::Blurred => CameraRangeBackground::Blurred,
    CameraBackground::Image => CameraRangeBackground::Image,
  };
  let (revision, mut edit) =
    timeline_edit::for_project(&camera.project, artifact_id, camera.recording_ms)
      .unwrap_or_else(|| (0, timeline_edit::untouched(artifact_id)));
  // An edit that has ranges already was made since; the old choice is only
  // what is left behind.
  if edit.camera_background_ranges.is_empty() {
    let end_ms = TimelinePlan::from_edit(&edit, camera.recording_ms)
      .map_or(camera.recording_ms, |plan| plan.duration_ms());
    if end_ms > 0 {
      if background == CameraRangeBackground::Image {
        let legacy = image::legacy_path(&folder);
        if legacy.is_file() {
          std::fs::rename(&legacy, image::path(&folder, CARRIED_ID))
            .map_err(|error| format!("Could not keep the background picture: {error}"))?;
        }
      }
      edit.camera_background_ranges = vec![CameraBackgroundRange {
        id: CARRIED_ID.to_owned(),
        start_ms: 0,
        end_ms,
        background,
      }];
      timeline_edit::persist(&camera.project, artifact_id, revision + 1, edit)?;
    }
  }
  keep(&folder, &Choice::from(kept))
}
