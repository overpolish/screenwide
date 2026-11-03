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

/// What fills a camera saved as a file of its own where its background is
/// taken away. MP4 has no transparency, so the room is filled with a screen
/// colour another editor keys out: green, or blue for someone wearing green.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum CameraKey {
  #[default]
  Green,
  Blue,
}

impl CameraKey {
  /// The broadcast chroma colours, which keyers are tuned to.
  pub const fn color(self) -> &'static str {
    match self {
      Self::Green => "#00b140",
      Self::Blue => "#0047bb",
    }
  }

  /// `output` with its canvas filled with the key colour: where the camera's
  /// background is taken away, the canvas is what shows.
  pub fn keyed(self, output: &ScreenshotOutputSettings) -> ScreenshotOutputSettings {
    ScreenshotOutputSettings {
      background_color: self.color().to_owned(),
      background_image_path: None,
      background_radius_percent: 0.0,
      background_type: "solid".to_owned(),
      ..output.clone()
    }
  }
}

/// The picture file a screenshot is saved as.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum ScreenshotFormat {
  #[default]
  Png,
  Jpeg,
  Webp,
}

impl ScreenshotFormat {
  pub const fn extension(self) -> &'static str {
    match self {
      Self::Png => "png",
      Self::Jpeg => "jpg",
      Self::Webp => "webp",
    }
  }

  pub fn encode(self, image: &CapturedImage) -> Result<Vec<u8>, String> {
    match self {
      Self::Png => screenwide_output::encode_png(image),
      Self::Jpeg => screenwide_output::encode_jpeg(image),
      Self::Webp => screenwide_output::encode_webp(image),
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
  /// What fills a separate camera file where its background is taken away.
  #[serde(default)]
  pub camera_key: CameraKey,
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
  /// The file a screenshot is saved as.
  #[serde(default)]
  pub screenshot_format: ScreenshotFormat,
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn a_keyed_camera_canvas_is_the_screen_colour_alone() {
    let mut output = screenwide_output::fixtures::test_output_settings(640, 360);
    output.background_type = "image".to_owned();
    output.background_image_path = Some("/tmp/room.png".to_owned());
    output.background_radius_percent = 4.0;
    for (key, color) in [(CameraKey::Green, "#00b140"), (CameraKey::Blue, "#0047bb")] {
      let keyed = key.keyed(&output);
      assert_eq!(keyed.background_type, "solid");
      assert_eq!(keyed.background_color, color);
      assert_eq!(keyed.background_image_path, None);
      assert_eq!(keyed.background_radius_percent, 0.0);
      assert_eq!(keyed.width, output.width);
      assert_eq!(keyed.crop_width, output.crop_width);
    }
  }
}
