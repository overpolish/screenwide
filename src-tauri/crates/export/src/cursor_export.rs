// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Final-video composition.
//!
//! Each frame is drawn on the shared GPU device by the same compositor the
//! preview and stills use. On macOS, AVFoundation decodes the screen into
//! Core Video buffers the compositor reads in place and writes the encoder's
//! buffers in place; on Windows, Media Foundation does the same through
//! Direct3D 11. Selected audio is stream-copied or mixed afterwards without
//! decoding the finished video.

use std::{path::Path, sync::atomic::AtomicBool};

use super::media_preview::{self, BakedVideoExportOptions, ExportRunResult, VideoExportOptions};
use super::track_selection::{AudioLayout, TrackSelection};
use screenwide_cursor_effects::CursorEffectSettings;
use screenwide_output::ScreenshotOutputSettings;

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(super) mod camera_effects;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod export_output;
#[cfg(all(test, any(target_os = "macos", target_os = "windows")))]
mod format_benchmark_tests;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod frame_annotations;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod frame_clock;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod frame_draw;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod frame_scene;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod progress_pace;
#[cfg(all(test, any(target_os = "macos", target_os = "windows")))]
mod project_benchmark_tests;
#[cfg(all(test, any(target_os = "macos", target_os = "windows")))]
mod video_tests;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "macos")]
use self::macos::platform;
#[cfg(target_os = "windows")]
use self::windows::platform;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod platform_unsupported;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
use self::platform_unsupported as platform;

#[cfg_attr(not(any(target_os = "macos", target_os = "windows")), allow(dead_code))]
pub struct CursorExportRequest<'a> {
  pub annotation_track: screenwide_annotations::timing::AnnotationTrack,
  pub audio_layout: AudioLayout,
  pub audio_source: Option<&'a Path>,
  pub camera: Option<(&'a Path, BakedVideoExportOptions)>,
  pub camera_effects: Option<ExportCameraEffects>,
  pub camera_on_top: bool,
  pub cancelled: &'a AtomicBool,
  /// The captions drawn over the picture, timed on the edited video.
  pub captions: Option<&'a screenwide_text::captions::CaptionTrack>,
  pub cursor: Option<&'a Path>,
  pub cursor_effects: CursorEffectSettings,
  pub keyboard: Option<&'a Path>,
  pub keyboard_effects: screenwide_keyboard_effects::KeyboardEffectSettings,
  pub destination: &'a Path,
  pub duration_ms: u64,
  pub height: u32,
  pub on_progress: &'a mut dyn FnMut(u64),
  pub screen: &'a Path,
  pub selection: &'a TrackSelection,
  pub timeline: Option<&'a screenwide_timeline_edit::TimelinePlan>,
  pub output: &'a ScreenshotOutputSettings,
  pub video: VideoExportOptions,
  pub width: u32,
}

/// The camera's effects as an export draws them, and on which picture: the
/// baked-in camera, or the screen where the screen is the camera.
#[derive(Clone)]
pub struct ExportCameraEffects {
  pub live: std::sync::Arc<super::camera_effects::live::LiveCameraEffects>,
  pub on_screen: bool,
}

fn video_bitrate(width: u32, height: u32, compression: u8) -> u64 {
  // Bits per pixel per frame. Mesh gradient backgrounds are the hardest
  // content H.264 sees here: starving them visibly bands and blocks, so the
  // ladder is sized for smooth gradients over text rather than flat UI.
  let quality = [0.1, 0.065, 0.036, 0.02, 0.011]
    .get(compression as usize)
    .copied()
    .unwrap_or(0.011);
  // Perceptual quality does not scale linearly with resolution: a downscaled
  // export needs more bits per pixel than a native-resolution one to look
  // equally clean, so smaller outputs get a gentle density boost relative to
  // a 4K reference.
  let pixels = f64::from(width) * f64::from(height);
  let density_boost = (8_294_400.0 / pixels.max(1.0)).powf(0.25).clamp(1.0, 1.8);
  let pixels_per_second = pixels * 60.0;
  (pixels_per_second * quality * density_boost)
    .round()
    .max(2_000_000.0) as u64
}

pub fn export(request: CursorExportRequest<'_>) -> Result<ExportRunResult, String> {
  export_formats(vec![request])
}

