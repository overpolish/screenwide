// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Annotate at work over a picture, for a window that shows what it does:
//! strokes made by the overlay's own gestures and drawn by its own shader,
//! on glass laid over the picture the page paints. It keeps a sketch of its
//! own, so the live overlay's session is never touched, and plays a session
//! ending by the rule the settings set: kept annotations stay on screen, as
//! the real tool leaves them, and others go with the session.

use std::time::{Duration, Instant};

use screenwide_annotations::reveal::AnnotationReveal;
use screenwide_annotations::{Annotation, AnnotationKind, AnnotationPoint};
use screenwide_compositor::annotation_gpu::placed_arrows;
use screenwide_gpu::surface::{Frame, Surface};

use super::geometry::display_annotation;
use super::input::Sketch;
use super::overlay::Renderer;
use super::settings::AnnotateSettings;

/// How long annotations kept from the last pass take to fade when the next
/// starts. Short enough to be gone before the next pass's first stroke.
const FADE: Duration = Duration::from_millis(250);
/// How often a fading frame is drawn.
const FRAME: Duration = Duration::from_millis(16);

/// How the page's settings dress what is drawn.
pub(crate) struct AnnotateLook {
  /// `#rrggbb`.
  pub(crate) color: String,
  /// Whether annotations outlast the session.
  pub(crate) keep: bool,
  /// The counter's diameter in points, and where its tail points.
  pub(crate) counter_size: f64,
  pub(crate) counter_angle: f64,
}

/// The glass and what is drawn on it, apart from where it is shown, so it
/// can be drawn into any target.
struct Glass {
  renderer: Renderer,
  /// Physical pixels, and how many to a point.
  size: (u32, u32),
  scale: f64,
  sketch: Sketch,
  /// The overlay's settings with the look the page shows.
  settings: AnnotateSettings,
  keep: bool,
  /// The tool the next press picks up.
  shape: AnnotationKind,
  /// Whether a session takes the pointer. Kept annotations stay drawn
  /// without one.
  session: bool,
  /// The last pass's kept annotations, fading out as the next begins, and
  /// how opaque they still are.
  leaving: Vec<Annotation>,
  leaving_opacity: f32,
}

impl Glass {
  fn new(size: (u32, u32), scale: f64) -> Result<Self, String> {
    Ok(Self {
      renderer: Renderer::new()?,
      size,
      scale,
      sketch: Sketch::default(),
      settings: super::settings::current(),
      keep: false,
      shape: AnnotationKind::Arrow,
      session: true,
      leaving: Vec::new(),
      leaving_opacity: 1.0,
    })
  }

  fn set_look(&mut self, look: AnnotateLook) {
    self.settings.default_color = look.color;
    self.settings.default_counter_size = look.counter_size;
    self.settings.default_counter_angle = look.counter_angle;
    self.keep = look.keep;
  }

  fn pointer(&mut self, phase: u32, point: AnnotationPoint) {
    if self.session {
      self
        .sketch
        .pointer(&self.settings, self.shape, phase, point);
    }
  }

  /// The session ends. Unkept annotations go at once, as the overlay
  /// takes them down; kept ones stay exactly as they are.
  fn exit(&mut self) {
    self.session = false;
    if !self.keep {
      self.sketch.clear();
    }
  }

  /// A fresh pass. What was kept leaves by fading rather than vanishing,
  /// so the loop's turn has no frame where it blinks.
  fn reset(&mut self) {
    self.leaving = self.sketch.annotations(&self.settings);
    self.leaving_opacity = 1.0;
    self.sketch.clear();
    self.shape = AnnotationKind::Arrow;
    self.session = true;
  }

  /// The fade `amount` of the way through, 0 to 1. Reports whether it is
  /// over, the leaving annotations gone.
  fn fade(&mut self, amount: f32) -> bool {
    let amount = amount.clamp(0.0, 1.0);
    self.leaving_opacity = 1.0 - amount * amount * (3.0 - 2.0 * amount);
    if amount >= 1.0 {
      self.leaving.clear();
    }
    self.leaving.is_empty()
  }

