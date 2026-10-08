// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use ts_rs::TS;

#[derive(Clone, Debug, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ScreenshotItemSnapshot {
  /// The annotations the item starts with, in its own pixels.
  pub annotations: Vec<Annotation>,
  pub height: u32,
  pub id: u64,
  pub width: u32,
}

/// What the window is told about the pending artifact. Deliberately without
/// pixels: the preview travels separately, as bytes.
#[derive(Clone, Debug, PartialEq, Serialize, TS)]
#[serde(
  rename_all = "camelCase",
  rename_all_fields = "camelCase",
  tag = "kind"
)]
#[ts(export, rename = "EditorArtifact")]
// Built once per artifact and serialised straight away, so the variant's size
// never matters; it only crosses clippy's line on Windows, where `PathBuf` is
// wider.
#[allow(clippy::large_enum_variant)]
pub enum EditorArtifactSnapshot {
  Screenshot {
    id: u64,
    items: Vec<ScreenshotItemSnapshot>,
    /// The canvas and layers saved in the project, which the window shows in
    /// place of the remembered look.
    #[ts(as = "Option<crate::editor::ScreenshotWorkspaceOutputSettings>", optional = nullable)]
    project_workspace: Option<serde_json::Value>,
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
    #[ts(optional = nullable)]
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
    /// The look saved in the project, which the window shows in place of the
    /// remembered one.
    #[ts(optional = nullable)]
    project_look: Option<super::project_look::ProjectLook>,
    primary_kind: PrimaryRecordingKind,
    source_scale_percent: u16,
    #[ts(optional = nullable)]
    timeline_edit: Option<timeline_edit::RecordingTimelineEdit>,
    #[ts(optional = nullable)]
    timeline_edit_revision: Option<u64>,
  },
}

pub(super) fn snapshot(app: &AppHandle, kind: EditorKind) -> EditorSnapshot {
  let state = app.state::<EditorState>();
  let artifact = state
    .slot(kind)
    .artifact
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .as_ref()
    .map(|artifact| match artifact {
      EditorArtifact::Screenshot {
        id,
        items,
        project,
        suggested_file_stem,
      } => EditorArtifactSnapshot::Screenshot {
        id: *id,
        items: items
          .iter()
          .map(|item| ScreenshotItemSnapshot {
            annotations: item.annotations.clone(),
            height: item.image.height,
            id: item.id,
            width: item.image.width,
          })
          .collect(),
        project_workspace: super::screenshot_project::restored_workspace(
          project,
          &items.iter().map(|item| item.id).collect::<Vec<_>>(),
        ),
        suggested_file_stem: suggested_file_stem.clone(),
        extension: SCREENSHOT_EXTENSION.to_owned(),
        width: items.first().map_or(0, |item| item.image.width),
        height: items.first().map_or(0, |item| item.image.height),
      },
      EditorArtifact::Recording {
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
        suggested_file_stem,
        width,
      } => {
        let (timeline_edit_revision, timeline_edit) = timeline_edit::snapshot_fields(project, *id);
        EditorArtifactSnapshot::Recording {
          audio_tracks: audio_tracks.clone(),
          camera: camera.clone(),
          can_compress: *primary_kind != PrimaryRecordingKind::Audio
            && media_preview::supports_compression(),
          cursor_data_version: cursor.as_ref().map(|cursor| cursor.format_version),
          has_cursor_data: cursor.is_some(),
          keyboard_data_version: keyboard.as_ref().map(|keyboard| keyboard.format_version),
          keyboard_maximum_width_units: keyboard.as_ref().map(|value| value.maximum_width_units),
          has_keyboard_data: keyboard
            .as_ref()
            .is_some_and(|keyboard| keyboard.has_shortcuts),
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
          project_look: super::project_look::for_project(project),
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
  let keyboard_effects = *state
    .keyboard_effects
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  let recording_export_choices = *state
    .recording_choices
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  let recording_output = state
    .recording_output
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .clone();
  let screenshot_delete_project_after_export = *state
    .screenshot_delete_project_after_export
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  EditorSnapshot {
    artifact,
    cursor_effects,
    directory: current_directory(app, kind),
    keyboard_effects,
    recording_export_choices,
    recording_output,
    screenshot_radius_percent,
    screenshot_background_radius_percent,
    screenshot_output,
    screenshot_delete_project_after_export,
    workspace: kind,
  }
}
