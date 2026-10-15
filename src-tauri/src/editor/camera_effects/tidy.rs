// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the camera's effects keep when the recording closes: the files the
//! effects and the background ranges as they stand use, and nothing else.
//! Turning an effect back on after that makes its file again.

use std::path::Path;

use super::{image, kept, make, matte, picks, project_folder};
use crate::editor::timeline_edit::saved_camera_background_ranges;
use screenwide_export::camera_effects::CameraBackground;
use screenwide_timeline_edit::CameraRangeBackground;

/// Files earlier makes of the effects wrote, which nothing reads now.
const RETIRED: &[&str] = &[
  "camera-background.png",
  "camera-composed.json",
  "camera-composed.mov",
  "camera-composed.mp4",
  "camera-cutout.mov",
  "camera-enhanced.mov",
];
const RETIRED_PREFIX: &str = "camera-composed-";

/// Removes the camera effects' files the project whose manifest is
/// `project` no longer uses.
pub(crate) fn on_close(project: &Path) {
  let folder = project_folder(project);
  let kept = kept(&folder);
  let ranges = saved_camera_background_ranges(project);
  let remove = |path: &Path| {
    let _ = std::fs::remove_file(path);
  };
  for name in RETIRED {
    remove(&folder.join(name));
  }
  if let Ok(entries) = std::fs::read_dir(&folder) {
    for path in entries.flatten().map(|entry| entry.path()) {
      if path
        .file_name()
        .is_some_and(|name| name.to_string_lossy().starts_with(RETIRED_PREFIX))
      {
        remove(&path);
      }
    }
  }
  if ranges.is_empty() {
    remove(&matte::path(&folder));
    remove(&matte::timings_path(&folder));
    matte::remove_chunks(&folder);
    picks::remove_composed(&folder, None);
  } else {
    let current = picks::composed(&folder);
    let name = current
      .as_deref()
      .and_then(Path::file_name)
      .and_then(|name| name.to_str());
    picks::remove_composed(&folder, name);
  }
  if !(kept.enhance || kept.colour) {
    remove(&make::measured_path(&folder));
  }
  // A range keeps its picture only while it shows it: with the editor
  // closed, nothing can undo it back to the picture.
  for id in image::chosen(&folder) {
    let shown = ranges
      .iter()
      .any(|range| range.id == id && range.background == CameraRangeBackground::Image);
    if !shown {
      remove(&image::path(&folder, &id));
    }
  }
  // The picture from before ranges is carried into a range when the
  // project opens; a project with no background left to carry has no use
  // for it.
  if kept.background == CameraBackground::Kept {
    remove(&image::legacy_path(&folder));
  }
}
