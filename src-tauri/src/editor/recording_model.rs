// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::screenshots::Capture;
use ts_rs::TS;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum AudioTrackKind {
  SystemAudio,
  Microphone,
  Unknown,
}

impl AudioTrackKind {
  /// The track's name in the editor, in the app's language. Built when a
  /// recording is presented, never stored, so a language change applies to
  /// every recording the next time it opens.
  pub fn label(self, stream_index: usize) -> String {
    match self {
      Self::SystemAudio => crate::i18n::t!("editor-timeline-system-audio"),
      Self::Microphone => crate::i18n::t!("editor-timeline-microphone"),
      Self::Unknown => crate::i18n::t!("editor-timeline-audio-track", number = stream_index + 1),
    }
  }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RecordingAudioTrack {
  pub kind: AudioTrackKind,
  pub label: String,
  pub stream_index: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RecordingCamera {
  pub duration_ms: u64,
  pub height: u32,
  pub original_size_bytes: u64,
  pub path: PathBuf,
  pub width: u32,
}

/// A baked camera's placement, in the screen output's own pixels: the camera
/// image by its centre and width, and the crop window that frames it.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
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
  /// Moves the project to the Trash once the export has been published.
  #[serde(default)]
  pub delete_project_after_export: bool,
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

/// The camera is drawn in front of the screen. Which layer is in front is
/// part of a composition's layout, not a setting a recording carries.
pub(crate) const CAMERA_IN_FRONT: bool = true;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RecordingOutputSettings {
  pub camera: ScreenshotOutputSettings,
  pub primary: ScreenshotOutputSettings,
}

impl RecordingOutputSettings {
  /// Writes each pane's capture, primary then camera, from
  /// [`EditorArtifact::captures`]. The webview's settings never carry it, so
  /// it is stamped before a composition is compared or used.
  pub(crate) fn stamp_captures(&mut self, captures: [Capture; 2]) {
    self.primary.stamp_capture(captures[0]);
    self.camera.stamp_capture(captures[1]);
  }
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

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AudioTrackVolume {
  pub decibels: i16,
  pub stream_index: usize,
}
