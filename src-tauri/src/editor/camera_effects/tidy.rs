// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the camera's effects keep when the recording closes: the files the
//! effects as they stand use, and nothing else. Turning an effect back on
//! after that makes its file again.

use std::path::Path;

use super::{choice, image, keep, make, matte, picks, project_folder, CameraBackground};

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
  let effects = choice(&folder);
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
  if effects.removes_background() {
    let current = picks::composed(&folder);
    let name = current
      .as_deref()
      .and_then(Path::file_name)
      .and_then(|name| name.to_str());
    picks::remove_composed(&folder, name);
  } else {
    remove(&matte::path(&folder));
    remove(&matte::timings_path(&folder));
    picks::remove_composed(&folder, None);
  }
  if !effects.enhance {
    remove(&make::measured_path(&folder));
  }
  if effects.background != CameraBackground::Image {
    remove(&image::path(&folder));
    if effects.has_image {
      let _ = keep(
        &folder,
        &super::CameraEffects {
          has_image: false,
          ..effects
        },
      );
    }
  }
}
