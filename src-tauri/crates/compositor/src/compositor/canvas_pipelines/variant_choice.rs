// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How a compositor's owner steers which canvas variants its draws take: an
//! export holding them to the one with every kind, and a preview compiling
//! what it misses in the background; see the parent module's notes.

use std::sync::atomic::Ordering;
use std::sync::Arc;

use screenwide_annotations::AnnotationKind;

use super::super::canvas_features::{self, FeatureMask, ALL_FEATURES};
use super::super::canvas_variants::{every_kind, kind_bit, KindMask};
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
    let device = canvas.device.clone();
    // One thread, the variant with none first: a preview mostly draws with
    // it, and the one with every kind takes many times as long to compile.
    let _ = std::thread::Builder::new()
      .name("canvas-shader-warm".to_owned())
      .spawn(move || {
        for (variant, target) in warm {
          variant.pipeline(&device, target);
        }
      });
  }

  /// Starts compiling, in the background, the slim variants a stroke of
  /// `kind` will be drawn with: each placed layer's last set with `kind`
  /// added, keeping the paths it was last drawn with and those `kind`
  /// brings. A pen taken up calls this, so its first stroke is drawn with
  /// its own variant rather than with every path kept while that compiles.
  /// A variant ready already is left as it is; so is everything where
  /// frames keep every path, as on Windows.
  pub fn prepare_annotation_kind(&self, kind: AnnotationKind) {
    let canvas = &self.canvas;
    let bit = kind_bit(kind);
    let brought = canvas_features::of_kind(kind);
    // A spotlight's blur and a lens's ground are drawn by the variant's
    // third pipeline.
    let draws_delta = matches!(kind, AnnotationKind::Spotlight | AnnotationKind::Magnify);
    let drawn = canvas
      .drawn
      .lock()
      .map(|drawn| drawn.values().copied().collect::<Vec<_>>())
      .unwrap_or_default();
    for (target, kinds, features) in drawn {
      let variant = canvas.variant(kinds | bit, features | brought);
      let delta = draws_delta.then_some(Target::Delta);
      for target in std::iter::once(target).chain(delta) {
        if variant.pipelines[target as usize].get().is_none() {
          canvas.compile_in_background(&variant, target);
        }
      }
    }
  }
}

impl super::CanvasPipelines {
  /// Notes what the placed layer in `slot` was drawn into `target` with,
  /// for [`Compositor::prepare_annotation_kind`]: only where frames are
  /// drawn slim and a preview compiles in the background.
  pub(in super::super) fn remember(
    &self,
    slot: usize,
    target: Target,
    kinds: KindMask,
    features: FeatureMask,
  ) {
    if features == ALL_FEATURES || !self.in_background.load(Ordering::Relaxed) {
      return;
    }
    if let Ok(mut drawn) = self.drawn.lock() {
      drawn.insert((slot, target as usize), (target, kinds, features));
    }
  }
}

/// See [`Compositor::hold_every_annotation_kind`].
pub struct EveryAnnotationKind<'a>(&'a Compositor);

impl Drop for EveryAnnotationKind<'_> {
  fn drop(&mut self) {
    self.0.every_annotation_kind.store(false, Ordering::Relaxed);
  }
}
