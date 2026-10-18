// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Windows final-video composition. Media Foundation decodes on a plain
//! Direct3D 11 device on wgpu's adapter, the preview shader draws offscreen
//! in wgpu, and the hardware H.264 transform consumes the copy on the same
//! Direct3D 11 device; see `export_canvas`.

mod encoder_config;
mod preview;
mod render;
pub(in super::super) use preview::preview_frame;
use render::render_video;

use std::{
  ffi::OsString,
  path::{Path, PathBuf},
  sync::atomic::{AtomicU64, Ordering},
};

use ::windows::{
  core::{Interface, PCWSTR},
  Win32::{
    Graphics::Direct3D11::{ID3D11Device, ID3D11Texture2D},
    Media::MediaFoundation::*,
  },
};

use super::super::*;
use super::export_canvas::WindowsExportCanvas;
use crate::cursor_export::frame_draw::FrameDraw;
use crate::decode::windows::gpu_decoder::GpuVideoReader;

const GPU_PROGRESS_PERCENT: u64 = 95;
static EXPORT_ATTEMPTS: AtomicU64 = AtomicU64::new(0);

fn win<T>(result: ::windows::core::Result<T>) -> Result<T, String> {
  result.map_err(|error| error.to_string())
}

struct VideoSink {
  _byte_stream: IMFByteStream,
  _device_manager: IMFDXGIDeviceManager,
  media_sink: IMFMediaSink,
  sink: Option<IMFSinkWriter>,
}

impl VideoSink {
  fn new(
    path: &Path,
    device: &ID3D11Device,
    ((width, height), fps): ((u32, u32), u32),
    bitrate: u64,
  ) -> Result<Self, String> {
    let mut reset_token = 0;
    let mut manager = None;
    win(unsafe { MFCreateDXGIDeviceManager(&mut reset_token, &mut manager) })?;
    let manager =
      manager.ok_or_else(|| "Media Foundation created no export D3D manager".to_owned())?;
    win(unsafe { manager.ResetDevice(device, reset_token) })?;

    let attributes = attributes(4)?;
    win(unsafe { attributes.SetUINT32(&MF_READWRITE_ENABLE_HARDWARE_TRANSFORMS, 1) })?;
    win(unsafe { attributes.SetUnknown(&MF_SINK_WRITER_D3D_MANAGER, &manager) })?;
    let bitrate = bitrate.min(u64::from(u32::MAX)) as u32;
    match encoder_config::constant_bitrate(bitrate) {
      Ok(store) => win(unsafe { attributes.SetUnknown(&MF_SINK_WRITER_ENCODER_CONFIG, &store) })?,
      Err(message) => eprintln!("windows export: constant bitrate not applied: {message}"),
    }
    let output = video_type(MFVideoFormat_H264, (width, height), fps)?;
    win(unsafe { output.SetUINT32(&MF_MT_AVG_BITRATE, bitrate) })?;
    win(unsafe { output.SetUINT32(&MF_MT_MPEG2_PROFILE, eAVEncH264VProfile_High.0 as u32) })?;
    win(unsafe { output.SetUINT32(&MF_MT_MAX_KEYFRAME_SPACING, fps) })?;

    let wide = path
      .to_str()
      .ok_or_else(|| "The export path is not valid UTF-8".to_owned())?
      .encode_utf16()
      .chain(Some(0))
      .collect::<Vec<_>>();
    let byte_stream = win(unsafe {
      MFCreateFile(
        MF_ACCESSMODE_WRITE,
        MF_OPENMODE_DELETE_IF_EXIST,
        MF_FILEFLAGS_NONE,
        PCWSTR(wide.as_ptr()),
      )
    })?;
    let media_sink = win(unsafe { MFCreateFMPEG4MediaSink(&byte_stream, &output, None) })?;
    let sink = win(unsafe { MFCreateSinkWriterFromMediaSink(&media_sink, &attributes) })?;
    let input = video_type(MFVideoFormat_ARGB32, (width, height), fps)?;
    win(unsafe { sink.SetInputMediaType(0, &input, None) })?;
    win(unsafe { sink.BeginWriting() })?;
    Ok(Self {
      _byte_stream: byte_stream,
      _device_manager: manager,
      media_sink,
      sink: Some(sink),
    })
  }

