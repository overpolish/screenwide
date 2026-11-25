// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Windows final-video composition. Media Foundation decodes on a plain
//! Direct3D 11 device on wgpu's adapter, the preview shader draws offscreen
//! in wgpu, and the hardware H.264 encoder takes the copy on the same
//! Direct3D 11 device; see `export_canvas`.

mod preview;
mod render;
pub(in super::super) use preview::preview_frame;
use render::render_video;

use std::{
  ffi::OsString,
  path::{Path, PathBuf},
  sync::atomic::{AtomicU64, Ordering},
};

use ::windows::Win32::Graphics::Direct3D11::{ID3D11Device, ID3D11Texture2D};
use screenwide_h264::{FileEncoder, RateControl, Settings};

use super::super::*;
use super::export_canvas::WindowsExportCanvas;
use crate::cursor_export::frame_draw::FrameDraw;
use crate::decode::windows::gpu_decoder::GpuVideoReader;

const GPU_PROGRESS_PERCENT: u64 = 95;
static EXPORT_ATTEMPTS: AtomicU64 = AtomicU64::new(0);

/// The export's video file at `path`, encoded on `device`.
fn video_file(
  path: &Path,
  device: &ID3D11Device,
  ((width, height), fps): ((u32, u32), u32),
  bitrate: u64,
) -> Result<FileEncoder, String> {
  let fps = fps.max(1);
  let settings = Settings {
    width,
    height,
    fps,
    bitrate: u32::try_from(bitrate).unwrap_or(u32::MAX),
    // Given only an average, each vendor's encoder picks its own rate
    // control and some let it float with the picture. Held constant on
    // every GPU, a look makes a file of the same size whichever GPU encodes
    // it.
    rate_control: RateControl::Constant,
    keyframe_interval: fps,
    low_latency: false,
    purpose: "Export",
  };
  FileEncoder::create(device, settings, None, path, None)
}

fn gpu_video_path() -> PathBuf {
  let attempt = EXPORT_ATTEMPTS.fetch_add(1, Ordering::Relaxed);
  std::env::temp_dir().join(format!(
    "{}windows-gpu-video-{}-{attempt}.mp4",
    media_preview::PREVIEW_PREFIX,
    std::process::id()
  ))
}

fn mux_args(request: &CursorExportRequest<'_>, video: &Path, temporary: &Path) -> Vec<OsString> {
  let mut args = ["-hide_banner", "-loglevel", "error", "-nostdin", "-y", "-i"]
    .map(OsString::from)
    .to_vec();
  args.push(video.into());
  args.extend([
    OsString::from("-i"),
    request.audio_source.unwrap_or(request.screen).into(),
  ]);
  args.extend(
    [
      "-progress",
      "pipe:1",
      "-nostats",
      "-map",
      "0:v:0",
      "-c:v",
      "copy",
    ]
    .map(OsString::from),
  );
  args.extend(request.timeline.map_or_else(
    || {
      request
        .selection
        .audio_args_from(request.audio_layout, 1)
        .into_iter()
        .map(OsString::from)
        .collect()
    },
    |timeline| {
      media_preview::timeline_audio_mapping_args(
        timeline,
        1,
        request.selection,
        request.audio_layout,
      )
    },
  ));
  args.extend(
    [
      "-tag:v",
      "avc1",
      "-movflags",
      "+faststart",
      "-map_metadata",
      "-1",
      "-f",
      "mp4",
    ]
    .map(OsString::from),
  );
  args.push(temporary.into());
  args
}

/// Exports `requests` from one decode, the first's progress and cancel
/// standing for all: each format's video is drawn and encoded in the same
/// pass, then muxed beside its audio. A format that fails or is cancelled
/// takes every file written back, so an export leaves every format or none.
pub(in super::super) fn export_formats(
  requests: Vec<CursorExportRequest<'_>>,
) -> Result<ExportRunResult, String> {
  let outputs = requests
    .iter()
    .map(export_output::scaled)
    .collect::<Vec<_>>();
  let mut beside = requests
    .into_iter()
    .zip(&outputs)
    .map(|(request, output)| CursorExportRequest {
      output,
      on_progress: request.on_progress,
      ..request
    })
    .collect::<Vec<_>>();
  if beside.is_empty() {
    return Err("There is no format to export".to_owned());
  }
  let mut request = beside.remove(0);
  let videos = (0..=beside.len())
    .map(|_| gpu_video_path())
    .collect::<Vec<_>>();
  let result = (|| {
    let (rendered, _) = render_video(&mut request, &beside, &videos, None)?;
    if !matches!(rendered, ExportRunResult::Completed) {
      return Ok(rendered);
    }
    let temporary = media_preview::remux_temp_path(request.destination);
    let args = mux_args(&request, &videos[0], &temporary);
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
    let mut result = media_preview::run_export(
      args,
      &temporary,
      request.destination,
      request.cancelled,
      &mut final_progress,
    )?;
    let mut written = vec![request.destination.to_path_buf()];
    for (format, video) in beside.iter().zip(&videos[1..]) {
      if !matches!(result, ExportRunResult::Completed) {
        break;
      }
      let temporary = media_preview::remux_temp_path(format.destination);
      let muxed = media_preview::run_export(
        mux_args(format, video, &temporary),
        &temporary,
        format.destination,
        request.cancelled,
        &mut |_| {},
      );
      match muxed {
        Ok(muxed) => result = muxed,
        Err(error) => {
          for path in &written {
            let _ = std::fs::remove_file(path);
          }
          let name = format
            .destination
            .file_name()
            .map_or_else(String::new, |name| name.to_string_lossy().into_owned());
          return Err(format!("{name}: {error}"));
        }
      }
      written.push(format.destination.to_path_buf());
    }
    if !matches!(result, ExportRunResult::Completed) {
      for path in &written {
        let _ = std::fs::remove_file(path);
      }
    }
    Ok(result)
  })();
  for video in &videos {
    let _ = std::fs::remove_file(video);
  }
  result
}
