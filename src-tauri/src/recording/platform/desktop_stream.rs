// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::mpsc::{SyncSender, TrySendError};
use std::sync::Arc;
use std::time::Instant;

use super::*;
use crate::capture_kit::windows_to_exclude;
use crate::desktop_capture::{CapturePiece, CapturePlan, DesktopDisplay};

mod worker;

use worker::{DesktopCompositionMessage, DesktopCompositionWorker, DesktopSourceFrame};

#[repr(C)]
struct DesktopOutputInner {
  frames: SyncSender<DesktopCompositionMessage>,
  source_index: usize,
  stats: Arc<CaptureStats>,
}

impl DesktopOutputInner {
  fn handle_video(&mut self, sample: &cm::SampleBuf) {
    if frame_status(sample) != Some(sc::FrameStatus::Complete) {
      return;
    }
    let (Some(image), Some(source_ns)) = (sample.image_buf(), time_to_ns(sample.pts())) else {
      return;
    };
    let frame = DesktopSourceFrame {
      buf: image.retained(),
      source_index: self.source_index,
      source_ns,
      wall: Instant::now(),
    };
    if let Err(TrySendError::Full(_)) = self
      .frames
      .try_send(DesktopCompositionMessage::Frame(frame))
    {
      self.stats.dropped.fetch_add(1, Ordering::Relaxed);
    }
  }
}

define_obj_type!(
  pub(super) DesktopOutput + OutputImpl,
  DesktopOutputInner,
  DESKTOP_OUTPUT_CLS
);

impl Output for DesktopOutput {}

#[objc::add_methods]
impl OutputImpl for DesktopOutput {
  extern "C" fn impl_stream_did_output_sample_buf(
    &mut self,
    _command: Option<&objc::Sel>,
    _stream: &sc::Stream,
    sample_buffer: &mut cm::SampleBuf,
    kind: sc::OutputType,
  ) {
    if kind == sc::OutputType::Screen {
      self.inner_mut().handle_video(sample_buffer);
    }
  }
}

pub(super) struct DesktopStreams {
  outputs: Vec<arc::R<DesktopOutput>>,
  streams: Vec<arc::R<sc::Stream>>,
  worker: DesktopCompositionWorker,
}

impl DesktopStreams {
  pub async fn start(&self) -> Result<(), String> {
    for (started, stream) in self.streams.iter().enumerate() {
      if let Err(error) = stream.start().await {
        for stream in &self.streams[..started] {
          stream.stop_with_ch(|_| {});
        }
        return Err(error.to_string());
      }
    }
    Ok(())
  }

  pub fn into_parts(self) -> (Vec<arc::R<sc::Stream>>, DesktopKeepalive) {
    (
      self.streams,
      DesktopKeepalive {
        _outputs: self.outputs,
        worker: self.worker,
      },
    )
  }
}

pub(super) struct DesktopKeepalive {
  _outputs: Vec<arc::R<DesktopOutput>>,
  worker: DesktopCompositionWorker,
}

impl DesktopKeepalive {
  pub fn stop(&mut self) {
    self.worker.stop();
  }
}

pub(super) struct DesktopStreamRequest<'a> {
  pub audio_output: Option<&'a arc::R<ScreenOutput>>,
  pub commands: SyncSender<Command>,
  pub content: &'a sc::ShareableContent,
  pub displays: &'a [DesktopDisplay],
  pub fps: u32,
  pub include_own_windows: bool,
  pub plan: &'a CapturePlan,
  pub queue: &'a dispatch::Queue,
  pub stats: Arc<CaptureStats>,
  pub show_cursor: bool,
}

pub(super) fn create(request: DesktopStreamRequest<'_>) -> Result<DesktopStreams, String> {
  let worker = DesktopCompositionWorker::spawn(
    request.plan.clone(),
    request.commands.clone(),
    Arc::clone(&request.stats),
  )?;
  let excluded = windows_to_exclude(request.content, request.include_own_windows);
  let available_displays = request.content.displays();
  let anchor_display = request.plan.pieces.first().map(|piece| piece.display_id);
  let mut outputs = Vec::with_capacity(request.plan.pieces.len());
  let mut streams = Vec::with_capacity(request.plan.pieces.len());
  for (source_index, piece) in request.plan.pieces.iter().enumerate() {
    let display_geometry = request
      .displays
      .iter()
      .find(|display| display.id == piece.display_id)
      .ok_or_else(|| "A desktop capture display is no longer available".to_owned())?;
    let display = available_displays
      .iter()
      .find(|display| display.display_id().0 == piece.display_id)
      .ok_or_else(|| "A ScreenCaptureKit display is no longer available".to_owned())?;
    let output = DesktopOutput::with(DesktopOutputInner {
      frames: worker.sender.as_ref().expect("worker is active").clone(),
      source_index,
      stats: Arc::clone(&request.stats),
    });
    let captures_audio = request.audio_output.is_some() && Some(piece.display_id) == anchor_display;
    let stream = stream_for_piece(
      display,
      display_geometry.scale,
      piece,
      &excluded,
      request.fps,
      captures_audio,
      request.show_cursor,
    );
    stream
      .add_stream_output(output.as_ref(), sc::OutputType::Screen, Some(request.queue))
      .map_err(|error| error.to_string())?;
    if captures_audio {
      stream
        .add_stream_output(
          request.audio_output.expect("checked above").as_ref(),
          sc::OutputType::Audio,
          Some(request.queue),
        )
        .map_err(|error| error.to_string())?;
    }
    outputs.push(output);
    streams.push(stream);
  }
  Ok(DesktopStreams {
    outputs,
    streams,
    worker,
  })
}

fn stream_for_piece(
  display: &sc::Display,
  scale: f64,
  piece: &CapturePiece,
  excluded: &ns::Array<sc::Window>,
  fps: u32,
  captures_audio: bool,
  show_cursor: bool,
) -> arc::R<sc::Stream> {
  let filter = sc::ContentFilter::with_display_excluding_windows(display, excluded);
  let mut cfg = sc::StreamCfg::new();
  cfg.set_width(piece.source_pixels.width as usize);
  cfg.set_height(piece.source_pixels.height as usize);
  cfg.set_scales_to_fit(false);
  cfg.set_src_rect(cg::Rect::new(
    f64::from(piece.source_pixels.x) / scale,
    f64::from(piece.source_pixels.y) / scale,
    f64::from(piece.source_pixels.width) / scale,
    f64::from(piece.source_pixels.height) / scale,
  ));
  cfg.set_pixel_format(cv::PixelFormat::_32_BGRA);
  cfg.set_minimum_frame_interval(cm::Time::new(1, fps as cm::TimeScale));
  cfg.set_queue_depth(STREAM_QUEUE_DEPTH);
  // ScreenCaptureKit only draws the pointer into the display currently
  // containing it. Keeping this setting identical on every source preserves
  // the user's baked-cursor choice while the shared cursor sidecar remains a
  // single global-desktop track.
  cfg.set_shows_cursor(show_cursor);
  cfg.set_captures_audio(captures_audio);
  if captures_audio {
    configure_system_audio(&mut cfg);
  }
  cfg.set_color_space_name(cg::color_space::names::srgb());
  sc::Stream::new(&filter, &cfg)
}

#[cfg(test)]
mod tests;
