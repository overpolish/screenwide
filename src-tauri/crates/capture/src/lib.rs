// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Capturing a recording: the screen, the camera, the microphone and the
//! system audio on each platform, encoded into the movie, with the cursor and
//! keyboard written beside it and the replay buffer kept warm. The app starts,
//! pauses and stops it, and owns the windows and state around it.

pub mod camera_format;
#[cfg(target_os = "macos")]
pub mod camera_frame_rate;
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub mod camera_frames;
#[cfg(target_os = "windows")]
pub mod camera_power_line;
pub mod capture_geometry;
#[cfg(target_os = "macos")]
pub mod capture_kit;
pub mod clock;
pub mod cursor;
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub mod desktop_canvas;
pub mod desktop_capture;
pub mod devices;
pub mod encoding;
pub mod keyboard;
pub mod microphone;
pub mod monitor;
pub mod note_gate;
#[cfg(target_os = "macos")]
pub mod platform;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub mod platform_unsupported;
#[cfg(target_os = "windows")]
pub mod platform_windows;
pub mod replay_ring;
pub mod sidecar_output;
pub mod types;

/// The platform's capture backend.
#[cfg(target_os = "macos")]
pub use platform as capture;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub use platform_unsupported as capture;
#[cfg(target_os = "windows")]
pub use platform_windows as capture;

pub use encoding::{CameraFinalizeInfo, FinalizeInfo};
pub use types::{
  CameraCaptureMode, CaptureStartupConfig, PrimaryCaptureSource, RecordingMode, RecordingSnapshot,
  RecordingStatus, Region, StartRecordingOptions, SystemAudioSelection,
};
