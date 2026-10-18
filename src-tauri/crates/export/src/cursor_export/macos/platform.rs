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

mod frame_source;
mod mux;

mod estimate;
pub(in super::super) use estimate::estimate_video_bytes;
mod preview;
pub(in super::super) use preview::preview_frame;

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
/// by AVFoundation into `path`. `annotations_within_ms` is the stretch of the
/// recording a preview draws, where it draws only that.
fn render_gpu_video(
  request: &mut CursorExportRequest<'_>,
  (ranges, annotations_within_ms): (&[screenwide_timeline::TimelineRange], Option<[u64; 2]>),
  path: &Path,
  x264: mux::X264Plan,
) -> Result<(ExportRunResult, Option<media_preview::StreamedExport>), String> {
  screenwide_output::validate_output_settings(request.width, request.height, request.output)?;
  let screen = c_path(request.screen)?;
  let camera = request.camera.map(|(path, _)| c_path(path)).transpose()?;
  let output = c_path(path)?;
  let bitrate = x264.bitrate;
  let composer = FrameComposer::new(request, annotations_within_ms)?;
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
    let (video_result, stream) = render_gpu_video(&mut request, (&ranges, None), &video, x264)?;
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
