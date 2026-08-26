// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenshotItemSnapshot {
  pub height: u32,
  pub id: u64,
  pub width: u32,
}

/// What the window is told about the pending artifact. Deliberately without
/// pixels: the preview travels separately, as bytes.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(
  rename_all = "camelCase",
  rename_all_fields = "camelCase",
  tag = "kind"
)]
pub enum ExportArtifactSnapshot {
  Screenshot {
    id: u64,
    items: Vec<ScreenshotItemSnapshot>,
    suggested_file_stem: String,
    extension: String,
    width: u32,
    height: u32,
  },
  Recording {
    audio_tracks: Vec<RecordingAudioTrack>,
    camera: Option<RecordingCamera>,
    can_compress: bool,
    cursor_data_version: Option<u16>,
    has_cursor_data: bool,
    keyboard_data_version: Option<u16>,
    keyboard_maximum_width_units: Option<u16>,
    has_keyboard_data: bool,
    id: u64,
    suggested_file_stem: String,
    extension: String,
    width: u32,
    height: u32,
    duration_ms: u64,
    original_size_bytes: u64,
    /// The working file, for the window to play through the asset protocol.
    /// Scoped to the recordings directory in `tauri.conf.json`, which is the
    /// only place this path can ever point.
    path: PathBuf,
    primary_kind: PrimaryRecordingKind,
    source_scale_percent: u16,
    timeline_edit: Option<timeline_edit::RecordingTimelineEdit>,
    timeline_edit_revision: Option<u64>,
  },
}

pub(super) fn snapshot(app: &AppHandle, kind: ExportKind) -> ExportSnapshot {
  let state = app.state::<ExportState>();
  let artifact = state
    .slot(kind)
    .artifact
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .as_ref()
    .map(|artifact| match artifact {
      ExportArtifact::Screenshot {
        id,
        items,
        suggested_file_stem,
      } => ExportArtifactSnapshot::Screenshot {
        id: *id,
        items: items
          .iter()
          .map(|item| ScreenshotItemSnapshot {
            height: item.image.height,
            id: item.id,
            width: item.image.width,
          })
          .collect(),
        suggested_file_stem: suggested_file_stem.clone(),
        extension: SCREENSHOT_EXTENSION.to_owned(),
        width: items.first().map_or(0, |item| item.image.width),
        height: items.first().map_or(0, |item| item.image.height),
      },
      ExportArtifact::Recording {
        audio_tracks,
        camera,
        cursor,
        keyboard,
        duration_ms,
        height,
        id,
        path,
        primary_kind,
        source_scale_percent,
        suggested_file_stem,
        width,
      } => {
        let (timeline_edit_revision, timeline_edit) = timeline_edit::snapshot_fields(path, *id);
        ExportArtifactSnapshot::Recording {
          audio_tracks: audio_tracks.clone(),
          camera: camera.clone(),
          can_compress: *primary_kind != PrimaryRecordingKind::Audio
            && media_preview::supports_compression(),
          cursor_data_version: cursor.as_ref().map(|cursor| cursor.format_version),
          has_cursor_data: cursor.is_some(),
          keyboard_data_version: keyboard.as_ref().map(|keyboard| keyboard.format_version),
          keyboard_maximum_width_units: keyboard.as_ref().map(|value| value.maximum_width_units),
          has_keyboard_data: keyboard.is_some(),
          id: *id,
          suggested_file_stem: suggested_file_stem.clone(),
          extension: if *primary_kind == PrimaryRecordingKind::Audio {
            AUDIO_EXTENSION.to_owned()
          } else {
            delivered_extension(path, media_preview::remuxer().is_some()).to_owned()
          },
          width: *width,
          height: *height,
          duration_ms: *duration_ms,
          original_size_bytes: std::fs::metadata(path).map_or(0, |metadata| metadata.len())
            + camera
              .as_ref()
              .and_then(|camera| std::fs::metadata(&camera.path).ok())
              .map_or(0, |metadata| metadata.len())
            + recording_sidecar::total_size(cursor.as_ref(), keyboard.as_ref()),
          path: path.clone(),
          primary_kind: *primary_kind,
          source_scale_percent: *source_scale_percent,
          timeline_edit,
          timeline_edit_revision,
        }
      }
    });

  let screenshot_radius_percent = *state
    .screenshot_radius_percent
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  let screenshot_background_radius_percent = *state
    .screenshot_background_radius_percent
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  let screenshot_output = state
    .screenshot_output
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .clone();
  let cursor_effects = *state
    .cursor_effects
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  let recording_output = state
    .recording_output
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .clone();
  ExportSnapshot {
    artifact,
    cursor_effects,
    directory: current_directory(app, kind),
    recording_output,
    screenshot_radius_percent,
    screenshot_background_radius_percent,
    screenshot_output,
    workspace: kind,
  }
}