/// The most formats [`export_formats`] should be asked to draw at once.
/// Windows encodes each on a hardware encoder session of its own, and
/// NVIDIA's consumer drivers have allowed as few as three at a time; x264
/// has no such limit.
#[cfg(target_os = "windows")]
pub const FORMATS_PER_PASS: usize = 3;
#[cfg(not(target_os = "windows"))]
pub const FORMATS_PER_PASS: usize = usize::MAX;

/// Several formats of one recording, each a file of its own, drawn from one
/// decode of it: every request reads the same screen and camera at the same
/// moments, and differs in what it draws, where, and what it is written
/// with. The first request's progress and cancel stand for all. Either every
/// file is written or none is.
pub fn export_formats(requests: Vec<CursorExportRequest<'_>>) -> Result<ExportRunResult, String> {
  for request in &requests {
    check_request(request)?;
  }
  check_one_recording(&requests)?;
  platform::export_formats(requests)
}

/// What `request` reads of the recording and when.
type Reading<'r> = (
  &'r Path,
  (u32, u32),
  u64,
  Option<&'r Path>,
  Option<&'r [screenwide_timeline::TimelineRange]>,
);

fn reading<'r>(request: &CursorExportRequest<'r>) -> Reading<'r> {
  (
    request.screen,
    (request.width, request.height),
    request.duration_ms,
    request.camera.map(|(path, _)| path),
    request
      .timeline
      .map(screenwide_timeline_edit::TimelinePlan::ranges),
  )
}

/// Whether `requests` read one recording the same way, which a single decode
/// of it can serve.
fn check_one_recording(requests: &[CursorExportRequest<'_>]) -> Result<(), String> {
  let Some((first, rest)) = requests.split_first() else {
    return Err("There is no format to export".to_owned());
  };
  if rest
    .iter()
    .any(|request| reading(request) != reading(first))
  {
    return Err("The formats do not read the recording the same way".to_owned());
  }
  Ok(())
}

/// The frame at `output_us` of `request`'s edited video, written to `png` as
/// `picture` asks: as the platform's encoder leaves it at the request's size
/// and compression, see `media_preview::PreviewStretch`, or as drawn. The
/// request's destination is never written.
pub fn preview_frame(
  request: CursorExportRequest<'_>,
  output_us: u64,
  picture: media_preview::PreviewPicture,
  png: &Path,
) -> Result<ExportRunResult, String> {
  check_request(&request)?;
  platform::preview_frame(request, output_us, picture, png)
}

/// How `request`'s edited video lays out its source.
#[cfg(any(target_os = "macos", target_os = "windows"))]
fn edited_ranges(request: &CursorExportRequest<'_>) -> Vec<screenwide_timeline::TimelineRange> {
  media_preview::edited_ranges(request.timeline, request.duration_ms)
}

fn check_request(request: &CursorExportRequest<'_>) -> Result<(), String> {
  let scene_clips = request
    .timeline
    .map_or(&[][..], |timeline| &timeline.scene_clips);
  if request.cursor.is_some()
    && !screenwide_annotations::timing::cursor_shown_anywhere(
      scene_clips,
      request.cursor_effects.bake,
    )
  {
    return Err("Cursor baking was not enabled".to_owned());
  }
  if request.keyboard.is_some() && !request.keyboard_effects.bake {
    return Err("Keyboard baking was not enabled".to_owned());
  }
  if !media_preview::supports_compression() {
    return Err("This FFmpeg build cannot finish the recording export".to_owned());
  }
  Ok(())
}

pub fn needs_composition(
  settings: &ScreenshotOutputSettings,
  source_width: u32,
  source_height: u32,
) -> bool {
  settings.width != source_width
    || settings.height != source_height
    || settings.background_radius_percent > 0.0
    || !settings.radius.is_zero()
    || settings.capture_corner_radius > 0.0
    || (settings.crop_height - f64::from(settings.height)).abs() > 0.000_001
    || (settings.crop_width - f64::from(settings.width)).abs() > 0.000_001
    || settings.crop_x.abs() > 0.000_001
    || settings.crop_y.abs() > 0.000_001
    || (settings.image_width - f64::from(settings.width)).abs() > 0.000_001
    || settings.image_x.abs() > 0.000_001
    || settings.image_y.abs() > 0.000_001
    || settings.recenter_inset_color.is_some()
    || settings.source_crop.x.abs() > 0.000_001
    || settings.source_crop.y.abs() > 0.000_001
    || (settings.source_crop.width - 1.0).abs() > 0.000_001
    || (settings.source_crop.height - 1.0).abs() > 0.000_001
}

#[cfg(test)]
mod composition_tests;
