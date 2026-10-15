// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The camera's background ranges are part of the edit, so the edit being
//! saved is what turns background removal on, off, or over another stretch.
//! What the ranges need is made, the work for ranges taken away stops, and
//! the camera is drawn with them.

use tauri::AppHandle;

use super::commands::{open, shown_changed, spawn_ready};
use super::stop::{self, Work};
use super::{choice, make, matte};
use screenwide_timeline_edit::{
  CameraBackgroundRange, RecordingTimelineEdit, RecordingTimelineSegment,
};

/// Follows the edit of recording `artifact_id` going from `before` to an
/// edit with the background ranges and segments given in its project,
/// where that changes where its background is taken away: the ranges
/// themselves, or the timeline they are timed on.
pub(crate) fn edit_saved(
  app: &AppHandle,
  artifact_id: u64,
  before: Option<&RecordingTimelineEdit>,
  (ranges, segments): (&[CameraBackgroundRange], &[RecordingTimelineSegment]),
) {
  let ranges_before = before.map_or(&[][..], |edit| &edit.camera_background_ranges[..]);
  let changed = ranges_before != ranges
    || (!ranges.is_empty() && before.is_none_or(|before| before.segments != segments));
  if !changed {
    return;
  }
  let Ok(camera) = open(app, artifact_id) else {
    return;
  };
  let folder = camera.folder();
  if ranges.is_empty() {
    for work in [Work::Matte, Work::Following, Work::Composing] {
      stop::stop(&folder, work);
    }
    shown_changed(app, artifact_id, &camera);
    return;
  }
  // Matting chunks the ranges no longer cover stops, and starts again on
  // what they cover now; chunks they newly cover are made after the ones
  // under way.
  if matte::making_unneeded(&camera) {
    stop::stop(&folder, Work::Matte);
    spawn_ready(app, artifact_id, camera.clone());
  } else if make::has_work(app, &camera, &choice(&folder)) {
    spawn_ready(app, artifact_id, camera.clone());
  }
  shown_changed(app, artifact_id, &camera);
}
