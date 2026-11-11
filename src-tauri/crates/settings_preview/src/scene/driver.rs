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

use screenwide_gpu::surface::{Frame, Surface};

use super::clock::FrameClock;
use super::content::canvas_texture;
use super::edge_fill::EdgeFill;
use super::render::Moment;
use super::{PreviewScene, SceneRenderer};
use crate::worker::Renderer;

/// How long to wait before asking again for a frame the surface had none
/// for, as while the window is hidden.
const RETRY: Duration = Duration::from_millis(16);

/// The steps a swap chain on Windows grows and shrinks in, in pixels, and how
/// far past the picture it reaches at least.
#[cfg(target_os = "windows")]
const SWAP_CHAIN_STEP: u32 = 128;
#[cfg(target_os = "windows")]
const SWAP_CHAIN_MARGIN: u32 = 32;

/// The swap chain to show a picture `picture` in, given the one there is.
/// Windows reallocates a swap chain only once every frame queued on the
/// shared device has finished, and shows nothing of it until a frame is
/// presented into the new one. DirectComposition shows the swap chain
/// unscaled and clipped to the picture, so there it is kept larger than the
/// picture, in steps, the frame drawn over all of it (`EdgeFill`). macOS
/// stretches a drawable over its layer, so there the two match.
fn swap_chain_size(current: (u32, u32), picture: (u32, u32)) -> (u32, u32) {
  #[cfg(target_os = "windows")]
  {
    let edge = |current: u32, needed: u32| {
      let stepped = (needed + SWAP_CHAIN_MARGIN).div_ceil(SWAP_CHAIN_STEP) * SWAP_CHAIN_STEP;
      // Kept while it reaches past the picture and is at most a step too
      // big, so a picture easing back and forth does not reallocate.
      if current >= needed + SWAP_CHAIN_MARGIN && current <= stepped + SWAP_CHAIN_STEP {
        current
      } else {
        stepped
      }
    };
    (edge(current.0, picture.0), edge(current.1, picture.1))
  }
  #[cfg(not(target_os = "windows"))]
  {
    let _ = current;
    picture
  }
}

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
    let swap_chain = swap_chain_size(self.surface.size(), size);
    self.surface.resize(&self.gpu, swap_chain);
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
