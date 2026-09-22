// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
  ffi::CString,
  path::PathBuf,
  sync::atomic::{AtomicU64, Ordering},
};

use super::*;
use crate::editor::cursor_effects::{NativeGpuArtwork, NativeGpuCursor};

#[path = "platform_macos/ffi.rs"]
mod ffi;
use ffi::{
  gpu_progress, gpu_should_cancel, screenwide_gpu_composite_cursor, GpuCallbacks, GpuCameraOverlay,
  GPU_PROGRESS_PERCENT,
};

#[path = "platform_macos/mux.rs"]
mod mux;

#[path = "timed_annotations.rs"]
mod timed_annotations;

static GPU_EXPORT_ATTEMPTS: AtomicU64 = AtomicU64::new(0);

fn c_path(path: &Path) -> Result<CString, String> {
  CString::new(path.as_os_str().as_encoded_bytes())
    .map_err(|_| format!("{} contains a null byte", path.display()))
}

fn gpu_video_path() -> PathBuf {
  let attempt = GPU_EXPORT_ATTEMPTS.fetch_add(1, Ordering::Relaxed);
  std::env::temp_dir().join(format!(
    "{}gpu-video-{}-{attempt}.mp4",
    media_preview::PREVIEW_PREFIX,
    std::process::id()
  ))
}

/// Flattens the evaluated timeline into the frame array the compositor indexes.
fn cursor_frames(timeline: &native_macos::CursorTimeline) -> Vec<NativeGpuCursor> {
  timeline
    .frames
    .iter()
    .map(|cursor| NativeGpuCursor::from(*cursor))
    .collect()
}

fn cursor_artworks(timeline: &native_macos::CursorTimeline) -> Vec<NativeGpuArtwork> {
  timeline
    .artworks
    .iter()
    .map(NativeGpuArtwork::from)
    .collect()
}