  fn write(
    &self,
    texture: &ID3D11Texture2D,
    pts_100ns: i64,
    duration_100ns: i64,
  ) -> Result<(), String> {
    let buffer = unsafe { MFCreateDXGISurfaceBuffer(&ID3D11Texture2D::IID, texture, 0, false) }
      .map_err(|error| error.to_string())?;
    let two_dimensional = buffer
      .cast::<IMF2DBuffer>()
      .map_err(|error| error.to_string())?;
    let length =
      unsafe { two_dimensional.GetContiguousLength() }.map_err(|error| error.to_string())?;
    win(unsafe { buffer.SetCurrentLength(length) })?;
    let sample = unsafe { MFCreateSample() }.map_err(|error| error.to_string())?;
    win(unsafe { sample.AddBuffer(&buffer) })?;
    win(unsafe { sample.SetSampleTime(pts_100ns.max(0)) })?;
    win(unsafe { sample.SetSampleDuration(duration_100ns.max(1)) })?;
    let sink = self
      .sink
      .as_ref()
      .ok_or_else(|| "The export encoder is already finalized".to_owned())?;
    win(unsafe { sink.WriteSample(0, &sample) })
  }

  fn finish(mut self) -> Result<(), String> {
    let sink = self
      .sink
      .take()
      .ok_or_else(|| "The export encoder is already finalized".to_owned())?;
    win(unsafe { sink.Finalize() })?;
    drop(sink);
    win(unsafe { self.media_sink.Shutdown() })
  }
}

impl Drop for VideoSink {
  fn drop(&mut self) {
    self.sink.take();
    let _ = unsafe { self.media_sink.Shutdown() };
  }
}

fn attributes(capacity: u32) -> Result<IMFAttributes, String> {
  let mut value = None;
  win(unsafe { MFCreateAttributes(&mut value, capacity) })?;
  value.ok_or_else(|| "Media Foundation created no export attributes".to_owned())
}

fn video_type(
  subtype: ::windows::core::GUID,
  (width, height): (u32, u32),
  fps: u32,
) -> Result<IMFMediaType, String> {
  let value = unsafe { MFCreateMediaType() }.map_err(|error| error.to_string())?;
  win(unsafe { value.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video) })?;
  win(unsafe { value.SetGUID(&MF_MT_SUBTYPE, &subtype) })?;
  win(unsafe {
    value.SetUINT64(
      &MF_MT_FRAME_SIZE,
      (u64::from(width) << 32) | u64::from(height),
    )
  })?;
  win(unsafe { value.SetUINT64(&MF_MT_FRAME_RATE, (u64::from(fps) << 32) | 1) })?;
  win(unsafe { value.SetUINT64(&MF_MT_PIXEL_ASPECT_RATIO, (1_u64 << 32) | 1) })?;
  win(unsafe { value.SetUINT32(&MF_MT_INTERLACE_MODE, MFVideoInterlace_Progressive.0 as u32) })?;
  Ok(value)
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

pub(in super::super) fn export(
  request: CursorExportRequest<'_>,
) -> Result<ExportRunResult, String> {
  let output = export_output::scaled(&request);
  let mut request = CursorExportRequest {
    output: &output,
    on_progress: &mut *request.on_progress,
    ..request
  };
  let video = gpu_video_path();
  let result = (|| {
    let (rendered, _) = render_video(&mut request, &video, None)?;
    if !matches!(rendered, ExportRunResult::Completed) {
      return Ok(rendered);
    }
    let temporary = media_preview::remux_temp_path(request.destination);
    let args = mux_args(&request, &video, &temporary);
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
    media_preview::run_export(
      args,
      &temporary,
      request.destination,
      request.cancelled,
      &mut final_progress,
    )
  })();
  let _ = std::fs::remove_file(&video);
  result
}

/// The bytes the video stream of `request`'s export takes: the constant
/// bitrate its encoder is held to, over the edited video, at the size
/// `render_video` draws it.
pub(in super::super) fn estimate_video_bytes(
  request: CursorExportRequest<'_>,
) -> Result<u64, String> {
  let (width, height) = screenwide_output::output_dimensions(&export_output::scaled(&request))?;
  let duration_ms = request
    .timeline
    .map_or(request.duration_ms, |timeline| timeline.duration_ms());
  Ok(
    super::super::video_bitrate(width, height, request.video.compression)
      .saturating_mul(duration_ms)
      / 8_000,
  )
}
