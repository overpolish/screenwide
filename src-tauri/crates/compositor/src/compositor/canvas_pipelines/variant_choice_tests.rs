// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A recording with scenes has each set's moving twin compiling as the set
//! is first drawn, so its first moving frame is not drawn with every path
//! kept while the twin compiles.

use std::sync::atomic::Ordering;

use screenwide_annotations::AnnotationKind;

use super::super::canvas_features::{FeatureMask, SCENE_MOTION};
use super::super::canvas_variants::{kind_bit, KindMask};
use super::{CanvasPipelines, Target};

/// The paths a paused recording frame keeps: the cursor and a mesh
/// background.
const PAUSED: FeatureMask = 0x201;

/// Pipelines as a preview has them, compiling what it misses in the
/// background, and shared with nothing else.
fn preview() -> CanvasPipelines {
  let gpu = screenwide_gpu::shared(screenwide_gpu::Workload::Heavy).expect("the shared GPU");
  let canvas = CanvasPipelines::new(&gpu.device);
  canvas.in_background.store(true, Ordering::Relaxed);
  canvas
}

fn compiling(canvas: &CanvasPipelines, kinds: KindMask, target: Target) -> bool {
  canvas.variant(kinds, PAUSED | SCENE_MOTION).queued[target as usize].load(Ordering::Acquire)
}

#[test]
fn a_lens_drawn_while_scenes_move_has_its_moving_twin_compiling() {
  let canvas = preview();
  canvas.scenes_move.store(true, Ordering::Relaxed);
  let lens = kind_bit(AnnotationKind::Magnify);
  canvas.remember(0, Target::Layer, lens, PAUSED);
  assert!(compiling(&canvas, lens, Target::Layer));
  // A lens's ground is drawn by the third pipeline, which is never noted.
  assert!(compiling(&canvas, lens, Target::Delta));
}

#[test]
fn nothing_compiles_ahead_while_scenes_stay_still() {
  let canvas = preview();
  let lens = kind_bit(AnnotationKind::Magnify);
  canvas.remember(0, Target::Layer, lens, PAUSED);
  assert!(!compiling(&canvas, lens, Target::Layer));
  assert!(!compiling(&canvas, lens, Target::Delta));
}
