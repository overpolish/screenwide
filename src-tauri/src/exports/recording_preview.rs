// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

struct RecordingPreviewSources {
  duration_ms: u64,
  path: PathBuf,
  tracks: Vec<RecordingAudioTrack>,
}

/// Returns the captured keyboard shortcuts as generic timed-lane items.
/// Missing keyboard capture is valid (for example when Accessibility access
/// was unavailable), so that case deliberately returns an empty lane.
/// Deletions and manual placements shape badge continuity — and with it each
/// badge's real exit time — so the current edit is applied before reading.
#[tauri::command]
pub async fn get_recording_keyboard_timeline(
  app: AppHandle,
  artifact_id: u64,
  shortcut_ids: Option<Vec<u64>>,
  shortcut_ranges: Option<Vec<crate::exports::timeline_edit::DeletedKeyboardShortcutRange>>,
  shortcut_positions: Option<Vec<crate::exports::timeline_edit::KeyboardShortcutPositionRange>>,
) -> Result<Vec<crate::exports::keyboard_effects::KeyboardTimelineItem>, String> {
  let path = {
    let state = app.state::<ExportState>();
    let artifact = state
      .recording
      .artifact
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let Some(ExportArtifact::Recording { id, keyboard, .. }) = artifact.as_ref() else {
      return Err("There is no recording to preview".to_owned());
    };
    if *id != artifact_id {
      return Err("That recording is no longer waiting to be exported".to_owned());
    }
    keyboard.as_ref().map(|keyboard| keyboard.path.clone())
  };

  tauri::async_runtime::spawn_blocking(move || {
    path.map_or_else(
      || Ok(Vec::new()),
      |path| {
        let keyboard = crate::exports::keyboard_effects::KeyboardCompositor::open(&path)?;
        keyboard.set_deleted_shortcuts(
          &shortcut_ids.unwrap_or_default(),
          &shortcut_ranges.unwrap_or_default(),
        );
        keyboard.set_shortcut_positions(&shortcut_positions.unwrap_or_default());
        Ok(keyboard.timeline_items())
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
    let state = app.state::<ExportState>();
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
    let preview = media_preview::prepare(
      artifact_id,
      &sources.path,
      sources.duration_ms,
      &sources.tracks,
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

fn recording_sources(
  state: &ExportState,
  artifact_id: u64,
) -> Result<RecordingPreviewSources, String> {
  let artifact = state
    .recording
    .artifact
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  let Some(ExportArtifact::Recording {
    audio_tracks,
    duration_ms,
    id,
    path,
    ..
  }) = artifact.as_ref()
  else {
    return Err("There is no recording to preview".to_owned());
  };
  if *id != artifact_id {
    return Err("That recording is no longer waiting to be exported".to_owned());
  }
  Ok(RecordingPreviewSources {
    duration_ms: *duration_ms,
    path: path.clone(),
    tracks: audio_tracks.clone(),
  })
}

fn ensure_current(state: &ExportState, artifact_id: u64) -> Result<(), String> {
  let current = state
    .recording
    .artifact
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .as_ref()
    .is_some_and(
      |artifact| matches!(artifact, ExportArtifact::Recording { id, .. } if *id == artifact_id),
    );
  current
    .then_some(())
    .ok_or_else(|| "That recording is no longer waiting to be exported".to_owned())
}