  /// What is on the glass, in the layer's pixels: the leaving annotations
  /// under the pass being drawn.
  fn drawn(&self) -> Vec<Annotation> {
    // A still annotation's opacity rides its reveal; the previous opacity
    // matches so the shader does not smear it as if it were moving.
    let opacity = self.leaving_opacity;
    let fading = AnnotationReveal {
      opacity,
      previous: [0.0, 1.0, 1.0, opacity],
      ..AnnotationReveal::WHOLE
    };
    let leaving = self.leaving.iter().map(|annotation| Annotation {
      reveal: fading,
      ..annotation.clone()
    });
    leaving
      .chain(self.sketch.annotations(&self.settings))
      .map(|annotation| display_annotation(&annotation, (0.0, 0.0), self.scale))
      .collect()
  }

  /// Draws the glass into `target`, `self.size` pixels in size.
  fn draw_into(&self, target: &wgpu::TextureView) -> Result<(), String> {
    // The annotations are already in the layer's pixels, so they are placed
    // through the identity, as the overlay places a display's.
    let prepared = placed_arrows(
      &self.drawn(),
      (0.0, 0.0),
      (1.0, 1.0),
      self.scale,
      None,
      None,
    );
    self
      .renderer
      .draw_arrows(target, self.size, &prepared, None, None)
  }
}

pub(crate) struct AnnotatePreview {
  glass: Glass,
  drawable: Surface,
  /// When the kept annotations began to fade, while they are.
  leaving_since: Option<Instant>,
}

impl AnnotatePreview {
  /// Draws into `drawable`, `size` physical pixels at `scale`.
  pub(crate) fn new(drawable: Surface, size: (u32, u32), scale: f64) -> Result<Self, String> {
    Ok(Self {
      glass: Glass::new(size, scale)?,
      drawable,
      leaving_since: None,
    })
  }

  pub(crate) fn set_look(&mut self, look: AnnotateLook) {
    self.glass.set_look(look);
    self.draw();
  }

  pub(crate) fn pick(&mut self, shape: AnnotationKind) {
    self.glass.shape = shape;
  }

  /// A pointer step at `point`, in the picture's points: 0 down, 1 drag,
  /// 2 up.
  pub(crate) fn pointer(&mut self, phase: u32, point: AnnotationPoint) {
    self.glass.pointer(phase, point);
    self.draw();
  }

  /// The session ends: the overlay goes, taking its annotations unless they
  /// are kept.
  pub(crate) fn exit(&mut self) {
    self.glass.exit();
    self.draw();
  }

  /// The next session starts. Nothing on screen changes: what was kept is
  /// still there, and nothing else was.
  pub(crate) fn enter(&mut self) {
    self.glass.session = true;
  }

  /// Back to a fresh session, what was kept fading away; when the next of
  /// the fade's frames is due.
  pub(crate) fn reset(&mut self) -> Option<Duration> {
    self.glass.reset();
    self.leaving_since = Some(Instant::now());
    self.frame()
  }

  /// Draws the fade as it stands now, and says when the next frame is due
  /// while it lasts.
  pub(crate) fn frame(&mut self) -> Option<Duration> {
    let since = self.leaving_since?;
    let gone = self
      .glass
      .fade(since.elapsed().as_secs_f32() / FADE.as_secs_f32());
    self.draw();
    if gone {
      self.leaving_since = None;
      return None;
    }
    Some(FRAME)
  }

  fn draw(&mut self) {
    if let Err(error) = self.render() {
      eprintln!("The Annotate preview dropped a frame: {error}");
    }
  }

  fn render(&mut self) -> Result<(), String> {
    let gpu = self.glass.renderer.gpu();
    self.drawable.resize(gpu, self.glass.size);
    let Frame::Ready(frame) = self.drawable.acquire(gpu)? else {
      return Ok(());
    };
    let target = frame.texture.create_view(&Default::default());
    self.glass.draw_into(&target)?;
    gpu.queue.present(frame);
    Ok(())
  }
}

#[cfg(test)]
#[path = "preview_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "preview_loop_tests.rs"]
mod loop_tests;
