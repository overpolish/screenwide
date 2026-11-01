// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::Deserialize;
use ts_rs::TS;

use super::export_inputs::ExportInputs;
use super::*;
use crate::editor::export_picture::ExportPicture;
mod estimate_helpers;
mod video;

/// One quality the Export island offers: the screen's compression and
/// scale, and a separate camera file's.
#[derive(Clone, Copy, Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ExportLookQuality {
  pub camera_compression: u8,
  pub camera_resolution_scale_percent: u16,
  pub compression: u8,
  pub resolution_scale_percent: u16,
}

/// The size of the recording's export with `options` at each of `looks`,
/// in order. Every look is measured in one pass over the picture, which is
/// drawn or decoded once and encoded at each look at once.
#[tauri::command]
pub async fn estimate_recording_export(
  app: AppHandle,
  artifact_id: u64,
  options: RecordingExportOptions,
  looks: Vec<ExportLookQuality>,
) -> Result<Vec<u64>, String> {
  tauri::async_runtime::spawn_blocking(move || {
    let state = app.state::<EditorState>();
    let cancelled = estimate_helpers::replace_running(&state);
    let inputs = ExportInputs::read(&state, artifact_id, options)?;
    let (screen_videos, camera_videos): (Vec<_>, Vec<_>) = looks
      .iter()
      .map(|look| inputs.look_videos(look))
      .collect::<Result<Vec<_>, String>>()?
      .into_iter()
      .unzip();

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
      None => vec![0; looks.len()],
      Some(picture) if inputs.bake_camera => estimate.composed(picture, &screen_videos)?,
      Some(picture) => estimate.screen(picture, &inputs.path, &inputs.tracks, &screen_videos)?,
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
        &camera_videos,
      )?,
      _ => vec![0; looks.len()],
    };
    let audio = estimate.to_output(inputs.selection.estimated_audio_bytes(
      &inputs.tracks,
      inputs.layout,
      inputs.duration_ms,
    ));
    Ok(
      screen_video
        .into_iter()
        .zip(camera_video)
        .map(|(screen, camera)| {
          estimate_helpers::with_container(screen.saturating_add(camera).saturating_add(audio))
        })
        .collect(),
    )
  })
  .await
  .map_err(|error| error.to_string())?
}
