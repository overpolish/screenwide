// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Putting a new capture, or a project being opened, in front of the user in
//! its editor workspace.

use super::artifact::{clear_recording_preview, next_id};
use super::*;
use screenwide_export::media_preview;

/// Where an artifact comes from, which decides whether it ends a capture's
/// reservation.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Origin {
  /// A capture that reserved its workspace before it started.
  Capture,
  /// A project being opened, which reserved nothing.
  Project,
}

/// Puts a new artifact in front of the user.
///
/// Whatever it replaces is kept in its project, so letting go of it loses
/// nothing. A capture arriving while a screenshot is open joins it instead
/// (see [`super::screenshot_project::present_screenshot`]).
pub(super) fn present_new(
  app: &AppHandle,
  artifact: EditorArtifact,
  origin: Origin,
) -> Result<(), String> {
  let kind = EditorKind::of(&artifact);
  // Only the recording workspace owns a preview, and only its own arrival
  // retires one: a screenshot must not tear down a recording open next door.
  if kind == EditorKind::Recording {
    clear_recording_preview(app);
  }
  let replaced = {
    let state = app.state::<EditorState>();
    let slot = state.slot(kind);
    let mut artifact_slot = slot
      .artifact
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    // A capture arriving while a screenshot is open is added to it before it
    // gets here; one that still finds a screenshot raced another. A project
    // opened in its place is saved, so it may replace it.
    if kind == EditorKind::Screenshot && artifact_slot.is_some() && origin == Origin::Capture {
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
    let replaced = artifact_slot.replace(artifact);
    if origin == Origin::Capture {
      *reservation = None;
    }
    replaced
  };
  // A project replaced by another is let go of as surely as one closed.
  if let Some(project) = replaced.as_ref().and_then(EditorArtifact::project) {
    super::project_look::clean_pictures(project);
    super::sound_library::clean(project);
    super::project_name::settle_name(app, project);
  }
  reveal(app, kind);
  Ok(())
}

/// Shows the artifact `present_new` installed for `kind`.
fn reveal(app: &AppHandle, kind: EditorKind) {
  // A window already built still holds the last snapshot, an empty one, so
  // the new capture goes to it before the window appears showing that. One
  // built now reads the snapshot as its page loads.
  emit_snapshot(app, kind);
  // Once an artifact is safely in front of the user, the capture controls
  // have finished their job. Keeping this at the shared presentation boundary
  // gives screenshots and recordings the same handoff without affecting
  // clipboard-only screenshots, which never open the editor window.
  window::show(app, kind, |app| {
    let _ = crate::app_windows::hide_recording_ui(app.clone());
  });
}

/// Hands a finished recording to the editor window. `project` is its
/// manifest, whose name is what an export is called unless the user says
/// otherwise.
///
/// Mirrors `present_screenshot`. The window previews the movie itself through
/// the native preview surface, so nothing still-image is carried here.
pub fn present_recording(
  app: &AppHandle,
  project: PathBuf,
  info: FinalizeInfo,
) -> Result<(), String> {
  present_recording_from(app, project, info, Origin::Capture)
}

/// Gives a recording kept without opening it, such as a saved replay clip,
/// the same first edit an opened one gets, and lists it with the recent
/// projects.
pub fn keep_recording(app: &AppHandle, project: &Path, info: FinalizeInfo) {
  seed_initial_edit(
    app,
    project,
    // Any id: the edit is bound to the artifact it opens as.
    0,
    info.annotation_clips,
    info.cursor_path.as_deref(),
    info.keyboard_path.as_deref(),
    &info.path,
    info.duration_ms,
  );
  screenwide_project::library::remember(app, project);
}

/// Writes a new recording's first edit, carrying the annotations drawn live
/// and the auto zooms made from its clicks and typing. A project that already
/// has an edit keeps it. Returns the recording's length as its movie gives
/// it.
#[allow(clippy::too_many_arguments)]
fn seed_initial_edit(
  app: &AppHandle,
  project: &Path,
  id: u64,
  annotation_clips: Vec<screenwide_annotations::timing::RecordingAnnotationClip>,
  cursor_path: Option<&Path>,
  keyboard_path: Option<&Path>,
  movie: &Path,
  duration_ms: u64,
) -> u64 {
  // The container keeps the length on its own time grid, which the writer's
  // clock does not land on. A reopened recording is measured from the file,
  // so a new one is measured the same way: auto zooms capped at a shorter
  // writer length would stop short of the end the timeline later reads.
  let duration_ms = media_preview::duration_ms(movie)
    .filter(|ms| *ms > 0)
    .unwrap_or(duration_ms);
  let scene_clips =
    super::auto_zoom::for_new_recording(app, cursor_path, keyboard_path, duration_ms);
  if !annotation_clips.is_empty() || !scene_clips.is_empty() {
    // The snapshot the editor window loads reads the project, so annotations
    // drawn live and the auto zooms have to be in it before the artifact is
    // presented.
    if let Err(error) =
      timeline_edit::persist_initial_edit(project, id, annotation_clips, scene_clips)
    {
      eprintln!("Could not keep this recording's live annotations and auto zooms: {error}");
    }
  }
  duration_ms
}

pub(super) fn present_recording_from(
  app: &AppHandle,
  project: PathBuf,
  info: FinalizeInfo,
  origin: Origin,
) -> Result<(), String> {
  let FinalizeInfo {
    annotation_clips,
    camera,
    corner_radius,
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

  let id = next_id(app);
  let duration_ms = seed_initial_edit(
    app,
    &project,
    id,
    annotation_clips,
    cursor_path.as_deref(),
    keyboard_path.as_deref(),
    &path,
    duration_ms,
  );

  let suggested_file_stem = project
    .file_stem()
    .and_then(|stem| stem.to_str())
    .unwrap_or_default()
    .to_owned();
  let remembered = project.clone();
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
      corner_radius,
      cursor: cursor_path.map(RecordingCursor::new),
      keyboard: keyboard_path.map(RecordingKeyboard::new),
      duration_ms,
      height,
      path,
      primary_kind,
      project,
      source_scale_percent: scale_percent(source_scale_factor),
      suggested_file_stem,
      width,
    },
    origin,
  )?;
  screenwide_project::library::remember(app, &remembered);
  Ok(())
}

pub(super) fn recording_audio_tracks(
  has_system_audio: bool,
  has_microphone: bool,
) -> Vec<RecordingAudioTrack> {
  let mut tracks = Vec::with_capacity(usize::from(has_system_audio) + usize::from(has_microphone));
  if has_system_audio {
    tracks.push(RecordingAudioTrack {
      kind: AudioTrackKind::SystemAudio,
      label: AudioTrackKind::SystemAudio.label(tracks.len()),
      stream_index: tracks.len(),
    });
  }
  if has_microphone {
    tracks.push(RecordingAudioTrack {
      kind: AudioTrackKind::Microphone,
      label: AudioTrackKind::Microphone.label(tracks.len()),
      stream_index: tracks.len(),
    });
  }
  tracks
}
