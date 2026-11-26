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
  ///
  /// Where frames are drawn slim (macOS), two slim variants keeping every
  /// path but a focus come last ([`canvas_features::ALL_BUT_FOCUS`]): one
  /// with no annotation, and one with every kind but a lens and a focus
  /// (`playback_kinds`). A frame whose own slim variant is still compiling
  /// draws with the smaller of them that holds it rather than with every
  /// path kept: the first scene a recording's playback moves, turns or
  /// blooms a background into, and the first annotation of a kind it shows,
  /// ask for a variant of their own. Measured on a 1080p canvas while that
  /// compiled, a frame without annotations cost the GPU 10 to 17 ms with
  /// every path kept and 2 to 6 with the first; one with an arrow, a shape
  /// and text 31 ms, and 7 with the second. A focus's path, and a lens's,
  /// each cost nearly as much as every path does, so a frame with either
  /// draws as before.
  pub fn compile_annotation_kinds_in_background(&self) {
    let canvas = &self.canvas;
    canvas.in_background.store(true, Ordering::Relaxed);
    let (none, every) = (
      canvas.variant(0, ALL_FEATURES),
      canvas.variant(every_kind(), ALL_FEATURES),
    );
    let mut warm = vec![
      (none, Target::Canvas),
      (Arc::clone(&every), Target::Canvas),
      (Arc::clone(&every), Target::Layer),
      (every, Target::Delta),
    ];
    if cfg!(target_os = "macos") {
      let plain = canvas.variant(0, canvas_features::ALL_BUT_FOCUS);
      let annotated = canvas.variant(playback_kinds(), canvas_features::ALL_BUT_FOCUS);
      // A preview places its canvas, so it is drawn as a layer; a
      // spotlight's blur is drawn into its own.
      warm.extend([
        (Arc::clone(&plain), Target::Layer),
        (plain, Target::Canvas),
        (Arc::clone(&annotated), Target::Layer),
        (Arc::clone(&annotated), Target::Delta),
        (annotated, Target::Canvas),
      ]);
    }
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
    let draws_delta = bit & delta_kinds() != 0;
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

  /// Says whether the frames drawn from now on may move a scene, as a
  /// recording's do once it has scenes. While they may, each set a placed
  /// layer is drawn with has its moving twin, the same set with the scene's
  /// motion path kept, compiled in the background as the set is first drawn,
  /// usually while the playhead rests, well before a playback moves the
  /// scene. A frame holding a lens or a focus has no stand-in
  /// (`playback_kinds`), so its first moving frame would otherwise be drawn
  /// with every path kept until its own compiled: measured at 15 to 90 ms a
  /// frame for over a second, with a lens on the first zoom of a playback.
  /// A twin ready already, or one a ready stand-in covers, is left as it is;
  /// so is everything where frames keep every path, as on Windows.
  pub fn set_scenes_move(&self, moves: bool) {
    let canvas = &self.canvas;
    if canvas.scenes_move.swap(moves, Ordering::Relaxed) || !moves {
      return;
    }
    let drawn = canvas
      .drawn
      .lock()
      .map(|drawn| drawn.values().copied().collect::<Vec<_>>())
      .unwrap_or_default();
    for (target, kinds, features) in drawn {
      canvas.prepare_moving(target, kinds, features);
    }
  }
}

impl super::CanvasPipelines {
  /// Notes what the placed layer in `slot` was drawn into `target` with,
  /// for [`Compositor::prepare_annotation_kind`], and has a set new to it
  /// ready for a moving scene ([`Compositor::set_scenes_move`]): only where
  /// frames are drawn slim and a preview compiles in the background.
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
    let set = (target, kinds, features);
    let Ok(mut drawn) = self.drawn.lock() else {
      return;
    };
    let is_new = drawn.insert((slot, target as usize), set) != Some(set);
    drop(drawn);
    if is_new && self.scenes_move.load(Ordering::Relaxed) {
      self.prepare_moving(target, kinds, features);
    }
  }

  /// Starts compiling the set's moving twin unless it, or a stand-in that
  /// covers it, is ready: for `target`, and for the third pipeline where
  /// the set draws with it (`delta_kinds`).
  fn prepare_moving(&self, target: Target, kinds: KindMask, features: FeatureMask) {
    let moving = features | canvas_features::SCENE_MOTION;
    let variant = self.variant(kinds, moving);
    let delta = (kinds & delta_kinds() != 0).then_some(Target::Delta);
    for target in std::iter::once(target).chain(delta) {
      if variant.pipelines[target as usize].get().is_none()
        && self.ready_slim_cover(target, kinds, moving).is_none()
      {
        self.compile_in_background(&variant, target);
      }
    }
  }
}

/// The kinds a set also draws with the variant's third pipeline: a
/// spotlight's blur and a lens's ground.
fn delta_kinds() -> KindMask {
  kind_bit(AnnotationKind::Spotlight) | kind_bit(AnnotationKind::Magnify)
}

/// The kinds the annotated playback variant holds: every kind but a lens
/// (`Magnify`), whose path alone made a frame cost nearly what keeping every
/// path does, and a focus, whose path the variant leaves out.
fn playback_kinds() -> KindMask {
  every_kind() & !kind_bit(AnnotationKind::Magnify) & !kind_bit(AnnotationKind::Focus)
}

/// See [`Compositor::hold_every_annotation_kind`].
pub struct EveryAnnotationKind<'a>(&'a Compositor);

impl Drop for EveryAnnotationKind<'_> {
  fn drop(&mut self) {
    self.0.every_annotation_kind.store(false, Ordering::Relaxed);
  }
}
