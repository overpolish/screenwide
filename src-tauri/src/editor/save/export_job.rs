// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

mod publish;

pub(super) fn save_recording_artifact(
  artifact: &EditorArtifact,
  writing: &Path,
  stem: &str,
  progress_app: &AppHandle,
  job_cancellation: &AtomicBool,
  options: RecordingExportOptions,
) -> Result<Option<PathBuf>, String> {
  let RecordingExportOptions {
    audio_track_volumes,
    bake_camera,
    camera_compression,
    camera_overlay,
    camera_resolution_scale_percent,
    collapse_audio,
    compression,
    cursor_effects,
    keyboard_effects,
    enabled_stream_indices,
    include_camera,
    include_primary_video,
    resolution_scale_percent,
    recording_output,
    timeline_edit: timeline_model,
    ..
  } = options;
  let EditorArtifact::Recording {
    audio_tracks,
    camera,
    cursor,
    keyboard,
    duration_ms,
    height,
    id,
    path: working,
    primary_kind,
    source_scale_percent,
    width,
    ..
  } = artifact
  else {
    unreachable!("recording export helper received a non-recording artifact");
  };
  validate_primary_resolution_scale(
    resolution_scale_percent,
    *source_scale_percent,
    *primary_kind,
  )?;
  validate_camera_resolution_scale(camera_resolution_scale_percent)?;
  validate_camera_overlay(
    camera_overlay,
    (
      recording_output.primary.width,
      recording_output.primary.height,
    ),
  )?;
  let selection = track_selection::TrackSelection::with_volumes(
    audio_tracks,
    &enabled_stream_indices,
    &audio_track_volumes,
  )?;
  let layout = if collapse_audio {
    track_selection::AudioLayout::Mixdown
  } else {
    track_selection::AudioLayout::SeparateTracks
  };
  let baked_cursor = cursor
    .as_ref()
    .filter(|_| cursor_effects.bake)
    .map(|cursor| cursor.path.as_path());
  let baked_keyboard = keyboard
    .as_ref()
    .filter(|_| keyboard_effects.bake)
    .map(|keyboard| keyboard.path.as_path());
  let primary_output = &recording_output.primary;
  let camera_output = &recording_output.camera;
  let persisted_timeline;
  let timeline_edit = if let Some(edit) = timeline_model
    .as_ref()
    .filter(|edit| edit.artifact_id == *id)
  {
    Some(edit)
  } else {
    persisted_timeline = timeline_edit::for_recording(working, *id).map(|(_, edit)| edit);
    persisted_timeline.as_ref()
  };
  let timeline =
    timeline_edit.and_then(|edit| timeline_edit::TimelinePlan::from_edit(edit, *duration_ms));
  let export_duration_ms = timeline
    .as_ref()
    .map_or(*duration_ms, timeline_edit::TimelinePlan::duration_ms);
  if !include_primary_video && !include_camera && enabled_stream_indices.is_empty() {
    return Err("Select at least one track to export".to_owned());
  }
  if *primary_kind == PrimaryRecordingKind::Audio {
    if include_primary_video || include_camera || bake_camera {
      return Err("This audio recording has no video track to export".to_owned());
    }
    return audio_save::save_audio(audio_save::AudioSaveRequest {
      app: progress_app,
      cancelled: job_cancellation,
      directory: writing,
      duration_ms: export_duration_ms,
      id: *id,
      layout,
      selected_any: !enabled_stream_indices.is_empty(),
      selection: &selection,
      stem,
      timeline: timeline.as_ref(),
      working,
    });
  }
  if !include_primary_video {
    if include_camera {
      let camera = camera
        .as_ref()
        .ok_or_else(|| "There is no camera track to export".to_owned())?;
      let saved = camera_save::save_camera_as_primary(
        working,
        camera,
        writing,
        stem,
        &selection,
        layout,
        *id,
        progress_app,
        job_cancellation,
        camera_compression,
        camera_resolution_scale_percent,
        camera_output,
        timeline.as_ref(),
      )?;
      if saved.is_some() {
        let _ = std::fs::remove_file(working);
        let _ = std::fs::remove_file(&camera.path);
        recording_sidecar::remove_working_files(cursor.as_ref(), keyboard.as_ref());
      }
      return Ok(saved);
    }

    let saved = audio_save::save_audio(audio_save::AudioSaveRequest {
      app: progress_app,
      cancelled: job_cancellation,
      directory: writing,
      duration_ms: export_duration_ms,
      id: *id,
      layout,
      selected_any: !enabled_stream_indices.is_empty(),
      selection: &selection,
      stem,
      timeline: timeline.as_ref(),
      working,
    })?;
    if saved.is_some() {
      if let Some(camera) = camera {
        let _ = std::fs::remove_file(&camera.path);
      }
      recording_sidecar::remove_working_files(cursor.as_ref(), keyboard.as_ref());
    }
    return Ok(saved);
  }

  if bake_camera {
    if !include_camera {
      return Err("Select the camera track before baking it in".to_owned());
    }
    let camera = camera
      .as_ref()
      .ok_or_else(|| "There is no camera recording to bake in".to_owned())?;
    let saved = camera_save::save_baked_recording(
      working,
      camera,
      writing,
      stem,
      &selection,
      layout,
      *id,
      *duration_ms,
      (*width, *height),
      camera_overlay,
      camera_output.drop_shadow,
      recording_output.camera_on_top,
      (compression, resolution_scale_percent, *source_scale_percent),
      baked_cursor.map(|cursor| (cursor, cursor_effects)),
      baked_keyboard.map(|keyboard| (keyboard, keyboard_effects)),
      primary_output,
      progress_app,
      job_cancellation,
      timeline.as_ref(),
    )?;
    if saved.is_some() {
      let _ = std::fs::remove_file(working);
      let _ = std::fs::remove_file(&camera.path);
      recording_sidecar::remove_working_files(cursor.as_ref(), keyboard.as_ref());
    }
    return Ok(saved);
  }

  let screen_progress_share = if include_camera && camera.is_some() {
    50.0
  } else {
    99.0
  };
  let saved = save_primary_recording(PrimaryRecordingSaveRequest {
    app: progress_app,
    artifact_id: *id,
    audio_tracks,
    cancelled: job_cancellation,
    compression,
    cursor: baked_cursor,
    cursor_effects,
    keyboard: baked_keyboard,
    keyboard_effects,
    directory: writing,
    duration_ms: *duration_ms,
    height: *height,
    layout,
    output: primary_output,
    progress_share: screen_progress_share,
    resolution_scale_percent,
    screen: working,
    selection: &selection,
    source_scale_percent: *source_scale_percent,
    stem,
    timeline: timeline.as_ref(),
    width: *width,
  })?;
  let Some(saved) = saved else {
    return Ok(None);
  };
  let next_phase = if include_camera && camera.is_some() {
    "camera"
  } else {
    "finalizing"
  };
  let _ = progress_app.emit(
    EXPORT_PROGRESS_EVENT,
    ExportProgress {
      artifact_id: *id,
      phase: next_phase,
      progress_percent: screen_progress_share,
    },
  );

  let mut saved_camera = None;
  if include_camera {
    let camera = camera
      .as_ref()
      .ok_or_else(|| "There is no camera track to export".to_owned())?;
    let camera_path = camera_save::save_camera_copy(
      camera,
      writing,
      stem,
      *id,
      progress_app,
      job_cancellation,
      screen_progress_share,
      camera_compression,
      camera_resolution_scale_percent,
      camera_output,
      timeline.as_ref(),
    )
    .inspect_err(|_| {
      let _ = std::fs::remove_file(&saved);
    })?;
    let Some(camera_path) = camera_path else {
      let _ = std::fs::remove_file(&saved);
      return Ok(None);
    };
    saved_camera = Some(camera_path);
  }
  let _ = progress_app.emit(
    EXPORT_PROGRESS_EVENT,
    ExportProgress {
      artifact_id: *id,
      phase: "finalizing",
      progress_percent: 99.0,
    },
  );
  publish::finish(
    &saved,
    saved_camera,
    working,
    camera.as_ref(),
    include_camera,
    cursor.as_ref(),
    keyboard.as_ref(),
  )?;
  Ok(Some(saved))
}
