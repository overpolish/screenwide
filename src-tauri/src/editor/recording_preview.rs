// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

struct RecordingPreviewSources {
  duration_ms: u64,
  path: PathBuf,
  project_folder: PathBuf,
  tracks: Vec<RecordingAudioTrack>,
}

/// Returns the captured keyboard shortcuts as generic timed-lane items.
/// Missing keyboard capture is valid (for example when Accessibility access
/// was unavailable), so that case deliberately returns an empty lane.
/// Deletions and manual placements shape badge continuity - and with it each
/// badge's real exit time - so the current edit is applied before reading.
#[tauri::command]
pub async fn get_recording_keyboard_timeline(
  app: AppHandle,
  artifact_id: u64,
  playback_ranges: Option<Vec<recording_preview_player::RecordingPreviewPlaybackRange>>,
  shortcut_ids: Option<Vec<u64>>,
  shortcut_ranges: Option<Vec<crate::editor::timeline_edit::DeletedKeyboardShortcutRange>>,
  shortcut_positions: Option<Vec<crate::editor::timeline_edit::KeyboardShortcutPositionRange>>,
) -> Result<Vec<crate::editor::keyboard_effects::KeyboardTimelineItem>, String> {
  let path = {
    let state = app.state::<EditorState>();
    let artifact = state
      .recording
      .artifact
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let Some(EditorArtifact::Recording { id, keyboard, .. }) = artifact.as_ref() else {
      return Err("There is no recording to preview".to_owned());
    };
    if *id != artifact_id {
      return Err("That recording is no longer available in the editor".to_owned());
    }
    keyboard.as_ref().map(|keyboard| keyboard.path.clone())
  };

  tauri::async_runtime::spawn_blocking(move || {
    let timeline_ranges = playback_ranges
      .as_deref()
      .map(recording_preview_player::animation_timeline_ranges);
    path.map_or_else(
      || Ok(Vec::new()),
      |path| {
        let keyboard = crate::editor::keyboard_effects::KeyboardCompositor::open(&path)?;
        keyboard.set_deleted_shortcuts(
          &shortcut_ids.unwrap_or_default(),
          &shortcut_ranges.unwrap_or_default(),
        );
        keyboard.set_shortcut_positions(&shortcut_positions.unwrap_or_default());
        Ok(keyboard.timeline_items_with_timeline(timeline_ranges.as_deref()))
      },
    )
  })
  .await
  .map_err(|error| error.to_string())?
}

/// Prepares the lightweight waveform data used beside the native player.
#[tauri::command]
pub async fn get_recording_preview(
  app: AppHandle,
  artifact_id: u64,
) -> Result<media_preview::RecordingPreview, String> {
  tauri::async_runtime::spawn_blocking(move || {
    let state = app.state::<EditorState>();
    let _preparing = state
      .recording_preview_preparation
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(preview) = state
      .recording_preview
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
      .as_ref()
      .filter(|preview| preview.artifact_id == artifact_id)
      .cloned()
    {
      return Ok(preview);
    }

    let sources = recording_sources(&state, artifact_id)?;
    let streams: Vec<usize> = sources
      .tracks
      .iter()
      .map(|track| track.stream_index)
      .collect();
    let preview = media_preview::prepare(
      artifact_id,
      &sources.path,
      sources.duration_ms,
      &sources.tracks,
      &crate::editor::speech::noise::cleaned_tracks(&sources.project_folder, &streams),
    )?;
    ensure_current(&state, artifact_id)?;
    state
      .recording_preview
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
      .replace(preview.clone());
    Ok(preview)
  })
  .await
  .map_err(|error| error.to_string())?
}

/// The event that hands the editor the recording's waveforms again after one
/// of them changed.
const PREVIEW_EVENT: &str = "editor://recording-preview";

/// Reads the `stream`th track's waveform again as it is now heard, cleaned
/// or not, once the waveforms have been prepared, and hands the editor the
/// result, so its waveform and meter follow Reduce noise.
pub(crate) fn refresh_waveform(
  app: &AppHandle,
  artifact_id: u64,
  stream: usize,
) -> Result<(), String> {
  let state = app.state::<EditorState>();
  let _preparing = state
    .recording_preview_preparation
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  let is_prepared = state
    .recording_preview
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .as_ref()
    .is_some_and(|preview| preview.artifact_id == artifact_id);
  if !is_prepared {
    // The first preparation reads the waveforms as heard, whenever it runs.
    return Ok(());
  }
  let sources = recording_sources(&state, artifact_id)?;
  let Some(track) = sources
    .tracks
    .iter()
    .find(|track| track.stream_index == stream)
  else {
    return Ok(());
  };
  let cleaned = crate::editor::speech::noise::cleaned_tracks(&sources.project_folder, &[stream]);
  let waveform = media_preview::waveform(
    &sources.path,
    track,
    sources.duration_ms,
    cleaned.first().map(|(_, file)| file.as_path()),
  )?;
  ensure_current(&state, artifact_id)?;
  let mut cached = state
    .recording_preview
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  let Some(preview) = cached
    .as_mut()
    .filter(|preview| preview.artifact_id == artifact_id)
  else {
    return Ok(());
  };
  if let Some(prepared) = preview
    .tracks
    .iter_mut()
    .find(|track| track.stream_index == stream)
  {
    prepared.waveform = waveform;
  }
  let _ = app.emit(PREVIEW_EVENT, preview.clone());
  Ok(())
}

fn recording_sources(
  state: &EditorState,
  artifact_id: u64,
) -> Result<RecordingPreviewSources, String> {
  let artifact = state
    .recording
    .artifact
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  let Some(EditorArtifact::Recording {
    audio_tracks,
    duration_ms,
    id,
    path,
    project,
    ..
  }) = artifact.as_ref()
  else {
    return Err("There is no recording to preview".to_owned());
  };
  if *id != artifact_id {
    return Err("That recording is no longer available in the editor".to_owned());
  }
  Ok(RecordingPreviewSources {
    duration_ms: *duration_ms,
    path: path.clone(),
    project_folder: project
      .parent()
      .map_or_else(|| PathBuf::from("."), Path::to_path_buf),
    tracks: audio_tracks.clone(),
  })
}

fn ensure_current(state: &EditorState, artifact_id: u64) -> Result<(), String> {
  let current = state
    .recording
    .artifact
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .as_ref()
    .is_some_and(
      |artifact| matches!(artifact, EditorArtifact::Recording { id, .. } if *id == artifact_id),
    );
  current
    .then_some(())
    .ok_or_else(|| "That recording is no longer available in the editor".to_owned())
}
