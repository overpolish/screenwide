// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::export_inputs::ExportInputs;
use super::*;
use crate::editor::export_picture::ExportPicture;
mod estimate_helpers;
mod video;

#[tauri::command]
pub async fn estimate_recording_export(
  app: AppHandle,
  artifact_id: u64,
  options: RecordingExportOptions,
) -> Result<u64, String> {
  tauri::async_runtime::spawn_blocking(move || {
    let state = app.state::<EditorState>();
    let cancelled = estimate_helpers::replace_running(&state);
    let inputs = ExportInputs::read(&state, artifact_id, options)?;

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
      duration_ms: inputs.duration_ms,
      state: &state,
      timeline: inputs.timeline.as_ref(),
    };
    let screen_video = match inputs.primary_picture() {
      None => 0,
      Some(picture) if inputs.bake_camera => estimate.composed(picture)?,
      Some(picture) => estimate.screen(picture, &inputs.path, &inputs.tracks, inputs.video())?,
    };
    let camera_video = match inputs.camera.as_ref() {
      Some(camera) if inputs.include_camera && !inputs.bake_camera => estimate.camera(
        ExportPicture::camera(
          camera,
          &inputs.recording_output.camera,
          inputs.timeline.as_ref(),
          inputs.camera_effects.clone(),
          inputs.camera_compression,
          inputs.camera_resolution_scale_percent,
          None,
        ),
        camera,
        inputs.camera_compression,
        inputs.camera_resolution_scale_percent,
      )?,
      _ => 0,
    };
    let audio = estimate.to_output(inputs.selection.estimated_audio_bytes(
      &inputs.tracks,
      inputs.layout,
      inputs.duration_ms,
    ));
    Ok(estimate_helpers::with_container(
      screen_video
        .saturating_add(camera_video)
        .saturating_add(audio),
    ))
  })
  .await
  .map_err(|error| error.to_string())?
}
