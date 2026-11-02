// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A recording scene played on a window's surface: composed into a canvas
//! of the picture's size on screen, then copied onto the surface, once
//! every display refresh while it moves and the preview is shown. The
//! surface paces it: it is set to wait for the display, so taking a frame
//! from it waits for the refresh that frees one, and the scene's time is
//! counted in those refreshes (`FrameClock`).

use std::time::{Duration, Instant};

use screenwide_gpu::surface::{Frame, Surface};

use super::clock::FrameClock;
use super::content::canvas_texture;
use super::render::Moment;
use super::{PreviewScene, SceneRenderer};
use crate::worker::Renderer;

/// How long to wait before asking again for a frame the surface had none
/// for, as while the window is hidden.
const RETRY: Duration = Duration::from_millis(16);

pub(crate) struct SceneDriver {
  renderer: SceneRenderer,
  surface: Surface,
  gpu: &'static screenwide_gpu::Gpu,
  canvas: Option<wgpu::Texture>,
  size: (u32, u32),
  visible: bool,
  clock: FrameClock,
  /// What the canvas holds.
  composed: Option<(Moment, (u32, u32))>,
  /// Whether the surface shows the canvas as it is, which is all a still
  /// needs.
  shown: bool,
}

impl SceneDriver {
  pub(crate) fn new(renderer: SceneRenderer, mut surface: Surface) -> Result<Self, String> {
    let gpu = screenwide_gpu::shared()?;
    surface.accept_copies(gpu);
    surface.wait_for_display(gpu);
    Ok(Self {
      renderer,
      surface,
      gpu,
      canvas: None,
      size: (0, 0),
      visible: false,
      clock: FrameClock::default(),
      composed: None,
      shown: false,
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
      _ => canvas_texture(self.gpu, size),
    };
    let composed = self
      .renderer
      .draw(&canvas.create_view(&Default::default()), size, moment);
    self.canvas = Some(canvas);
    composed?;
    self.composed = Some((moment, size));
    Ok(())
  }

  /// Takes a frame from the surface, waiting for the display to free one,
  /// and shows the moment of the scene it stands for on it. `false` when
  /// the surface had no frame to give.
  fn present(&mut self) -> Result<bool, String> {
    let Frame::Ready(frame) = self.surface.acquire(self.gpu)? else {
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
    encoder.copy_texture_to_texture(
      canvas.as_image_copy(),
      frame.texture.as_image_copy(),
      wgpu::Extent3d {
        width: self.size.0,
        height: self.size.1,
        depth_or_array_layers: 1,
      },
    );
    self.gpu.queue.submit([encoder.finish()]);
    self.gpu.queue.present(frame);
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
    self.surface.resize(self.gpu, size);
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
