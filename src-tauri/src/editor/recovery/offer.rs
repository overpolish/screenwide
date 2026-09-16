// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Offering an unsaved recording from an earlier run back to the user.

use super::*;

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
      // A recovered recording keeps whatever timeline sidecar it already
      // wrote; there is nothing live left to hand over.
      annotation_clips: Vec::new(),
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
    crate::shortcuts::diagnostics::recovery_offered(app);
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
