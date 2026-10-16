// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use screenwide_output::Capture;
use screenwide_scenes::CameraOverlaySettings;

/// The subtitle file an export writes beside its video, named as it is.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum ExportSubtitles {
  #[default]
  None,
  Srt,
  Vtt,
}

impl ExportSubtitles {
  pub fn file(self) -> Option<screenwide_text::captions::SubtitleFile> {
    match self {
      Self::None => None,
      Self::Srt => Some(screenwide_text::captions::SubtitleFile::Srt),
      Self::Vtt => Some(screenwide_text::captions::SubtitleFile::Vtt),
    }
  }
}

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
  pub subtitles: ExportSubtitles,
  #[serde(default)]
  pub timeline_edit: Option<screenwide_timeline_edit::RecordingTimelineEdit>,
  /// The captions drawn into the picture, made from `timeline_edit` and the
  /// transcript as the export starts; never sent by the editor.
  #[serde(skip)]
  pub captions: Option<screenwide_text::captions::CaptionTrack>,
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
