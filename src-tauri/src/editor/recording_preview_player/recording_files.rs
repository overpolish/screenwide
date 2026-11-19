// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a recording's preview sources are read from: its files, their
//! sizes and length, and how it was captured. The open editor's recording
//! holds them; a saved project is read for them the way opening it does,
//! so a recording can be drawn as its edit shows it without opening it.

use std::path::{Path, PathBuf};

use super::*;
use screenwide_export::recording_model::{RecordingAudioTrack, RecordingCamera};
use screenwide_output::Capture;

pub(super) struct RecordingFiles {
  pub(super) audio_tracks: Vec<RecordingAudioTrack>,
  pub(super) camera: Option<RecordingCamera>,
  pub(super) captures: [Capture; 2],
  pub(super) cursor_path: Option<PathBuf>,
  pub(super) keyboard_path: Option<PathBuf>,
  pub(super) duration_ms: u64,
  pub(super) height: u32,
  pub(super) path: PathBuf,
  pub(super) primary_kind: PrimaryRecordingKind,
  /// The project's manifest.
  pub(super) project: PathBuf,
  pub(super) width: u32,
}

impl RecordingFiles {
  /// The recording the editor has open, if it is `artifact_id`.
  pub(super) fn of_open(app: &AppHandle, artifact_id: u64) -> Result<Self, String> {
    let state = app.state::<EditorState>();
    let artifact = state
      .recording
      .artifact
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let Some(
      recording @ EditorArtifact::Recording {
        audio_tracks,
        camera,
        cursor,
        keyboard,
        duration_ms,
        height,
        id,
        path,
        primary_kind,
        project,
        width,
        ..
      },
    ) = artifact.as_ref()
    else {
      return Err("There is no recording to preview".to_owned());
    };
    if *id != artifact_id {
      return Err("That recording is no longer available in the editor".to_owned());
    }
    Ok(Self {
      audio_tracks: audio_tracks.clone(),
      camera: camera.clone(),
      captures: recording.captures(),
      cursor_path: cursor.as_ref().map(|value| value.path.clone()),
      keyboard_path: keyboard.as_ref().map(|value| value.path.clone()),
      duration_ms: *duration_ms,
      height: *height,
      path: path.clone(),
      primary_kind: *primary_kind,
      project: project.clone(),
      width: *width,
    })
  }

  /// The recording saved in the project whose manifest is `project`, read
  /// as opening it reads it: a track that is gone is left out.
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  pub(super) fn of_project(project: &Path) -> Result<Self, String> {
    use screenwide_export::media_preview;

    let manifest = screenwide_project::read(project).map_err(|error| error.to_string())?;
    let recording = manifest.recorded()?;
    let path = screenwide_project::resolve(project, &recording.media.primary)?;
    if !path.is_file() {
      return Err(format!("{} is missing", path.display()));
    }
    let present = |name: &Option<String>| {
      name
        .as_deref()
        .and_then(|name| screenwide_project::resolve(project, name).ok())
        .filter(|path| path.is_file())
    };
    let info = media_preview::recording_info(&path)
      .ok_or_else(|| "The recording cannot be read".to_owned())?;
    let camera = present(&recording.media.camera).and_then(|path| {
      let info = media_preview::recording_info(&path)?;
      Some(RecordingCamera {
        duration_ms: info.duration_ms,
        height: info.height,
        original_size_bytes: 0,
        path,
        width: info.width,
      })
    });
    let mut audio_tracks = crate::editor::artifact_present::recording_audio_tracks(
      recording.has_system_audio,
      recording.has_microphone,
    );
    if audio_tracks.is_empty() {
      audio_tracks = media_preview::inspect_audio_tracks(&path).unwrap_or_default();
    }
    let scale_factor = if recording.source_scale_factor.is_finite() {
      recording.source_scale_factor
    } else {
      1.0
    };
    let captures = crate::editor::recording_model::recording_captures(
      info.width,
      crate::editor::save::scale_percent(scale_factor),
      recording
        .corner_radius
        .filter(|radius| radius.is_finite() && *radius > 0.0),
      camera.as_ref().map(|camera| camera.width),
    );
    Ok(Self {
      audio_tracks,
      camera,
      captures,
      cursor_path: present(&recording.media.cursor),
      keyboard_path: present(&recording.media.keyboard),
      duration_ms: info.duration_ms,
      height: info.height,
      path,
      primary_kind: recording.primary_kind,
      project: project.to_path_buf(),
      width: info.width,
    })
  }
}
