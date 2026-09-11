// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::save::{save_recording_copy, save_selected_recording_copy};
use super::*;

#[path = "camera_save/baked.rs"]
mod baked;
pub(in crate::editor) use baked::save_baked_recording;

/// Check a baked overlay against the screen canvas it is placed in.
///
/// Placement is in that canvas's own pixels, so the bounds are the same shares
/// of it the window has always been able to produce: nothing smaller than a
/// thirtieth of the canvas, and nothing farther out than eight canvases.
pub(super) fn validate_camera_overlay(
  settings: CameraOverlaySettings,
  canvas: (u32, u32),
) -> Result<(), String> {
  let values = [
    settings.camera_x,
    settings.camera_y,
    settings.camera_width,
    settings.frame_height,
    settings.frame_width,
    settings.frame_x,
    settings.frame_y,
    settings.radius_percent,
  ];
  let width = f64::from(canvas.0.max(1));
  let height = f64::from(canvas.1.max(1));
  let horizontal = -width * 8.0..=width * 8.0;
  let vertical = -height * 8.0..=height * 8.0;
  if values.iter().any(|value| !value.is_finite())
    || !horizontal.contains(&settings.camera_x)
    || !vertical.contains(&settings.camera_y)
    || !(width * 0.03..=width * 8.0).contains(&settings.camera_width)
    || !(height * 0.03..=height * 8.0).contains(&settings.frame_height)
    || !(width * 0.03..=width * 8.0).contains(&settings.frame_width)
    || !horizontal.contains(&settings.frame_x)
    || !vertical.contains(&settings.frame_y)
    || !(0.0..=50.0).contains(&settings.radius_percent)
  {
    return Err("The camera overlay settings are not valid".to_owned());
  }
  Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn emit_progress(
  app: &AppHandle,
  artifact_id: u64,
  phase: &'static str,
  processed_ms: u64,
  duration_ms: u64,
  start: f64,
  share: f64,
) {
  let progress_percent = if duration_ms == 0 {
    start
  } else {
    start + (processed_ms as f64 / duration_ms as f64 * share).min(share)
  };
  let _ = app.emit(
    EXPORT_PROGRESS_EVENT,
    ExportProgress {
      artifact_id,
      phase,
      progress_percent,
    },
  );
}

#[allow(clippy::too_many_arguments)]
pub(super) fn save_camera_copy(
  camera: &RecordingCamera,
  directory: &Path,
  stem: &str,
  artifact_id: u64,
  progress_app: &AppHandle,
  cancelled: &AtomicBool,
  progress_start: f64,
  compression: u8,
  resolution_scale_percent: u16,
  output: &ScreenshotOutputSettings,
  timeline: Option<&timeline_edit::TimelinePlan>,
) -> Result<Option<PathBuf>, String> {
  let progress_duration_ms =
    timeline.map_or(camera.duration_ms, timeline_edit::TimelinePlan::duration_ms);
  let camera_stem = format!("{stem} Camera");
  let empty_selection = track_selection::TrackSelection::default();
  let mut on_progress = |processed_ms| {
    emit_progress(
      progress_app,
      artifact_id,
      "camera",
      processed_ms,
      progress_duration_ms,
      progress_start,
      99.0 - progress_start,
    );
  };

  if cursor_export::needs_composition(output, camera.width, camera.height) {
    let path = unique_path(directory, &camera_stem, RECORDING_EXTENSION, &|candidate| {
      candidate.exists()
    });
    let result = cursor_export::export(cursor_export::CursorExportRequest {
      audio_layout: track_selection::AudioLayout::SeparateTracks,
      audio_source: None,
      camera: None,
      camera_on_top: true,
      cancelled,
      cursor: None,
      cursor_effects: cursor_effects::CursorEffectSettings::default(),
      keyboard: None,
      keyboard_effects: keyboard_effects::KeyboardEffectSettings::default(),
      destination: &path,
      duration_ms: camera.duration_ms,
      height: camera.height,
      on_progress: &mut on_progress,
      output,
      screen: &camera.path,
      selection: &empty_selection,
      timeline,
      video: media_preview::VideoExportOptions {
        compression,
        resolution_scale_percent: 100,
        source_scale_percent: 100,
      },
      width: camera.width,
    })?;
    return match result {
      media_preview::ExportRunResult::Completed => {
        let _ = std::fs::remove_file(&camera.path);
        Ok(Some(path))
      }
      media_preview::ExportRunResult::Cancelled => Ok(None),
    };
  }
  let exporter = media_preview::selected_recording_exporter();
  if exporter.is_none() && (timeline.is_some() || compression > 0 || resolution_scale_percent < 100)
  {
    return Err("FFmpeg is required to compress the camera recording".to_owned());
  };
  let saved = if let Some(exporter) = exporter {
    save_selected_recording_copy(
      &camera.path,
      directory,
      &camera_stem,
      &empty_selection,
      track_selection::AudioLayout::SeparateTracks,
      media_preview::ExportRunOptions {
        cancelled,
        on_progress: &mut on_progress,
        timeline,
        video: media_preview::VideoExportOptions {
          compression,
          resolution_scale_percent,
          source_scale_percent: 100,
        },
      },
      Some(exporter),
    )?
  } else {
    Some(save_recording_copy(
      &camera.path,
      directory,
      &camera_stem,
      None,
    )?)
  };
  if saved.is_some() {
    let _ = std::fs::remove_file(&camera.path);
  }
  Ok(saved)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn save_camera_as_primary(
  audio_source: &Path,
  camera: &RecordingCamera,
  directory: &Path,
  stem: &str,
  selection: &track_selection::TrackSelection,
  layout: track_selection::AudioLayout,
  artifact_id: u64,
  progress_app: &AppHandle,
  cancelled: &AtomicBool,
  compression: u8,
  resolution_scale_percent: u16,
  output: &ScreenshotOutputSettings,
  timeline: Option<&timeline_edit::TimelinePlan>,
) -> Result<Option<PathBuf>, String> {
  let progress_duration_ms =
    timeline.map_or(camera.duration_ms, timeline_edit::TimelinePlan::duration_ms);
  if cursor_export::needs_composition(output, camera.width, camera.height) {
    let path = unique_path(directory, stem, RECORDING_EXTENSION, &|candidate| {
      candidate.exists()
    });
    let mut on_progress = |processed_ms| {
      emit_progress(
        progress_app,
        artifact_id,
        "camera",
        processed_ms,
        progress_duration_ms,
        0.0,
        99.0,
      );
    };
    return match cursor_export::export(cursor_export::CursorExportRequest {
      audio_layout: layout,
      audio_source: Some(audio_source),
      camera: None,
      camera_on_top: true,
      cancelled,
      cursor: None,
      cursor_effects: cursor_effects::CursorEffectSettings::default(),
      keyboard: None,
      keyboard_effects: keyboard_effects::KeyboardEffectSettings::default(),
      destination: &path,
      duration_ms: camera.duration_ms,
      height: camera.height,
      on_progress: &mut on_progress,
      output,
      screen: &camera.path,
      selection,
      timeline,
      video: media_preview::VideoExportOptions {
        compression,
        resolution_scale_percent: 100,
        source_scale_percent: 100,
      },
      width: camera.width,
    })? {
      media_preview::ExportRunResult::Completed => Ok(Some(path)),
      media_preview::ExportRunResult::Cancelled => Ok(None),
    };
  }
  let exporter = media_preview::camera_recording_exporter()
    .ok_or_else(|| "FFmpeg is required to export the camera track on its own".to_owned())?;
  let path = unique_path(directory, stem, RECORDING_EXTENSION, &|candidate| {
    candidate.exists()
  });
  let mut on_progress = |processed_ms| {
    emit_progress(
      progress_app,
      artifact_id,
      "camera",
      processed_ms,
      progress_duration_ms,
      0.0,
      99.0,
    );
  };
  match exporter(
    audio_source,
    &camera.path,
    &path,
    selection,
    layout,
    media_preview::ExportRunOptions {
      cancelled,
      on_progress: &mut on_progress,
      timeline,
      video: media_preview::VideoExportOptions {
        compression,
        resolution_scale_percent,
        source_scale_percent: 100,
      },
    },
  )? {
    media_preview::ExportRunResult::Completed => Ok(Some(path)),
    media_preview::ExportRunResult::Cancelled => Ok(None),
  }
}
