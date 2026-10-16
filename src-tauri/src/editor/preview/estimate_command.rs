// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::editor::export_picture::{ExportPicture, Screen};
use screenwide_export::media_preview::VideoExportOptions;
use screenwide_export::track_selection;
mod estimate_helpers;
mod video;

#[tauri::command]
pub async fn estimate_recording_export(
  app: AppHandle,
  artifact_id: u64,
  options: RecordingExportOptions,
) -> Result<u64, String> {
  let RecordingExportOptions {
    audio_track_volumes,
    bake_camera,
    camera_compression,
    camera_overlay,
    camera_resolution_scale_percent,
    collapse_audio,
    compression,
    delete_project_after_export: _,
    cursor_effects,
    keyboard_effects,
    enabled_stream_indices,
    include_camera,
    include_primary_video,
    resolution_scale_percent,
    mut recording_output,
    screenshot_output: _,
    subtitles: _,
    timeline_edit: timeline_model,
    captions: _,
  } = options;
  if compression > 4 || camera_compression > 4 {
    return Err("Compression must be between 0 and 4".to_owned());
  }
  validate_camera_overlay(
    camera_overlay,
    (
      recording_output.primary.width,
      recording_output.primary.height,
    ),
  )?;

  tauri::async_runtime::spawn_blocking(move || {
    let state = app.state::<EditorState>();
    let cancelled = estimate_helpers::replace_running(&state);
    let (camera_recording, captures, recording) = {
      let artifact = state
        .recording
        .artifact
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
      let Some(
        artifact @ EditorArtifact::Recording {
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
          source_scale_percent,
          width,
          ..
        },
      ) = artifact.as_ref()
      else {
        return Err("There is no recording to estimate".to_owned());
      };
      if *id != artifact_id {
        return Err("That recording is no longer available in the editor".to_owned());
      }
      (
        crate::editor::camera_effects::CameraRecording::of(artifact),
        artifact.captures(),
        (
          audio_tracks.clone(),
          camera.clone(),
          cursor.as_ref().map(|cursor| cursor.path.clone()),
          keyboard.as_ref().map(|keyboard| keyboard.path.clone()),
          *duration_ms,
          (*width, *height),
          path.clone(),
          *primary_kind,
          project.clone(),
          *source_scale_percent,
        ),
      )
    };
    let (
      ref tracks,
      ref camera,
      ref cursor,
      ref keyboard,
      duration_ms,
      (width, height),
      ref path,
      primary_kind,
      ref project,
      source_scale_percent,
    ) = recording;
    if bake_camera && (!include_primary_video || !include_camera || camera.is_none()) {
      return Err("There is no camera recording to bake in".to_owned());
    }
    if !include_primary_video && !include_camera && enabled_stream_indices.is_empty() {
      return Err("Select at least one track to export".to_owned());
    }
    validate_primary_resolution_scale(
      resolution_scale_percent,
      source_scale_percent,
      primary_kind,
    )?;
    validate_camera_resolution_scale(camera_resolution_scale_percent)?;
    recording_output.stamp_captures(captures);

    let project_folder = project.parent().unwrap_or(std::path::Path::new("."));
    let selection = crate::editor::speech::as_heard(
      track_selection::TrackSelection::with_volumes(
        tracks,
        &enabled_stream_indices,
        &audio_track_volumes,
      )?,
      project_folder,
      &enabled_stream_indices,
    );
    let layout = if collapse_audio {
      track_selection::AudioLayout::Mixdown
    } else {
      track_selection::AudioLayout::SeparateTracks
    };
    let timeline =
      timeline_edit::export_plan(timeline_model.as_ref(), project, artifact_id, duration_ms);
    let edit =
      timeline_edit::export_edit(timeline_model.as_ref(), project, artifact_id, duration_ms);
    // The same inputs the export reads: the camera's effects over the
    // edit's background ranges, and the captions drawn into the picture.
    let camera_effects = camera_recording
      .and_then(|camera| crate::editor::camera_effects::live::load(&camera, edit.as_ref()));
    let primary_output = &recording_output.primary;
    let captions = edit.as_ref().and_then(|edit| {
      crate::editor::captions::track_for(
        (project, tracks, duration_ms),
        edit,
        crate::editor::captions::frame_for(primary_output.width, primary_output.height),
      )
    });
    let camera_is_primary = primary_kind == PrimaryRecordingKind::Camera;
    let has_video = width > 0 && height > 0;

    let _preparing = state
      .compression_estimate_preparation
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    // A newer estimate asked while this one waited its turn: its answer is
    // the one wanted.
    if cancelled.load(std::sync::atomic::Ordering::Acquire) {
      return Err("A newer estimate replaced this one".to_owned());
    }
    let estimate = video::Estimate {
      artifact_id,
      cancelled: &cancelled,
      duration_ms,
      state: &state,
      timeline: timeline.as_ref(),
    };
    let video = VideoExportOptions {
      compression,
      resolution_scale_percent,
      source_scale_percent,
    };
    let screen = |camera_effects| Screen {
      camera_effects,
      captions: captions.as_ref(),
      cursor: cursor
        .as_ref()
        .filter(|_| cursor_effects.bake)
        .map(|cursor| (cursor.as_path(), cursor_effects)),
      keyboard: keyboard
        .as_ref()
        .filter(|_| keyboard_effects.bake)
        .map(|keyboard| (keyboard.as_path(), keyboard_effects)),
      duration_ms,
      layout,
      output: primary_output,
      screen: path,
      selection: &selection,
      size: (width, height),
      timeline: timeline.as_ref(),
      video,
    };
    let screen_video = match camera.as_ref() {
      _ if !include_primary_video || !has_video => 0,
      Some(camera) if bake_camera => estimate.composed(ExportPicture::baked(
        screen(camera_effects.clone()),
        camera,
        camera_overlay,
        recording_output.camera.drop_shadow,
      ))?,
      _ => estimate.screen(
        ExportPicture::screen(screen(camera_effects.clone().filter(|_| camera_is_primary))),
        path,
        tracks,
        video,
      )?,
    };
    let camera_video = match camera.as_ref() {
      Some(camera) if include_camera && !bake_camera => estimate.camera(
        ExportPicture::camera(
          camera,
          &recording_output.camera,
          timeline.as_ref(),
          camera_effects,
          camera_compression,
          None,
        ),
        camera,
        camera_compression,
        camera_resolution_scale_percent,
      )?,
      _ => 0,
    };
    let audio = estimate.to_output(selection.estimated_audio_bytes(tracks, layout, duration_ms));
    Ok(estimate_helpers::with_container(
      screen_video
        .saturating_add(camera_video)
        .saturating_add(audio),
    ))
  })
  .await
  .map_err(|error| error.to_string())?
}
