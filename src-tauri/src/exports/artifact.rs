// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn snapshots(app: &AppHandle) -> ExportSnapshots {
  ExportSnapshots {
    recording: snapshot(app, ExportKind::Recording),
    screenshot: snapshot(app, ExportKind::Screenshot),
  }
}

/// Broadcast rather than sent to the owning window: the recording bar tracks
/// what is waiting too. Receivers route the payload by its `workspace`.
pub(super) fn emit_snapshot(app: &AppHandle, kind: ExportKind) {
  let _ = app.emit(EXPORT_CHANGED_EVENT, snapshot(app, kind));
}

pub(super) fn delete_working_file(artifact: &ExportArtifact) {
  if let ExportArtifact::Recording {
    camera,
    cursor,
    keyboard,
    path,
    ..
  } = artifact
  {
    let _ = std::fs::remove_file(path);
    if let Some(camera) = camera {
      let _ = std::fs::remove_file(&camera.path);
    }
    recording_sidecar::remove_working_files(cursor.as_ref(), keyboard.as_ref());
    timeline_edit::remove_for_recording(path);
  }
}

/// Removes everything built for the artifact that is going away.
///
/// Every path that lets go of a recording - discarding it, replacing it with a
/// new capture, saving it - comes through here, so no derivative outlives the
/// artifact it was made from.
pub(super) fn clear_recording_preview(app: &AppHandle) {
  super::recording_preview_player::stop_all(app);
  let state = app.state::<ExportState>();
  state
    .recording_preview
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .take();
  state
    .compression_estimates
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .clear();
}

/// The next artifact identity. Two consecutive captures are otherwise
/// indistinguishable, and the window needs to tell them apart.
pub(super) fn next_id(app: &AppHandle) -> u64 {
  app
    .state::<ExportState>()
    .generation
    .fetch_add(1, Ordering::SeqCst)
    .wrapping_add(1)
}

/// Puts a new artifact in front of the user. Admission is checked before any
/// state is changed, so this path can never silently replace unsaved work.
pub(super) fn present_new(app: &AppHandle, artifact: ExportArtifact) -> Result<(), String> {
  let kind = ExportKind::of(&artifact);
  // Only the recording workspace owns a preview, and only its own arrival
  // retires one: a screenshot must not tear down a recording waiting next door.
  if kind == ExportKind::Recording {
    clear_recording_preview(app);
  }
  {
    let state = app.state::<ExportState>();
    let slot = state.slot(kind);
    let mut artifact_slot = slot
      .artifact
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if artifact_slot.is_some() {
      drop(artifact_slot);
      workspace::focus_pending(app, kind);
      return Err("An export workspace is already open".to_owned());
    }
    let mut reservation = state
      .capture_reservation
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let defaults = crate::settings::current(app);
    let default_directory = match &artifact {
      ExportArtifact::Screenshot { .. } => defaults.screenshot_directory,
      ExportArtifact::Recording { .. } => defaults.recording_directory,
    }
    .or_else(|| crate::screenshots::screenshot_directory(app).ok());
    *slot
      .directory
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner()) = default_directory;
    *artifact_slot = Some(artifact);
    *reservation = None;
  }

  if let Err(error) = window::show(app, kind) {
    // A hidden artifact is a deadlocked workspace. Keep a recording's file on
    // disk so startup recovery can offer it again, but release the in-memory
    // admission state so the current app remains usable.
    let state = app.state::<ExportState>();
    *state
      .slot(kind)
      .artifact
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
    *state
      .capture_reservation
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
    emit_snapshot(app, kind);
    return Err(error.to_string());
  }
  // Once an artifact is safely in front of the user, the capture controls
  // have finished their job. Keeping this at the shared presentation boundary
  // gives screenshots and recordings the same handoff without affecting
  // clipboard-only screenshots, which never open the export window.
  let _ = crate::windows::hide_recording_ui(app.clone());
  emit_snapshot(app, kind);

  Ok(())
}

/// Hands a freshly captured still to the export window.
pub fn present_screenshot(
  app: &AppHandle,
  image: CapturedImage,
  suggested_file_stem: String,
) -> Result<(), String> {
  let item = ScreenshotItem {
    id: next_id(app),
    image,
  };

  {
    let state = app.state::<ExportState>();
    let mut artifact = state
      .screenshot
      .artifact
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(ExportArtifact::Screenshot { items, .. }) = artifact.as_mut() {
      items.push(item);
      *state
        .capture_reservation
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
      drop(artifact);
      window::show(app, ExportKind::Screenshot).map_err(|error| error.to_string())?;
      let _ = crate::windows::hide_recording_ui(app.clone());
      emit_snapshot(app, ExportKind::Screenshot);
      return Ok(());
    }
  }

  present_new(
    app,
    ExportArtifact::Screenshot {
      id: next_id(app),
      items: vec![item],
      suggested_file_stem,
    },
  )
}

/// Hands a finished recording to the export window.
///
/// Mirrors `present_screenshot`. The window previews the movie itself through
/// the native preview surface, so nothing still-image is carried here.
pub fn present_recording(
  app: &AppHandle,
  info: FinalizeInfo,
  suggested_file_stem: String,
) -> Result<(), String> {
  let FinalizeInfo {
    camera,
    cursor_path,
    keyboard_path,
    has_microphone,
    has_system_audio,
    duration_ms,
    height,
    path,
    primary_kind,
    source_scale_factor,
    width,
  } = info;

  let mut audio_tracks = recording_audio_tracks(has_system_audio, has_microphone);
  if audio_tracks.is_empty() {
    audio_tracks = media_preview::inspect_audio_tracks(&path).unwrap_or_default();
  }

  present_new(
    app,
    ExportArtifact::Recording {
      id: next_id(app),
      audio_tracks,
      camera: camera.map(|camera| {
        let camera_duration_ms = (camera.duration_ms > 0)
          .then_some(camera.duration_ms)
          .or_else(|| media_preview::duration_ms(&camera.path))
          .unwrap_or(duration_ms);
        RecordingCamera {
          duration_ms: camera_duration_ms,
          height: camera.height,
          original_size_bytes: std::fs::metadata(&camera.path).map_or(0, |metadata| metadata.len()),
          path: camera.path,
          width: camera.width,
        }
      }),
      cursor: cursor_path.map(RecordingCursor::new),
      keyboard: keyboard_path.map(RecordingKeyboard::new),
      duration_ms,
      height,
      path,
      primary_kind,
      source_scale_percent: scale_percent(source_scale_factor),
      suggested_file_stem,
      width,
    },
  )
}

pub(super) fn take_artifact(app: &AppHandle, kind: ExportKind) -> Option<ExportArtifact> {
  let state = app.state::<ExportState>();
  let artifact = state
    .slot(kind)
    .artifact
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .take();

  artifact
}

/// Drops one workspace's pending artifact and puts its window away. Cancelling
/// and closing that window are the same act; the other workspace is untouched.
pub fn discard(app: &AppHandle, kind: ExportKind) {
  if kind == ExportKind::Recording {
    clear_recording_preview(app);
  }
  if let Some(artifact) = take_artifact(app, kind) {
    delete_working_file(&artifact);
  }
  let _ = window::hide(app, kind);
  emit_snapshot(app, kind);
}