fn render_gpu_video(
  request: &mut CursorExportRequest<'_>,
  timeline: Option<&native_macos::CursorTimeline>,
  keyboard_timeline: Option<&native_macos::KeyboardTimeline>,
  path: &Path,
) -> Result<ExportRunResult, String> {
  crate::screenshots::validate_output_settings(request.width, request.height, request.output)?;
  let screen = c_path(request.screen)?;
  let cursors = timeline.map(cursor_frames).unwrap_or_default();
  let artworks = timeline.map(cursor_artworks).unwrap_or_default();
  let keyboards = keyboard_timeline
    .map(|timeline| timeline.frames.as_slice())
    .unwrap_or_default();
  let (annotations, annotation_data) = timed_annotations::for_request(request);
  let annotation_data = annotation_data.view();
  let camera = request.camera.map(|(path, _)| c_path(path)).transpose()?;
  let camera_overlay = request
    .camera
    .map(|(_, options)| media_preview::bake_geometry(options))
    .transpose()?
    .map(|geometry| {
      let scale_x = f64::from(request.output.width) / f64::from(geometry.output_width.max(1));
      let scale_y = f64::from(request.output.height) / f64::from(geometry.output_height.max(1));
      let scaled = |value: u32, scale: f64| (f64::from(value) * scale).round() as u32;
      let scaled_position = |value: i32, scale: f64| (f64::from(value) * scale).round() as i32;
      GpuCameraOverlay {
        crop_x: geometry.crop_x,
        crop_y: geometry.crop_y,
        crop_width: geometry.crop_width,
        crop_height: geometry.crop_height,
        frame_x: scaled_position(geometry.frame_x, scale_x),
        frame_y: scaled_position(geometry.frame_y, scale_y),
        frame_width: scaled(geometry.frame_width, scale_x),
        frame_height: scaled(geometry.frame_height, scale_y),
        radius: scaled(geometry.radius, scale_x.min(scale_y)),
        drop_shadow: u32::from(
          request
            .camera
            .is_some_and(|(_, options)| options.camera_drop_shadow),
        ),
        camera_on_top: u32::from(request.camera_on_top),
      }
    });
  let output = c_path(path)?;
  let timeline_ranges = request
    .timeline
    .map_or(&[][..], |timeline| timeline.ranges());
  let mut canvas =
    crate::screenshots::native_canvas(request.width, request.height, request.output, false)?;
  canvas.clip_cursor_at_video_edge = u32::from(request.cursor_effects.clip_at_video_edge);
  let mut error = vec![0_i8; 2_048];
  let mut callbacks = GpuCallbacks {
    cancelled: request.cancelled,
    duration_ms: request.duration_ms,
    on_progress: request.on_progress,
  };
  let result = unsafe {
    screenwide_gpu_composite_cursor(
      screen.as_ptr(),
      cursors.as_ptr(),
      cursors.len() as u32,
      artworks.as_ptr(),
      artworks.len() as u32,
      keyboards.as_ptr(),
      keyboards.len() as u32,
      annotations.as_ptr(),
      annotations.len() as u32,
      &annotation_data,
      timeline_ranges.as_ptr(),
      timeline_ranges.len() as u32,
      camera
        .as_ref()
        .map_or(std::ptr::null(), |path| path.as_ptr()),
      camera_overlay
        .as_ref()
        .map_or(std::ptr::null(), std::ptr::from_ref),
      &canvas,
      output.as_ptr(),
      request.width,
      request.height,
      request.output.width,
      request.output.height,
      super::video_bitrate(
        request.output.width,
        request.output.height,
        request.video.compression,
      ),
      (&mut callbacks as *mut GpuCallbacks<'_>).cast(),
      gpu_should_cancel,
      gpu_progress,
      error.as_mut_ptr(),
      error.len(),
    )
  };
  match result {
    1 => Ok(ExportRunResult::Completed),
    -1 => Ok(ExportRunResult::Cancelled),
    _ => {
      let message = unsafe { std::ffi::CStr::from_ptr(error.as_ptr()) }
        .to_string_lossy()
        .into_owned();
      Err(if message.is_empty() {
        "The Metal cursor compositor failed".to_owned()
      } else {
        message
      })
    }
  }
}

fn export_gpu(mut request: CursorExportRequest<'_>) -> Result<ExportRunResult, String> {
  let timeline = native_macos::evaluate(&request)?;
  let keyboard_timeline = native_macos::evaluate_keyboard(&request)?;
  let result = (|| {
    let video = gpu_video_path();
    let video_result = render_gpu_video(
      &mut request,
      timeline.as_ref(),
      keyboard_timeline.as_ref(),
      &video,
    )?;
    if !matches!(video_result, ExportRunResult::Completed) {
      let _ = std::fs::remove_file(&video);
      return Ok(video_result);
    }
    let temporary = media_preview::remux_temp_path(request.destination);
    let args = mux::args(&request, &video, &temporary);
    let duration_ms = request
      .timeline
      .map_or(request.duration_ms, |timeline| timeline.duration_ms());
    let on_progress = &mut request.on_progress;
    let mut final_progress = |processed_ms: u64| {
      on_progress(
        duration_ms.saturating_mul(GPU_PROGRESS_PERCENT) / 100
          + processed_ms.saturating_mul(100 - GPU_PROGRESS_PERCENT) / 100,
      );
    };
    let result = media_preview::run_export(
      args,
      &temporary,
      request.destination,
      request.cancelled,
      &mut final_progress,
    );
    let _ = std::fs::remove_file(&video);
    result
  })();
  if request.cancelled.load(Ordering::Acquire) {
    return Ok(ExportRunResult::Cancelled);
  }
  result
}

pub(super) fn export(request: CursorExportRequest<'_>) -> Result<ExportRunResult, String> {
  let output = export_output::scaled(&request);
  export_gpu(CursorExportRequest {
    output: &output,
    on_progress: &mut *request.on_progress,
    ..request
  })
}

#[cfg(test)]
#[path = "platform_macos_annotation_tests.rs"]
mod annotation_tests;
#[cfg(test)]
#[path = "platform_macos_counter_tests.rs"]
mod counter_tests;
#[cfg(test)]
#[path = "platform_macos_keyboard_tests.rs"]
mod keyboard_tests;
#[cfg(test)]
#[path = "platform_macos_recenter_tests.rs"]
mod recenter_tests;
#[cfg(test)]
#[path = "platform_macos_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "platform_macos_timed_annotation_tests.rs"]
mod timed_annotation_tests;

#[path = "export_output.rs"]
mod export_output;
