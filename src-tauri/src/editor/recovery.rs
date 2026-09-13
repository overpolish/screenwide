// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[path = "recovery/cleanup.rs"]
mod cleanup;
pub(super) use cleanup::{orphaned_recordings, sweep_cancelled_recordings};

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

pub(super) fn sweep_orphaned_recordings(app: &AppHandle) {
  let Ok(directory) = crate::recording::recordings_directory(app) else {
    return;
  };
  sweep_preview_files(&directory);
  sweep_cancelled_recordings(&directory);
  let plan = orphan_plan(orphaned_recordings(&directory), SystemTime::now());

  for path in plan.delete {
    timeline_edit::remove_for_recording(&path);
    let _ = std::fs::remove_file(path);
  }
  // After the deletions, so a swept recording takes its metadata with it, and
  // before the recovery below, which still needs the survivor's own.
  sweep_orphaned_meta(&directory);
  let Some(path) = plan.present else {
    sweep_unclaimed_cameras(&directory, None);
    sweep_unclaimed_cursors(&directory, None);
    sweep_unclaimed_keyboards(&directory, None);
    timeline_edit::sweep_unclaimed(&directory, None);
    return;
  };
  record("recovery_candidate_found", json!({}));
  let camera_path = camera_for_recording(&path);
  let cursor_path = cursor_for_recording(&path);
  let keyboard_path = keyboard_for_recording(&path);
  sweep_unclaimed_cameras(&directory, camera_path.as_deref());
  sweep_unclaimed_cursors(&directory, cursor_path.as_deref());
  sweep_unclaimed_keyboards(&directory, keyboard_path.as_deref());
  timeline_edit::sweep_unclaimed(&directory, Some(&path));

  let recorded_at = std::fs::metadata(&path)
    .and_then(|metadata| metadata.modified())
    .map_or_else(
      |_| chrono::Local::now(),
      chrono::DateTime::<chrono::Local>::from,
    );
  let suggested_file_stem = crate::screenshots::capture_file_stem(recorded_at.naive_local());
  // Written while the capture was running, and the only record of things the
  // container itself cannot say. A recording made before this existed, or one
  // whose sidecar did not survive, falls back to what the name implies.
  let meta = crate::recording::meta_sidecar::read(&path);
  let primary_kind = if let Some(meta) = &meta {
    meta.primary_kind
  } else if path
    .file_name()
    .and_then(|name| name.to_str())
    .is_some_and(|name| name.starts_with("audio-"))
  {
    crate::recording::PrimaryRecordingKind::Audio
  } else {
    crate::recording::PrimaryRecordingKind::Screen
  };
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  let recovered_info = media_preview::recording_info(&path);
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  if primary_kind == crate::recording::PrimaryRecordingKind::Screen
    && recovered_info.is_none_or(|info| info.width == 0 || info.height == 0)
  {
    // A screen recording without readable dimensions cannot produce a finite
    // preview layout. It is an incomplete container, not a recoverable movie.
    let _ = std::fs::remove_file(&path);
    timeline_edit::remove_for_recording(&path);
    crate::recording::meta_sidecar::remove(&path);
    if let Some(path) = camera_path {
      let _ = std::fs::remove_file(path);
    }
    if let Some(path) = cursor_path {
      let _ = std::fs::remove_file(path);
    }
    if let Some(path) = keyboard_path {
      let _ = std::fs::remove_file(path);
    }
    return;
  }
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  let (duration_ms, width, height) = recovered_info.map_or((0, 0, 0), |info| {
    (info.duration_ms, info.width, info.height)
  });
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  let recovered_camera = camera_path.map(|path| {
    let info = media_preview::recording_info(&path);
    crate::recording::CameraFinalizeInfo {
      duration_ms: info.map_or(0, |value| value.duration_ms),
      height: info.map_or(0, |value| value.height),
      path,
      width: info.map_or(0, |value| value.width),
    }
  });
  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  let (duration_ms, width, height) = (0, 0, 0);
  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  let recovered_camera = camera_path.map(|path| crate::recording::CameraFinalizeInfo {
    duration_ms: 0,
    height: 0,
    path,
    width: 0,
  });
  if let Err(error) = present_recording(
    app,
    FinalizeInfo {
      camera: recovered_camera,
      cursor_path,
      keyboard_path,
      has_microphone: meta.as_ref().is_some_and(|meta| meta.has_microphone),
      has_system_audio: meta.as_ref().is_some_and(|meta| meta.has_system_audio),
      duration_ms,
      height,
      path: path.clone(),
      primary_kind,
      source_scale_factor: meta
        .as_ref()
        .map(|meta| meta.source_scale_factor)
        // A nonsense factor would size the export list off a division by
        // something impossible. Without a sidecar, from before there was
        // one, the main display's scale is the likeliest reading: a
        // recording is nearly always made on the Mac that recovers it.
        .filter(|factor| factor.is_finite() && *factor > 0.0)
        .unwrap_or_else(|| display_scale_factor(app)),
      width,
    },
    suggested_file_stem,
  ) {
    record("recovery_offer_failed", json!({"error": error.to_string()}));
    eprintln!("Could not offer back an unsaved recording: {error}");
  } else {
    record("recovery_offered", json!({}));
    // Its values now live in the artifact the editor holds. Unlike the cursor
    // and keyboard sidecars, which the editor goes on reading from disk, this
    // one has nothing left to say.
    crate::recording::meta_sidecar::remove(&path);
  }
}

fn display_scale_factor(app: &AppHandle) -> f32 {
  app
    .primary_monitor()
    .ok()
    .flatten()
    .map_or(1.0, |monitor| monitor.scale_factor() as f32)
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
