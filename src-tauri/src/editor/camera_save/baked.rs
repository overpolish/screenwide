// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use screenwide_export::cursor_export;
use screenwide_export::media_preview;
use screenwide_export::track_selection;
use screenwide_scenes::CameraOverlaySettings;

#[allow(clippy::too_many_arguments)]
pub(in crate::editor) fn save_baked_recording(
  screen: &Path,
  camera: &RecordingCamera,
  directory: &Path,
  stem: &str,
  selection: &track_selection::TrackSelection,
  layout: track_selection::AudioLayout,
  artifact_id: u64,
  duration_ms: u64,
  screen_size: (u32, u32),
  overlay: CameraOverlaySettings,
  camera_drop_shadow: bool,
  camera_on_top: bool,
  video_settings: (u8, u16, u16),
  cursor: Option<(&Path, screenwide_cursor_effects::CursorEffectSettings)>,
  keyboard: Option<(&Path, screenwide_keyboard_effects::KeyboardEffectSettings)>,
  output: &ScreenshotOutputSettings,
  progress_app: &AppHandle,
  cancelled: &AtomicBool,
  timeline: Option<&screenwide_timeline_edit::TimelinePlan>,
  camera_effects: Option<
    std::sync::Arc<screenwide_export::camera_effects::live::LiveCameraEffects>,
  >,
  captions: Option<&screenwide_captions::CaptionTrack>,
) -> Result<Option<PathBuf>, String> {
  let progress_duration_ms = timeline.map_or(
    duration_ms,
    screenwide_timeline_edit::TimelinePlan::duration_ms,
  );
  let mut on_progress = |processed_ms| {
    emit_progress(
      progress_app,
      artifact_id,
      "recording",
      processed_ms,
      progress_duration_ms,
      0.0,
      99.0,
    );
  };
  let path = unique_path(directory, stem, RECORDING_EXTENSION, &|candidate| {
    candidate.exists()
  });
  let baked = media_preview::BakedVideoExportOptions {
    camera_drop_shadow,
    camera_height: camera.height,
    camera_width: camera.width,
    overlay,
    screen_height: output.height,
    screen_width: output.width,
    video: media_preview::VideoExportOptions {
      compression: video_settings.0,
      // Camera geometry is calculated at the reference size. The requested
      // output/source scales are passed separately to the compositor below.
      resolution_scale_percent: 100,
      source_scale_percent: 100,
    },
  };
  let (cursor_path, cursor_effects) = cursor.map_or(
    (
      None,
      screenwide_cursor_effects::CursorEffectSettings::default(),
    ),
    |(path, settings)| (Some(path), settings),
  );
  let (keyboard_path, keyboard_effects) = keyboard.map_or(
    (
      None,
      screenwide_keyboard_effects::KeyboardEffectSettings::default(),
    ),
    |(path, settings)| (Some(path), settings),
  );
  let result = cursor_export::export(cursor_export::CursorExportRequest {
    annotation_track: screenwide_annotations::timing::AnnotationTrack::Primary,
    audio_layout: layout,
    audio_source: None,
    camera: Some((&camera.path, baked)),
    camera_effects: camera_effects.map(|live| cursor_export::ExportCameraEffects {
      live,
      on_screen: false,
    }),
    camera_on_top,
    cancelled,
    captions,
    cursor: cursor_path,
    cursor_effects,
    keyboard: keyboard_path,
    keyboard_effects,
    destination: &path,
    duration_ms,
    height: screen_size.1,
    on_progress: &mut on_progress,
    output,
    screen,
    selection,
    #[cfg(target_os = "windows")]
    surface: crate::editor::preview_platform::recording_export_surface,
    timeline,
    video: media_preview::VideoExportOptions {
      compression: video_settings.0,
      resolution_scale_percent: video_settings.1,
      source_scale_percent: video_settings.2,
    },
    width: screen_size.0,
  })?;
  match result {
    media_preview::ExportRunResult::Completed => {
      emit_progress(
        progress_app,
        artifact_id,
        "finalizing",
        progress_duration_ms,
        progress_duration_ms,
        0.0,
        99.0,
      );
      if !path.is_file() {
        let _ = std::fs::remove_file(&path);
        return Err("The exported recording did not finish publishing".to_owned());
      }
      Ok(Some(path))
    }
    media_preview::ExportRunResult::Cancelled => Ok(None),
  }
}
