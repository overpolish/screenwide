// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A recording's tracks and output settings as an export reads them.

use std::path::PathBuf;

use screenwide_output::{Capture, ScreenshotOutputSettings};
use serde::{Deserialize, Serialize};
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
      Self::SystemAudio => screenwide_i18n::t!("editor-timeline-system-audio"),
      Self::Microphone => screenwide_i18n::t!("editor-timeline-microphone"),
      Self::Unknown => {
        screenwide_i18n::t!("editor-timeline-audio-track", number = stream_index + 1)
      }
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

/// The camera is drawn in front of the screen. Which layer is in front is
/// part of a composition's layout, not a setting a recording carries.
pub const CAMERA_IN_FRONT: bool = true;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RecordingOutputSettings {
  pub camera: ScreenshotOutputSettings,
  pub primary: ScreenshotOutputSettings,
}

impl RecordingOutputSettings {
  /// Writes each pane's capture, primary then camera, from
  /// the editor's captures. The webview's settings never carry it, so
  /// it is stamped before a composition is compared or used.
  pub fn stamp_captures(&mut self, captures: [Capture; 2]) {
    self.primary.stamp_capture(captures[0]);
    self.camera.stamp_capture(captures[1]);
  }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AudioTrackVolume {
  pub decibels: i16,
  pub stream_index: usize,
}
