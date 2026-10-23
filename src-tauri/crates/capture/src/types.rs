// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#![cfg_attr(not(target_os = "macos"), allow(dead_code))]

use std::{path::PathBuf, sync::Arc};

use serde::{Deserialize, Serialize};
use tauri::{LogicalPosition, LogicalSize};
use ts_rs::TS;

use super::encoding::FailureReport;

/// The frame rate a recording takes unless Settings says otherwise.
pub const DEFAULT_FPS: u32 = 60;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum RecordingStatus {
  #[default]
  Idle,
  Starting,
  Recording,
  Paused,
  Stopping,
}

impl RecordingStatus {
  pub const fn label(self) -> &'static str {
    match self {
      Self::Idle => "idle",
      Self::Starting => "starting",
      Self::Recording => "recording",
      Self::Paused => "paused",
      Self::Stopping => "stopping",
    }
  }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum RecordingMode {
  Screen,
  Region,
  Window,
  Camera,
  Audio,
}

#[derive(Clone, Copy, Debug, Deserialize, TS)]
#[ts(export)]
pub struct Region {
  #[ts(as = "crate::types::Position")]
  pub position: LogicalPosition<f64>,
  #[ts(as = "crate::types::Size")]
  pub size: LogicalSize<f64>,
}

/// Options assembled by the recording controls from the source and input stores.
#[derive(Clone, Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StartRecordingOptions {
  pub mode: RecordingMode,
  #[serde(default)]
  pub monitor_id: Option<u32>,
  #[serde(default)]
  pub window_id: Option<u32>,
  #[serde(default)]
  pub region: Option<Region>,
  #[serde(default)]
  pub show_cursor: bool,
  #[serde(default)]
  pub capture_keyboard_shortcuts: bool,
  #[serde(default)]
  pub system_audio: bool,
  #[serde(default)]
  pub system_audio_application_ids: Vec<String>,
  #[serde(default)]
  pub system_audio_process_ids: Vec<u32>,
  #[serde(default)]
  pub microphone_id: Option<String>,
  #[serde(default)]
  pub camera_id: Option<String>,
  #[serde(default)]
  pub camera_width: Option<u32>,
  #[serde(default)]
  pub camera_height: Option<u32>,
  #[serde(default)]
  pub camera_fps: Option<u32>,
  #[serde(default)]
  pub camera_flipped: bool,
  /// Anti-flicker for 50 Hz mains (PAL): the camera runs at a PAL cadence on
  /// macOS and has its power line frequency control set on Windows.
  #[serde(default)]
  pub camera_pal: bool,
  #[serde(default = "default_fps")]
  pub fps: u32,
}

#[derive(Clone, Debug)]
pub struct CameraCaptureMode {
  pub device_id: String,
  pub flipped: bool,
  pub fps: u32,
  pub height: u32,
  #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
  pub pal: bool,
  pub width: u32,
}

/// Platform-neutral description of the primary video source. Native capture
/// adapters translate this intent into ScreenCaptureKit/AVFoundation on macOS
/// and Windows Graphics Capture/Media Foundation on Windows.
pub enum PrimaryCaptureSource {
  Screen {
    fps: u32,
    monitor_id: u32,
    show_cursor: bool,
  },
  Region {
    fps: u32,
    monitor_id: u32,
    region: Region,
    show_cursor: bool,
  },
  Window {
    fps: u32,
    show_cursor: bool,
    window_id: u32,
  },
  Camera,
  Audio,
}

/// Everything a native capture adapter needs to open one recording. Keeping
/// this contract free of framework objects lets each platform use its own
/// capture stack while sharing lifecycle, timing and export semantics.
pub struct CaptureStartupConfig {
  pub camera: Option<CameraCaptureMode>,
  pub camera_path: Option<PathBuf>,
  /// Keeps Screenwide's own windows in the picture instead of hiding them,
  /// which is how the app records demos of itself.
  pub include_own_windows: bool,
  pub microphone_id: Option<String>,
  pub monitor: Arc<super::monitor::RecordingMonitor>,
  pub on_failure: FailureReport,
  pub path: PathBuf,
  pub primary: PrimaryCaptureSource,
  pub system_audio: SystemAudioSelection,
}

/// A source snapshot taken when Record is pressed. Bundle identifiers resolve
/// ScreenCaptureKit application filters on macOS; process IDs identify WASAPI
/// loopback sessions on Windows.
#[derive(Clone, Debug, Default)]
pub struct SystemAudioSelection {
  pub application_ids: Vec<String>,
  pub enabled: bool,
  /// Reserved for the Windows WASAPI adapter, which selects audio sessions by
  /// process rather than by ScreenCaptureKit bundle identifier.
  #[allow(dead_code)]
  pub process_ids: Vec<u32>,
}

const fn default_fps() -> u32 {
  DEFAULT_FPS
}

/// Epoch-millisecond timestamps are stamped by Rust so every window - including
/// ones that reload or join late - derives the same elapsed time.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RecordingSnapshot {
  pub status: RecordingStatus,
  pub mode: Option<RecordingMode>,
  pub countdown_seconds_remaining: u8,
  pub started_at_ms: Option<u64>,
  pub accumulated_ms: u64,
  pub paused_at_ms: Option<u64>,
}

#[derive(Clone, Copy, Debug, Serialize, TS)]
#[ts(export)]
pub struct Position {
  pub x: i32,
  pub y: i32,
}

#[derive(Clone, Copy, Debug, Serialize, TS)]
#[ts(export)]
pub struct Size {
  pub width: u32,
  pub height: u32,
}
