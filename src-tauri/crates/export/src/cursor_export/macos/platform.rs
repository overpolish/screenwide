// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
  ffi::CString,
  path::PathBuf,
  sync::atomic::{AtomicU64, Ordering},
};

use super::super::*;

mod ffi;
use ffi::{
  compose_frame, open_stream, progress, screenwide_video_export, should_cancel, wait_frame,
  ExportContext, GPU_PROGRESS_PERCENT,
};

mod frames;
use frames::FrameComposer;

mod frame_camera;
mod frame_captions;
mod frame_grid;
mod frame_placement;
mod frame_source;
mod mux;

mod estimate;
pub(in super::super) use estimate::estimate_video_bytes;

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

/// Draws the frames `ranges` lay out, every frame of the export where it
/// keeps the whole source, and encodes them: streamed to x264 as `x264`
/// plans, answering the run to finish, or, where FFmpeg cannot be started,
/// by AVFoundation into `path`.
fn render_gpu_video(
  request: &mut CursorExportRequest<'_>,
  timeline: Option<&macos::native::CursorTimeline>,
  keyboard_timeline: Option<&macos::native::KeyboardTimeline>,
  ranges: &[screenwide_timeline::TimelineRange],
  path: &Path,
  x264: mux::X264Plan,
) -> Result<(ExportRunResult, Option<media_preview::StreamedExport>), String> {
  screenwide_output::validate_output_settings(request.width, request.height, request.output)?;
  let screen = c_path(request.screen)?;
  let camera = request.camera.map(|(path, _)| c_path(path)).transpose()?;
  let output = c_path(path)?;
  let bitrate = x264.bitrate;
  let composer = FrameComposer::new(request, timeline, keyboard_timeline)?;
  let mut error = vec![0_i8; 2_048];
  let mut context = ExportContext {
    cancelled: request.cancelled,
    duration_ms: request.duration_ms,
    on_progress: request.on_progress,
    progress_pace: Default::default(),
    composer,
    error: None,
    x264: Some(x264),
    stream: None,
  };
  let result = unsafe {
    screenwide_video_export(
      screen.as_ptr(),
      camera
        .as_ref()
        .map_or(std::ptr::null(), |path| path.as_ptr()),
      output.as_ptr(),
      ranges.as_ptr(),
      ranges.len() as u32,
      request.output.width,
      request.output.height,
      bitrate,
      (&mut context as *mut ExportContext<'_>).cast(),
      should_cancel,
      progress,
      compose_frame,
      wait_frame,
      open_stream,
      error.as_mut_ptr(),
      error.len(),
    )
  };
  let stream = context.stream.take();
  match result {
    1 => Ok((ExportRunResult::Completed, stream)),
    -1 => Ok((ExportRunResult::Cancelled, stream)),
    _ => {
      let message = unsafe { std::ffi::CStr::from_ptr(error.as_ptr()) }
        .to_string_lossy()
        .into_owned();
      // A frame the GPU failed on says so; otherwise an encoder that quit
      // says why, before the loop's own word for the frame it stopped on.
      let encoder = stream.and_then(media_preview::StreamedExport::abandon);
      Err(context.error.or(encoder).unwrap_or(if message.is_empty() {
        "The video export failed".to_owned()
      } else {
        message
      }))
    }
  }
}

fn export_gpu(mut request: CursorExportRequest<'_>) -> Result<ExportRunResult, String> {
  let timeline = macos::native::evaluate(&request)?;
  let keyboard_timeline = macos::native::evaluate_keyboard(&request)?;
  let duration_ms = request
    .timeline
    .map_or(request.duration_ms, |timeline| timeline.duration_ms());
  let result = (|| {
    let video = gpu_video_path();
    let x264 = mux::X264Plan {
      output: mux::X264Output::Export {
        audio: mux::Audio::of(&request),
        temporary: media_preview::remux_temp_path(request.destination),
      },
      width: request.output.width,
      height: request.output.height,
      compression: request.video.compression,
      bitrate: super::super::video_bitrate(
        request.output.width,
        request.output.height,
        request.video.compression,
      ),
    };
    let ranges = request
      .timeline
      .map_or_else(Vec::new, |timeline| timeline.ranges().to_vec());
    let (video_result, stream) = render_gpu_video(
      &mut request,
      timeline.as_ref(),
      keyboard_timeline.as_ref(),
      &ranges,
      &video,
      x264,
    )?;
    if let Some(stream) = stream {
      // A run left unfinished stops FFmpeg and removes its file as it drops.
      if !matches!(video_result, ExportRunResult::Completed) {
        return Ok(video_result);
      }
      let result = stream.finish(request.destination, request.cancelled)?;
      if matches!(result, ExportRunResult::Completed) {
        (request.on_progress)(duration_ms);
      }
      return Ok(result);
    }
    if !matches!(video_result, ExportRunResult::Completed) {
      let _ = std::fs::remove_file(&video);
      return Ok(video_result);
    }
    let temporary = media_preview::remux_temp_path(request.destination);
    let args = mux::Audio::of(&request).remux_args(&video, &temporary);
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

pub(in super::super) fn export(
  request: CursorExportRequest<'_>,
) -> Result<ExportRunResult, String> {
  let output = export_output::scaled(&request);
  export_gpu(CursorExportRequest {
    output: &output,
    on_progress: &mut *request.on_progress,
    ..request
  })
}

#[cfg(test)]
mod annotation_tests;
#[cfg(test)]
mod camera_background_video_tests;
#[cfg(test)]
mod camera_effect_video_tests;
#[cfg(test)]
mod caption_video_tests;
#[cfg(test)]
mod counter_tests;
#[cfg(test)]
mod cursor_order_tests;
#[cfg(test)]
mod cursor_order_video_tests;
#[cfg(test)]
mod draw_tests;
#[cfg(test)]
mod highlight_tests;
#[cfg(test)]
mod image_tests;
#[cfg(test)]
mod keyboard_tests;
#[cfg(test)]
mod magnify_tests;
#[cfg(test)]
mod magnify_video_tests;
#[cfg(test)]
mod pinned_annotation_tests;
#[cfg(test)]
mod recenter_tests;
#[cfg(test)]
mod redact_attack_tests;
#[cfg(test)]
mod redact_blur_tests;
#[cfg(test)]
mod redact_classic_tests;
#[cfg(test)]
mod redact_tests;
#[cfg(test)]
mod redact_video_tests;
#[cfg(test)]
mod reordered_video_tests;
#[cfg(test)]
mod scene_custom_video_tests;
#[cfg(test)]
mod scene_layout_video_tests;
#[cfg(test)]
mod scene_video_tests;
#[cfg(test)]
mod shape_tests;
#[cfg(test)]
mod spotlight_tests;
#[cfg(test)]
mod spotlight_video_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod text_tests;

#[cfg(test)]
mod timed_annotation_tests;

mod export_output;
