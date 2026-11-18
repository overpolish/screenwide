// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A recording scene played on a window's surface: composed into a canvas
//! of the picture's size on screen, then drawn onto the surface, once every
//! display refresh while it moves and the preview is shown. The surface is
//! set to wait for the display, and each frame waits for the last to be done
//! on the GPU before it is drawn: Windows hands out a swap chain's frames
//! without waiting, and a frame drawn for every change the page sent queued
//! behind the display, the picture trailing the page further each frame.
//! Changes that arrive meanwhile are taken together. The scene's time is
//! counted in refreshes (`FrameClock`).

use std::time::{Duration, Instant};

use screenwide_gpu::surface::{Frame, Headroom, Surface};

use super::clock::FrameClock;
use super::content::canvas_texture;
use super::edge_fill::EdgeFill;
use super::render::Moment;
use super::{PreviewScene, SceneRenderer};
use crate::worker::Renderer;

/// How long to wait before asking again for a frame the surface had none
/// for, as while the window is hidden.
const RETRY: Duration = Duration::from_millis(16);

/// The swap chain's room past the picture on Windows, where the frame is
/// drawn over all of it (`EdgeFill`).
const SWAP_CHAIN: Headroom = Headroom {
  step: 128,
  margin: 32,
  slack: 128,
};

pub(crate) struct SceneDriver {
  renderer: SceneRenderer,
  surface: Surface,
  gpu: std::sync::Arc<screenwide_gpu::Gpu>,
  fill: EdgeFill,
  canvas: Option<wgpu::Texture>,
  size: (u32, u32),
  visible: bool,
  clock: FrameClock,
  /// What the canvas holds.
  composed: Option<(Moment, (u32, u32))>,
  /// Whether the surface shows the canvas as it is, which is all a still
  /// needs.
  shown: bool,
  /// The last frame's work, which the next frame waits for.
  in_flight: Option<wgpu::SubmissionIndex>,
}

impl SceneDriver {
  pub(crate) fn new(renderer: SceneRenderer, mut surface: Surface) -> Result<Self, String> {
    let gpu = screenwide_gpu::shared(screenwide_gpu::Workload::Light)?;
    surface.wait_for_display(&gpu);
    Ok(Self {
      renderer,
      surface,
      fill: EdgeFill::new(&gpu.device),
      gpu,
      canvas: None,
      size: (0, 0),
      visible: false,
      clock: FrameClock::default(),
      composed: None,
      shown: false,
      in_flight: None,
    })
  }

  fn restart(&mut self) -> Option<Duration> {
    self.clock = FrameClock::default();
    self.composed = None;
    self.shown = false;
    Some(Duration::ZERO)
  }

  /// Composes `moment` into the canvas, unless it holds it already.
  fn compose(&mut self, moment: Moment) -> Result<(), String> {
    let size = self.size;
    if self.composed == Some((moment, size)) {
      return Ok(());
    }
    let canvas = match self.canvas.take() {
      Some(canvas) if (canvas.width(), canvas.height()) == size => canvas,
      _ => canvas_texture(&self.gpu, size),
    };
    let composed = self
      .renderer
      .draw(&canvas.create_view(&Default::default()), size, moment);
    self.canvas = Some(canvas);
    composed?;
    self.composed = Some((moment, size));
    Ok(())
  }

  /// Takes a frame from the surface once the last one is done, and shows
  /// the moment of the scene it stands for on it. `false` when the surface
  /// had no frame to give.
  fn present(&mut self) -> Result<bool, String> {
    if let Some(last) = self.in_flight.take() {
      self
        .gpu
        .device
        .poll(wgpu::PollType::Wait {
          submission_index: Some(last),
          timeout: None,
        })
        .map_err(|error| error.to_string())?;
    }
    let Frame::Ready(frame) = self.surface.acquire(&self.gpu)? else {
      return Ok(false);
    };
    let moment = self.renderer.moment(self.clock.tick(Instant::now()));
    self.compose(moment)?;
    let Some(canvas) = &self.canvas else {
      return Ok(false);
    };
    let mut encoder = self
      .gpu
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Screenwide settings preview frame"),
      });
    self
      .fill
      .draw(&self.gpu.device, &mut encoder, canvas, &frame.texture);
    let submission = self.gpu.queue.submit([encoder.finish()]);
    self.gpu.queue.present(frame);
    self.in_flight = Some(submission);
    Ok(true)
  }
}

impl Renderer for SceneDriver {
  fn scene(&mut self, scene: PreviewScene) -> Option<Duration> {
    if let Err(error) = self.renderer.set_scene(scene) {
      eprintln!("A settings preview could not take its new values: {error}");
    }
    self.restart()
  }

  fn resize(&mut self, size: (u32, u32)) -> Option<Duration> {
    self.size = size;
    self.surface.fit(&self.gpu, size, SWAP_CHAIN);
    self.shown = false;
    Some(Duration::ZERO)
  }

  /// A preview shown again plays from the start, as the page's own loops do.
  fn visible(&mut self, visible: bool) -> Option<Duration> {
    self.visible = visible;
    if visible {
      self.restart()
    } else {
      None
    }
  }

  /// A moving scene is drawn every refresh, a frame of the recording held
  /// for as many refreshes as it lasts; a still once.
  fn frame(&mut self) -> Option<Duration> {
    let moving = self.renderer.duration_ms().is_some();
    if !self.visible || self.size.0 == 0 || self.size.1 == 0 || (self.shown && !moving) {
      return None;
    }
    match self.present() {
      Ok(true) => {
        self.shown = true;
        moving.then_some(Duration::ZERO)
      }
      Ok(false) => Some(RETRY),
      Err(error) => {
        eprintln!("A settings preview could not draw: {error}");
        None
      }
    }
  }
}
