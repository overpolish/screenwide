// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
  ffi::CString,
  path::PathBuf,
  sync::atomic::{AtomicU64, Ordering},
};

use super::super::*;

mod beside;
use beside::Beside;

mod ffi;
use ffi::{
  compose_frame, open_stream, progress, screenwide_video_export, should_cancel, wait_frame,
  ExportContext, GPU_PROGRESS_PERCENT,
};

mod frames;
use frames::FrameComposer;

mod frame_source;
mod mux;

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

/// The x264 run that writes `request`'s video beside its audio.
fn export_plan(request: &CursorExportRequest<'_>) -> mux::X264Plan {
  mux::X264Plan {
    output: mux::X264Output::Export {
      audio: mux::Audio::of(request),
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
  }
}

/// What a run of the loop leaves: how it ended, the loop's own x264 run
/// where FFmpeg wrote it, and the formats drawn beside it.
type Rendered = (
  ExportRunResult,
  Option<media_preview::StreamedExport>,
  Vec<Beside>,
);

/// The loop's own picture where every format is drawn into buffers of its
/// own: too small to cost anything, and written nowhere.
const UNUSED_LOOP_SIZE: u32 = 16;

/// Draws the frames `ranges` lay out, every frame of the export where it
/// keeps the whole source, and encodes them: streamed to x264 as `x264`
/// plans, answering the run to finish, or, where FFmpeg cannot be started,
/// by AVFoundation into `path`. With `beside`, every format, `request`'s
/// first, is drawn from the same decoded frames into buffers of its own and
/// written to an x264 run of its own on a thread of its own, so no encoder
/// holds the loop back; only x264 writes them. `annotations_within_ms` is
/// the stretch of the recording a preview draws, where it draws only that.
fn render_gpu_video<'r>(
  request: &mut CursorExportRequest<'r>,
  beside: &[CursorExportRequest<'r>],
  (ranges, annotations_within_ms): (&[screenwide_timeline::TimelineRange], Option<[u64; 2]>),
  path: &Path,
  x264: mux::X264Plan,
) -> Result<Rendered, String> {
  screenwide_output::validate_output_settings(request.width, request.height, request.output)?;
  for format in beside {
    screenwide_output::validate_output_settings(format.width, format.height, format.output)?;
  }
  let screen = c_path(request.screen)?;
  let camera = request.camera.map(|(path, _)| c_path(path)).transpose()?;
  let output = c_path(path)?;
  let bitrate = x264.bitrate;
  let drawn = std::iter::once(&*request).chain(beside).collect::<Vec<_>>();
  let separate = !beside.is_empty();
  let composer = FrameComposer::new(&drawn, !separate, annotations_within_ms)?;
  let (beside, x264, null, size) = if separate {
    (
      drawn
        .iter()
        .map(|format| Beside::new(format))
        .collect::<Result<Vec<_>, _>>()?,
      None,
      Some(
        std::fs::OpenOptions::new()
          .write(true)
          .open("/dev/null")
          .map_err(|error| error.to_string())?,
      ),
      (UNUSED_LOOP_SIZE, UNUSED_LOOP_SIZE),
    )
  } else {
    (
      Vec::new(),
      Some(x264),
      None,
      (request.output.width, request.output.height),
    )
  };
  let mut error = vec![0_i8; 2_048];
  let mut context = ExportContext {
    cancelled: request.cancelled,
    duration_ms: request.duration_ms,
    on_progress: request.on_progress,
    progress_pace: Default::default(),
    composer,
    beside,
    error: None,
    x264,
    stream: None,
    null,
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
      size.0,
      size.1,
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
  let beside = std::mem::take(&mut context.beside);
  // A format that failed stops the loop as a cancel would, and says why.
  let failure = context.error.take();
  match (result, failure) {
    (1, None) => Ok((ExportRunResult::Completed, stream, beside)),
    (-1, None) => Ok((ExportRunResult::Cancelled, stream, beside)),
    (_, failure) => {
      drop(beside);
      let message = unsafe { std::ffi::CStr::from_ptr(error.as_ptr()) }
        .to_string_lossy()
        .into_owned();
      // A frame the GPU failed on says so; otherwise an encoder that quit
      // says why, before the loop's own word for the frame it stopped on.
      let encoder = stream.and_then(media_preview::StreamedExport::abandon);
      Err(failure.or(encoder).unwrap_or(if message.is_empty() {
        "The video export failed".to_owned()
      } else {
        message
      }))
    }
  }
}

/// Lets each of `beside` finish its file. One that fails or is cancelled
/// takes every file written back, so an export leaves either every format
/// or none.
fn finish_beside(beside: Vec<Beside>, cancelled: &AtomicBool) -> Result<ExportRunResult, String> {
  let mut finished = Vec::new();
  for format in beside {
    let destination = format.destination().to_path_buf();
    let result = format.finish(cancelled);
    if matches!(result, Ok(ExportRunResult::Completed)) {
      finished.push(destination);
      continue;
    }
    for path in &finished {
      let _ = std::fs::remove_file(path);
    }
    return result;
  }
  Ok(ExportRunResult::Completed)
}

fn export_gpu<'r>(
  mut request: CursorExportRequest<'r>,
  beside: &[CursorExportRequest<'r>],
) -> Result<ExportRunResult, String> {
  let duration_ms = request
    .timeline
    .map_or(request.duration_ms, |timeline| timeline.duration_ms());
  let result = (|| {
    let video = gpu_video_path();
    let x264 = export_plan(&request);
    let ranges = request
      .timeline
      .map_or_else(Vec::new, |timeline| timeline.ranges().to_vec());
    let separate = !beside.is_empty();
    let (video_result, stream, beside) =
      render_gpu_video(&mut request, beside, (&ranges, None), &video, x264)?;
    if separate {
      // Every format went to a run of its own, which drops abandoned where
      // the loop did not finish.
      let _ = std::fs::remove_file(&video);
      let mut result = video_result;
      if matches!(result, ExportRunResult::Completed) {
        result = finish_beside(beside, request.cancelled)?;
      }
      if matches!(result, ExportRunResult::Completed) {
        (request.on_progress)(duration_ms);
      }
      return Ok(result);
    }
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

/// Exports `requests`, the first's progress and cancel standing for all.
pub(in super::super) fn export_formats(
  requests: Vec<CursorExportRequest<'_>>,
) -> Result<ExportRunResult, String> {
  let outputs = requests
    .iter()
    .map(export_output::scaled)
    .collect::<Vec<_>>();
  let mut scaled = requests
    .into_iter()
    .zip(&outputs)
    .map(|(request, output)| CursorExportRequest {
      output,
      on_progress: request.on_progress,
      ..request
    })
    .collect::<Vec<_>>();
  if scaled.is_empty() {
    return Err("There is no format to export".to_owned());
  }
  let lead = scaled.remove(0);
  export_gpu(lead, &scaled)
}
