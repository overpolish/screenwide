// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Annotate at work over a picture, for a window that shows what it does:
//! strokes made by the overlay's own gestures and drawn by its own shader,
//! on glass laid over the picture the page paints. It keeps a sketch of its
//! own, so the live overlay's session is never touched, and plays a session
//! ending and the next starting by the rule the settings set: kept
//! annotations come back, others go with the session.

use screenwide_annotations::{AnnotationKind, AnnotationPoint};
use screenwide_compositor::annotation_gpu::placed_arrows;
use screenwide_gpu::surface::{Frame, Surface};

use super::geometry::display_annotation;
use super::input::Sketch;
use super::overlay::Renderer;
use super::settings::AnnotateSettings;

pub(crate) struct AnnotatePreview {
  renderer: Renderer,
  drawable: Surface,
  /// Physical pixels, and how many to a point.
  size: (u32, u32),
  scale: f64,
  sketch: Sketch,
  /// The overlay's settings with the colour the page shows.
  settings: AnnotateSettings,
  keep: bool,
  /// The tool the next press picks up.
  shape: AnnotationKind,
  /// Whether a session is on screen.
  shown: bool,
}

impl AnnotatePreview {
  /// Draws into `drawable`, `size` physical pixels at `scale`.
  pub(crate) fn new(drawable: Surface, size: (u32, u32), scale: f64) -> Result<Self, String> {
    Ok(Self {
      renderer: Renderer::new()?,
      drawable,
      size,
      scale,
      sketch: Sketch::default(),
      settings: super::settings::current(),
      keep: false,
      shape: AnnotationKind::Arrow,
      shown: true,
    })
  }

  /// The colour new annotations take, `#rrggbb`, and whether they outlast
  /// the session.
  pub(crate) fn set_look(&mut self, color: String, keep: bool) {
    self.settings.default_color = color;
    self.keep = keep;
    self.draw();
  }

  pub(crate) fn pick(&mut self, shape: AnnotationKind) {
    self.shape = shape;
  }

  /// A pointer step at `point`, in the picture's points: 0 down, 1 drag,
  /// 2 up.
  pub(crate) fn pointer(&mut self, phase: u32, point: AnnotationPoint) {
    if self.shown {
      self
        .sketch
        .pointer(&self.settings, self.shape, phase, point);
      self.draw();
    }
  }

  /// The session ends: the overlay goes, taking its annotations unless they
  /// are kept.
  pub(crate) fn exit(&mut self) {
    self.shown = false;
    if !self.keep {
      self.sketch.clear();
    }
    self.draw();
  }

  /// The next session starts, with whatever was kept.
  pub(crate) fn enter(&mut self) {
    self.shown = true;
    self.draw();
  }

  /// Back to a fresh session with nothing drawn.
  pub(crate) fn reset(&mut self) {
    self.sketch.clear();
    self.shape = AnnotationKind::Arrow;
    self.shown = true;
    self.draw();
  }

  fn draw(&mut self) {
    if let Err(error) = self.render() {
      eprintln!("The Annotate preview dropped a frame: {error}");
    }
  }

  fn render(&mut self) -> Result<(), String> {
    let gpu = self.renderer.gpu();
    self.drawable.resize(gpu, self.size);
    let Frame::Ready(frame) = self.drawable.acquire(gpu)? else {
      return Ok(());
    };
    let drawn: Vec<_> = if self.shown {
      self
        .sketch
        .annotations(&self.settings)
        .iter()
        .map(|annotation| display_annotation(annotation, (0.0, 0.0), self.scale))
        .collect()
    } else {
      Vec::new()
    };
    // The annotations are already in the layer's pixels, so they are placed
    // through the identity, as the overlay places a display's.
    let prepared = placed_arrows(&drawn, (0.0, 0.0), (1.0, 1.0), self.scale, None, None);
    let target = frame.texture.create_view(&Default::default());
    self
      .renderer
      .draw_arrows(&target, self.size, &prepared, None, None)?;
    gpu.queue.present(frame);
    Ok(())
  }
}
