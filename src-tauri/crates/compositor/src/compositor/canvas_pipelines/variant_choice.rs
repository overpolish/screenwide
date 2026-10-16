// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How a compositor's owner steers which canvas variants its draws take: an
//! export holding them to the one with every kind, and a preview compiling
//! what it misses in the background; see the parent module's notes.

use std::sync::atomic::Ordering;
use std::sync::Arc;

use super::super::canvas_features::ALL_FEATURES;
use super::super::canvas_variants::every_kind;
use super::super::Compositor;
use super::Target;

impl Compositor {
  /// Draws every annotated frame with the variant with every kind until the
  /// hold drops, as an export wants; see the module's notes. The Windows
  /// export shares the preview's compositor, so it holds this only while
  /// the preview is held off it.
  #[must_use = "the hold ends when it drops"]
  pub fn hold_every_annotation_kind(&self) -> EveryAnnotationKind<'_> {
    self.every_annotation_kind.store(true, Ordering::Relaxed);
    EveryAnnotationKind(self)
  }

  /// Has a draw that is missing its variant compile it in the background
  /// from now on, as a preview wants, and makes ready the variants a preview
  /// leans on: the one with no annotation, which most of its frames draw
  /// with, and the one with every kind, which draws any frame whose own
  /// variant is still compiling, and every annotated frame on Windows - onto
  /// a whole canvas, as a layer laid over others, which a stacked
  /// screenshot's layers and what lies over a focus are drawn as, and as
  /// what the annotations under a blurring spotlight add. One with none of
  /// its own ready would otherwise be compiled where it is drawn, holding
  /// the frame.
  pub fn compile_annotation_kinds_in_background(&self) {
    let canvas = &self.canvas;
    canvas.in_background.store(true, Ordering::Relaxed);
    let (none, every) = (
      canvas.variant(0, ALL_FEATURES),
      canvas.variant(every_kind(), ALL_FEATURES),
    );
    let warm = [
      (none, Target::Canvas),
      (Arc::clone(&every), Target::Canvas),
      (Arc::clone(&every), Target::Layer),
      (every, Target::Delta),
    ];
    let (device, layout) = (canvas.device.clone(), canvas.layout.clone());
    // One thread, the variant with none first: a preview mostly draws with
    // it, and the one with every kind takes many times as long to compile.
    let _ = std::thread::Builder::new()
      .name("canvas-shader-warm".to_owned())
      .spawn(move || {
        for (variant, target) in warm {
          variant.pipeline(&device, &layout, target);
        }
      });
  }
}

/// See [`Compositor::hold_every_annotation_kind`].
pub struct EveryAnnotationKind<'a>(&'a Compositor);

impl Drop for EveryAnnotationKind<'_> {
  fn drop(&mut self) {
    self.0.every_annotation_kind.store(false, Ordering::Relaxed);
  }
}
