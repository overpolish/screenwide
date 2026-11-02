// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Annotate at work in a preview. It draws only when something changes: live
//! annotations are drawn whole, with nothing animating between inputs.

use std::time::Duration;

use screenwide_annotations::{AnnotationKind, AnnotationPoint};
use screenwide_settings_preview::scene::{PreviewInput, PreviewScene};
use screenwide_settings_preview::Renderer;

use crate::annotate::preview::{AnnotateLook, AnnotatePreview};

/// The overlay's pointer phases.
const DOWN: u32 = 0;
const DRAG: u32 = 1;
const UP: u32 = 2;

pub(super) struct AnnotateRenderer(AnnotatePreview);

impl AnnotateRenderer {
  /// Annotate on glass `pixels` in size, `scale` pixels to the point,
  /// dressed as `look` says.
  pub(super) fn new(
    drawable: screenwide_gpu::surface::Surface,
    pixels: (u32, u32),
    scale: f64,
    look: AnnotateLook,
  ) -> Result<Self, String> {
    let mut preview = AnnotatePreview::new(drawable, pixels, scale)?;
    preview.set_look(look);
    Ok(Self(preview))
  }
}

impl Renderer for AnnotateRenderer {
  fn input(&mut self, input: PreviewInput, x: f64, y: f64) -> Option<Duration> {
    let preview = &mut self.0;
    let point = AnnotationPoint { x, y };
    match input {
      PreviewInput::Down => preview.pointer(DOWN, point),
      PreviewInput::Drag => preview.pointer(DRAG, point),
      PreviewInput::Up => preview.pointer(UP, point),
      PreviewInput::PickArrow => preview.pick(AnnotationKind::Arrow),
      PreviewInput::PickCounter => preview.pick(AnnotationKind::Counter),
      PreviewInput::Exit => preview.exit(),
      PreviewInput::Enter => preview.enter(),
      PreviewInput::Reset => return preview.reset(),
      PreviewInput::Hover => {}
    }
    None
  }

  fn scene(&mut self, scene: PreviewScene) -> Option<Duration> {
    if let Some(look) = look(scene) {
      self.0.set_look(look);
    }
    None
  }

  fn frame(&mut self) -> Option<Duration> {
    self.0.frame()
  }
}

/// How an Annotate scene dresses what is drawn.
pub(super) fn look(scene: PreviewScene) -> Option<AnnotateLook> {
  let PreviewScene::Annotate {
    color,
    keep,
    counter_size,
    counter_angle,
  } = scene
  else {
    return None;
  };
  Some(AnnotateLook {
    color,
    keep,
    counter_size,
    counter_angle,
  })
}
