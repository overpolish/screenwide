// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use screenwide_output::Capture;
use screenwide_scenes::CameraOverlaySettings;

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RecordingExportOptions {
  pub audio_track_volumes: Vec<screenwide_export::recording_model::AudioTrackVolume>,
  pub bake_camera: bool,
  pub camera_compression: u8,
  pub camera_overlay: CameraOverlaySettings,
  pub camera_resolution_scale_percent: u16,
  pub collapse_audio: bool,
  pub compression: u8,
  /// Moves the project to the Trash once the export has been published.
  #[serde(default)]
  pub delete_project_after_export: bool,
  pub cursor_effects: screenwide_cursor_effects::CursorEffectSettings,
  pub keyboard_effects: screenwide_keyboard_effects::KeyboardEffectSettings,
  pub enabled_stream_indices: Vec<usize>,
  pub include_camera: bool,
  pub include_primary_video: bool,
  pub resolution_scale_percent: u16,
  pub recording_output: RecordingOutputSettings,
  pub screenshot_output: ScreenshotWorkspaceOutputSettings,
  #[serde(default)]
  pub timeline_edit: Option<screenwide_timeline_edit::RecordingTimelineEdit>,
}

impl EditorArtifact {
  /// What a recording's panes were captured at, primary then camera, which
  /// redactions and annotation sizes are measured against: the screen at the
  /// scale it was recorded at, and a camera at one point to its pixel. Zero
  /// where there is no pane.
  pub(crate) fn captures(&self) -> [Capture; 2] {
    let EditorArtifact::Recording {
      camera,
      source_scale_percent,
      width,
      ..
    } = self
    else {
      return [Capture::default(); 2];
    };
    let scale = f64::from((*source_scale_percent).max(1)) / 100.0;
    [
      Capture {
        width_points: f64::from(*width) / scale,
        scale,
      },
      camera
        .as_ref()
        .map_or_else(Capture::default, |camera| Capture {
          width_points: f64::from(camera.width),
          scale: 1.0,
        }),
    ]
  }
}
