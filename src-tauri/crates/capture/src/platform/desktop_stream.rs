// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::mpsc::{SyncSender, TrySendError};
use std::sync::Arc;
use std::time::Instant;

use super::*;
use crate::capture_kit::{capture_exclusion, CaptureExclusion};
use crate::desktop_capture::{CapturePiece, CapturePlan, DesktopDisplay};
use crate::platform::desktop_compositor::Composition;

mod window;
mod worker;

pub(super) use window::{create_window, WindowFilterRefresh, WindowStreamRequest};
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
  streams: Vec<WatchedStream>,
  worker: DesktopCompositionWorker,
}

impl DesktopStreams {
  pub async fn start(&self) -> Result<(), String> {
    for (started, watched) in self.streams.iter().enumerate() {
      if let Err(error) = watched.stream.start().await {
        for watched in &self.streams[..started] {
          watched.stream.stop_with_ch(|_| {});
        }
        return Err(error.to_string());
      }
    }
    Ok(())
  }

  pub fn into_parts(self) -> (Vec<WatchedStream>, DesktopKeepalive) {
    (
      self.streams,
      DesktopKeepalive {
        _outputs: self.outputs,
        refresh: None,
        worker: self.worker,
      },
    )
  }
}

pub(super) struct DesktopKeepalive {
  _outputs: Vec<arc::R<DesktopOutput>>,
  /// Keeps a window's displays leaving out apps opened since it started.
  refresh: Option<WindowFilterRefresh>,
  worker: DesktopCompositionWorker,
}

impl DesktopKeepalive {
  pub fn follow_new_apps(&mut self, refresh: WindowFilterRefresh) {
    self.refresh = Some(refresh);
  }

  pub fn stop(&mut self) {
    if let Some(refresh) = self.refresh.as_mut() {
      refresh.stop();
    }
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
  pub watch: &'a StreamWatch,
}

pub(super) fn create(request: DesktopStreamRequest<'_>) -> Result<DesktopStreams, String> {
  let worker = DesktopCompositionWorker::spawn(
    Composition::Region(request.plan.clone()),
    request.commands.clone(),
    Arc::clone(&request.stats),
  )?;
  let excluded = capture_exclusion(request.content, request.include_own_windows);
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
    let (filter, cfg) = piece_stream(
      display,
      display_geometry.scale,
      piece,
      &excluded,
      request.fps,
      captures_audio,
      request.show_cursor,
    );
    let mut recipe = StreamRecipe::new(&filter, cfg, request.queue).output(
      RecipeOutput::Desktop(output.clone()),
      sc::OutputType::Screen,
    );
    if captures_audio {
      recipe = recipe.output(
        RecipeOutput::Screen(request.audio_output.expect("checked above").clone()),
        sc::OutputType::Audio,
      );
    }
    outputs.push(output);
    streams.push(recipe.build(request.watch)?);
  }
  Ok(DesktopStreams {
    outputs,
    streams,
    worker,
  })
}

/// The filter and configuration of one piece's stream.
fn piece_stream(
  display: &sc::Display,
  scale: f64,
  piece: &CapturePiece,
  excluded: &CaptureExclusion,
  fps: u32,
  captures_audio: bool,
  show_cursor: bool,
) -> (arc::R<sc::ContentFilter>, arc::R<sc::StreamCfg>) {
  let filter = excluded.filter(display);
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
  (filter, cfg)
}

#[cfg(test)]
mod tests;
