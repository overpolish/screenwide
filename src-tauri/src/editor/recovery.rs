// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[path = "recovery/cleanup.rs"]
mod cleanup;
pub(super) use cleanup::{orphaned_recordings, sweep_cancelled_recordings};
#[path = "recovery/offer.rs"]
mod offer;
use offer::sweep_orphaned_recordings;

use super::{
  recording_sidecar::{
    cursor_for_recording, keyboard_for_recording, sweep_unclaimed_cursors,
    sweep_unclaimed_keyboards,
  },
  *,
};
use crate::shortcuts::diagnostics::record;
use serde_json::json;

/// What to do with the recordings found in the working directory at startup.
#[derive(Debug, Default, Eq, PartialEq)]
pub struct OrphanPlan {
  pub delete: Vec<PathBuf>,
  pub present: Option<PathBuf>,
}

/// Decides the fate of every recording left behind by a previous run.
///
/// A recording is only ever in the working directory because it was never
/// saved - the app quit, or crashed, between finishing and saving. The most
/// recent one is worth offering back, because it is almost certainly the one
/// that was on screen when that happened. Anything past its keeping age goes,
/// including the newest, so a machine that crashed a month ago does not
/// resurrect a recording nobody remembers making.
pub fn orphan_plan(entries: Vec<(PathBuf, SystemTime)>, now: SystemTime) -> OrphanPlan {
  let (fresh, stale): (Vec<_>, Vec<_>) = entries.into_iter().partition(|(_, modified)| {
    now
      .duration_since(*modified)
      .is_ok_and(|age| age <= ORPHAN_MAX_AGE)
      // A file stamped in the future has no believable age; keeping it is the
      // safer half of the guess.
      || modified > &now
  });

  OrphanPlan {
    delete: stale.into_iter().map(|(path, _)| path).collect(),
    present: fresh
      .into_iter()
      .max_by_key(|(_, modified)| *modified)
      .map(|(path, _)| path),
  }
}

/// Removes metadata sidecars whose recording is no longer there.
///
/// The sidecar describes one movie and is consumed the moment that movie is
/// finished or offered back, so any left next to nothing are the residue of a
/// crash or of a recording swept away for age.
pub(super) fn sweep_orphaned_meta(directory: &Path) {
  let Ok(entries) = std::fs::read_dir(directory) else {
    return;
  };
  for entry in entries.flatten() {
    let path = entry.path();
    let Some(stem) = path
      .file_name()
      .and_then(|name| name.to_str())
      .and_then(|name| name.strip_suffix(crate::recording::meta_sidecar::SUFFIX))
    else {
      continue;
    };
    let has_recording = WORKING_RECORDING_EXTENSIONS
      .iter()
      .any(|extension| path.with_file_name(format!("{stem}.{extension}")).exists());
    if !has_recording {
      let _ = std::fs::remove_file(path);
    }
  }
}

pub(super) fn camera_for_recording(recording: &Path) -> Option<PathBuf> {
  let name = recording.file_name()?.to_str()?;
  let suffix = name.strip_prefix("recording-")?;
  let camera = recording.with_file_name(format!("camera-{suffix}"));
  camera.is_file().then_some(camera)
}

pub(super) fn sweep_unclaimed_cameras(directory: &Path, keep: Option<&Path>) {
  let Ok(entries) = std::fs::read_dir(directory) else {
    return;
  };
  for entry in entries.flatten() {
    let path = entry.path();
    let is_camera = path
      .file_name()
      .and_then(|name| name.to_str())
      .is_some_and(|name| name.starts_with("camera-"));
    if is_camera && keep != Some(path.as_path()) {
      let _ = std::fs::remove_file(path);
    }
  }
}

/// Offers back the recording an earlier run never got to save.
///
/// Deliberately not the whole artifact: the duration lived in the frames,
/// which are long gone. The name and the file are what matter.
/// Deletes every preview derivative left in the working directory.
///
/// They exist only for as long as an artifact is on screen, so at startup
/// there is no such thing as one worth keeping: any that are there were
/// stranded by a crash, and each is a copy of a movie sitting in the app's own
/// data directory where nobody will ever look for it.
///
/// The match is on the name's prefix rather than its extension, which is what
/// makes it reach the `.part` files a mix encodes into as well: those are
/// named after the mix they were going to become, so an encode killed halfway
/// is reclaimed here without this needing to know anything about it.
pub(super) fn sweep_preview_files(directory: &Path) {
  let Ok(entries) = std::fs::read_dir(directory) else {
    return;
  };

  for entry in entries.flatten() {
    let path = entry.path();
    if media_preview::is_preview_file(&path) {
      let _ = std::fs::remove_file(path);
    }
  }
}

pub fn initialize(app: &AppHandle) {
  *app
    .state::<EditorState>()
    .recording_output
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner()) = load_recording_output(app);
  *app
    .state::<EditorState>()
    .screenshot_output
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner()) = load_screenshot_output(app);
  *app
    .state::<EditorState>()
    .screenshot_background_radius_percent
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner()) = load_screenshot_background_radius(app);
  *app
    .state::<EditorState>()
    .screenshot_radius_percent
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner()) = load_screenshot_radius(app);
  *app
    .state::<EditorState>()
    .cursor_effects
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner()) = load_cursor_effects(app);
  sweep_orphaned_recordings(app);
}
