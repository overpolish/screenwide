// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Putting a new capture, or a recording recovered at launch, in front of the
//! user in its editor workspace.

use super::artifact::{clear_recording_preview, next_id};
use super::*;

/// When [`present_new`] shows the artifact it installed.
pub(super) enum Reveal {
  /// Here and now.
  Now,
  /// From a worker thread, as a live capture's arrives. Startup recovery
  /// presents during setup, on the main thread, and a snapshot emitted there
  /// never reached the editor's webview: in a release build that page has
  /// loaded and fetched an empty snapshot before setup runs, so the editor
  /// waited for good. From a worker, the emit and the show are posted to the
  /// event loop, which runs them once setup has returned.
  Deferred,
}

/// Puts a new artifact in front of the user. Admission is checked before any
/// state is changed, so this path can never silently replace unsaved work.
pub(super) fn present_new(
  app: &AppHandle,
  artifact: EditorArtifact,
  reveal_when: Reveal,
) -> Result<(), String> {
  let kind = EditorKind::of(&artifact);
  // Only the recording workspace owns a preview, and only its own arrival
  // retires one: a screenshot must not tear down a recording waiting next door.
  if kind == EditorKind::Recording {
    clear_recording_preview(app);
  }
  {
    let state = app.state::<EditorState>();
    let slot = state.slot(kind);
    let mut artifact_slot = slot
      .artifact
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if artifact_slot.is_some() {
      drop(artifact_slot);
      workspace::focus_pending(app, kind);
      return Err("An editor workspace is already open".to_owned());
    }
    let mut reservation = state
      .capture_reservation
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let defaults = crate::settings::current(app);
    let default_directory = match &artifact {
      EditorArtifact::Screenshot { .. } => defaults.screenshot_directory,
      EditorArtifact::Recording { .. } => defaults.recording_directory,
    }
    .or_else(|| crate::screenshots::screenshot_directory(app).ok());
    *slot
      .directory
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner()) = default_directory;
    *artifact_slot = Some(artifact);
    *reservation = None;
  }
  match reveal_when {
    Reveal::Now => reveal(app, kind),
    Reveal::Deferred => {
      let app = app.clone();
      tauri::async_runtime::spawn_blocking(move || {
        if let Err(error) = reveal(&app, kind) {
          eprintln!("Could not show the recovered {kind:?} workspace: {error}");
        }
      });
      Ok(())
    }
  }
}

/// Shows the artifact `present_new` installed for `kind`.
fn reveal(app: &AppHandle, kind: EditorKind) -> Result<(), String> {
  // The hidden webview still holds the last snapshot, an empty one, so the
  // new capture goes to it before the window appears showing that.
  emit_snapshot(app, kind);
  if let Err(error) = window::show(app, kind) {
    // A hidden artifact is a deadlocked workspace. Keep a recording's file on
    // disk so startup recovery can offer it again, but release the in-memory
    // admission state so the current app remains usable.
    let state = app.state::<EditorState>();
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
  // clipboard-only screenshots, which never open the editor window.
  let _ = crate::app_windows::hide_recording_ui(app.clone());
  Ok(())
}

/// Hands a freshly captured still to the editor window, seeding its layer
/// with the live annotations the still covered. `scale_factor` is the
/// display scale it was captured at.
pub fn present_screenshot(
  app: &AppHandle,
  image: CapturedImage,
  scale_factor: f64,
  annotations: Vec<Annotation>,
  suggested_file_stem: String,
) -> Result<(), String> {
  let item = ScreenshotItem {
    annotations,
    id: next_id(app),
    image,
    scale_factor,
  };

  {
    let state = app.state::<EditorState>();
    let mut artifact = state
      .screenshot
      .artifact
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(EditorArtifact::Screenshot { items, .. }) = artifact.as_mut() {
      items.push(item);
      *state
        .capture_reservation
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
      drop(artifact);
      emit_snapshot(app, EditorKind::Screenshot);
      window::show(app, EditorKind::Screenshot).map_err(|error| error.to_string())?;
      let _ = crate::app_windows::hide_recording_ui(app.clone());
      return Ok(());
    }
  }

  present_new(
    app,
    EditorArtifact::Screenshot {
      id: next_id(app),
      items: vec![item],
      suggested_file_stem,
    },
    Reveal::Now,
  )
}

/// Hands a finished recording to the editor window.
///
/// Mirrors `present_screenshot`. The window previews the movie itself through
/// the native preview surface, so nothing still-image is carried here.
pub fn present_recording(
  app: &AppHandle,
  info: FinalizeInfo,
  suggested_file_stem: String,
) -> Result<(), String> {
  present_recording_with(app, info, suggested_file_stem, Reveal::Now)
}

/// [`present_recording`] for a recording startup recovery found, which is
/// shown once setup has returned; see [`Reveal::Deferred`].
pub(super) fn present_recovered_recording(
  app: &AppHandle,
  info: FinalizeInfo,
  suggested_file_stem: String,
) -> Result<(), String> {
  present_recording_with(app, info, suggested_file_stem, Reveal::Deferred)
}

fn present_recording_with(
  app: &AppHandle,
  info: FinalizeInfo,
  suggested_file_stem: String,
  reveal_when: Reveal,
) -> Result<(), String> {
  let FinalizeInfo {
    annotation_clips,
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
  // The container keeps the length on its own time grid, which the writer's
  // clock does not land on. A reopened recording is measured from the file,
  // so a new one is measured the same way: auto zooms capped at a shorter
  // writer length would stop short of the end the timeline later reads.
  let duration_ms = media_preview::duration_ms(&path)
    .filter(|ms| *ms > 0)
    .unwrap_or(duration_ms);

  let mut audio_tracks = recording_audio_tracks(has_system_audio, has_microphone);
  if audio_tracks.is_empty() {
    audio_tracks = media_preview::inspect_audio_tracks(&path).unwrap_or_default();
  }

  let id = next_id(app);
  let scene_clips = super::auto_zoom::for_new_recording(
    app,
    cursor_path.as_deref(),
    keyboard_path.as_deref(),
    duration_ms,
  );
  if !annotation_clips.is_empty() || !scene_clips.is_empty() {
    // The snapshot the editor window loads reads this sidecar, so annotations
    // drawn live and the auto zooms have to be in it before the artifact is
    // presented.
    if let Err(error) =
      timeline_edit::persist_initial_edit(&path, id, annotation_clips, scene_clips)
    {
      eprintln!("Could not keep this recording's live annotations and auto zooms: {error}");
    }
  }

  present_new(
    app,
    EditorArtifact::Recording {
      id,
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
    reveal_when,
  )
}
