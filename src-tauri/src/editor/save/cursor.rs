// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use screenwide_export::cursor_export;
use screenwide_export::media_preview;
use screenwide_export::track_selection;

pub(super) struct CursorSaveRequest<'a> {
  pub app: &'a AppHandle,
  pub artifact_id: u64,
  pub camera_effects:
    Option<std::sync::Arc<screenwide_export::camera_effects::live::LiveCameraEffects>>,
  pub cancelled: &'a AtomicBool,
  pub cursor: Option<&'a Path>,
  pub cursor_effects: screenwide_cursor_effects::CursorEffectSettings,
  pub directory: &'a Path,
  pub duration_ms: u64,
  pub height: u32,
  pub keyboard: Option<&'a Path>,
  pub keyboard_effects: screenwide_keyboard_effects::KeyboardEffectSettings,
  pub layout: track_selection::AudioLayout,
  pub output: &'a ScreenshotOutputSettings,
  pub progress_share: f64,
  pub screen: &'a Path,
  pub selection: &'a track_selection::TrackSelection,
  pub stem: &'a str,
  pub timeline: Option<&'a screenwide_timeline_edit::TimelinePlan>,
  pub video: media_preview::VideoExportOptions,
  pub width: u32,
}

pub(super) fn save_baked(request: CursorSaveRequest<'_>) -> Result<Option<PathBuf>, String> {
  let progress_duration_ms = request.timeline.map_or(
    request.duration_ms,
    screenwide_timeline_edit::TimelinePlan::duration_ms,
  );
  let path = unique_path(
    request.directory,
    request.stem,
    RECORDING_EXTENSION,
    &|candidate| candidate.exists(),
  );
  let mut on_progress = |processed_ms| {
    camera_save::emit_progress(
      request.app,
      request.artifact_id,
      "recording",
      processed_ms,
      progress_duration_ms,
      0.0,
      request.progress_share,
    );
  };
  match cursor_export::export(cursor_export::CursorExportRequest {
    annotation_track: screenwide_annotations::timing::AnnotationTrack::Primary,
    audio_layout: request.layout,
    audio_source: None,
    camera: None,
    camera_effects: request
      .camera_effects
      .map(|live| cursor_export::ExportCameraEffects {
        live,
        on_screen: true,
      }),
    camera_on_top: true,
    cancelled: request.cancelled,
    cursor: request.cursor,
    cursor_effects: request.cursor_effects,
    keyboard: request.keyboard,
    keyboard_effects: request.keyboard_effects,
    destination: &path,
    duration_ms: request.duration_ms,
    height: request.height,
    on_progress: &mut on_progress,
    screen: request.screen,
    selection: request.selection,
    #[cfg(target_os = "windows")]
    surface: crate::editor::preview_platform::recording_export_surface,
    timeline: request.timeline,
    output: request.output,
    video: request.video,
    width: request.width,
  })? {
    media_preview::ExportRunResult::Completed => Ok(Some(path)),
    media_preview::ExportRunResult::Cancelled => Ok(None),
  }
}
