// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AudioTrackKind {
  SystemAudio,
  Microphone,
  Unknown,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingAudioTrack {
  pub kind: AudioTrackKind,
  pub label: String,
  pub stream_index: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingCamera {
  pub duration_ms: u64,
  pub height: u32,
  pub original_size_bytes: u64,
  pub path: PathBuf,
  pub width: u32,
}

/// A baked camera's placement, in the screen output's own pixels: the camera
/// image by its centre and width, and the crop window that frames it.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CameraOverlaySettings {
  pub camera_x: f64,
  pub camera_y: f64,
  pub camera_width: f64,
  pub frame_height: f64,
  pub frame_width: f64,
  pub frame_x: f64,
  pub frame_y: f64,
  pub radius_percent: f64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RecordingExportOptions {
  pub audio_track_volumes: Vec<AudioTrackVolume>,
  pub bake_camera: bool,
  pub camera_compression: u8,
  pub camera_overlay: CameraOverlaySettings,
  pub camera_resolution_scale_percent: u16,
  pub collapse_audio: bool,
  pub compression: u8,
  pub cursor_effects: cursor_effects::CursorEffectSettings,
  pub keyboard_effects: keyboard_effects::KeyboardEffectSettings,
  pub enabled_stream_indices: Vec<usize>,
  pub include_camera: bool,
  pub include_primary_video: bool,
  pub resolution_scale_percent: u16,
  pub recording_output: RecordingOutputSettings,
  pub screenshot_output: ScreenshotWorkspaceOutputSettings,
  #[serde(default)]
  pub timeline_edit: Option<timeline_edit::RecordingTimelineEdit>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingOutputSettings {
  pub camera: ScreenshotOutputSettings,
  #[serde(default = "default_camera_on_top")]
  pub camera_on_top: bool,
  pub primary: ScreenshotOutputSettings,
}

impl RecordingOutputSettings {
  /// Writes each pane's source width in logical points, primary then camera,
  /// from [`EditorArtifact::capture_widths`]. The webview's settings never
  /// carry them, so they are stamped before a composition is compared or used.
  pub(crate) fn stamp_capture_widths(&mut self, widths: [f64; 2]) {
    self.primary.capture_width_points = widths[0];
    self.camera.capture_width_points = widths[1];
  }
}

impl EditorArtifact {
  /// A recording's pane widths in logical points, which redactions are sized
  /// in, primary then camera: the screen's pixels over the scale it was
  /// recorded at, and a camera's own pixels. Zero where there is no pane.
  pub(crate) fn capture_widths(&self) -> [f64; 2] {
    let EditorArtifact::Recording {
      camera,
      source_scale_percent,
      width,
      ..
    } = self
    else {
      return [0.0; 2];
    };
    [
      f64::from(*width) * 100.0 / f64::from((*source_scale_percent).max(1)),
      camera
        .as_ref()
        .map_or(0.0, |camera| f64::from(camera.width)),
    ]
  }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AudioTrackVolume {
  pub decibels: i16,
  pub stream_index: usize,
}

fn default_camera_on_top() -> bool {
  true
}
