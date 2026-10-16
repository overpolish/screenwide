// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The canvas pass's pipelines: a variant for each set of annotation kinds a
//! draw shows ([`super::canvas_variants`]), each for a whole canvas, for a
//! screenshot layer blended over the layers under it, or for the pass that
//! draws what the annotations under a blurring spotlight add
//! ([`super::mark_blur`]).
//!
//! A variant is compiled the first time something asks for it, and every
//! compositor on the device shares them. The variant with no annotation and
//! the one with every kind come compiled from the build
//! ([`super::canvas_precompiled`]), so all that is left of them here is the
//! GPU driver's own pass, which the system keeps. Any other set is compiled
//! from WGSL when first asked for: on Windows that takes seconds, every
//! launch, as nothing keeps DXC's output. A draw takes its own variant where
//! it is ready; otherwise the smallest ready variant holding all its kinds,
//! which draws the same picture more slowly; and only otherwise waits for its
//! own. A preview keeps the variant with every kind ready, so a kind new to
//! what it draws is drawn at once, and compiles the set it was missing in the
//! background, which the frames after it change to.
//!
//! An export draws every annotated frame with the variant with every kind
//! ([`Compositor::hold_every_annotation_kind`]) and compiles nothing.
//! Compiling its own sets held a Windows export over two minutes before its
//! first frame (12 sets, 8 cores), while the variant with every kind cost
//! about 1 ms more an annotated 1080p frame: it wins on any export with less
//! than about 40 minutes of annotated video.
//!
//! On Windows a preview does the same. Measured there, a frame drawn with
//! the variant with every kind cost the GPU no more than one drawn with its
//! own set (2.00 against 1.91 ms on an RTX 2070 Super, 67.6 against 72.6 ms
//! on a Ryzen 7700X's integrated GPU), while compiling the set took DXC
//! seconds of CPU on every launch and the frames drawn while it ran
//! stuttered.
//!
//! On macOS each of those draws changes to a slim variant once one is ready:
//! the same kinds, keeping only the shader's other paths the frame takes
//! ([`super::canvas_features`]). Metal compiles it in the background the
//! first time a frame asks for it, in about half a second with no annotation
//! and about two with every kind, and the system keeps it for later launches;
//! the frames until then are drawn as above.

/// Whether every annotated draw takes the variant with every kind; see the
/// module's notes.
const ALWAYS_EVERY_KIND: bool = cfg!(target_os = "windows");

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock, PoisonError};

use super::canvas_features::{FeatureMask, ALL_FEATURES};
use super::canvas_modules::{self, CanvasModules};
use super::canvas_variants::{covers, every_kind, KindMask};
use screenwide_gpu::Gpu;

mod variant_choice;
pub use variant_choice::EveryAnnotationKind;

/// What a canvas pipeline draws into.
#[derive(Clone, Copy)]
pub(super) enum Target {
  /// A whole canvas, written over.
  Canvas = 0,
  /// A screenshot layer, blended over the layers under it.
  Layer = 1,
  /// The layer of what the annotations under a blurring spotlight add.
  Delta = 2,
}

pub(super) struct CanvasPipelines {
  device: wgpu::Device,
  layout: wgpu::PipelineLayout,
  variants: Mutex<HashMap<(KindMask, FeatureMask), Arc<Variant>>>,
  /// Whether a draw missing its variant starts compiling it on another
  /// thread, as a preview's does, rather than only waiting for it.
  in_background: AtomicBool,
  /// Whether the variants the build compiled are used, rather than compiled
  /// here from WGSL like any other.
  precompiled: bool,
}

/// The canvas pipelines every compositor on `gpu` shares, and the bind group
/// layout their bindings are made against: two previews, the stills and an
/// export then compile each variant once between them.
pub(super) fn shared(
  gpu: &'static Gpu,
  bindings: impl FnOnce() -> wgpu::BindGroupLayout,
) -> (wgpu::BindGroupLayout, Arc<CanvasPipelines>) {
  type Shared = (usize, wgpu::BindGroupLayout, Arc<CanvasPipelines>);
  static SHARED: Mutex<Vec<Shared>> = Mutex::new(Vec::new());
  let device = std::ptr::from_ref(gpu) as usize;
  let mut shared = SHARED.lock().unwrap_or_else(PoisonError::into_inner);
  if let Some((_, layout, canvas)) = shared.iter().find(|(at, ..)| *at == device) {
    return (layout.clone(), Arc::clone(canvas));
  }
  let layout = bindings();
  let canvas = Arc::new(CanvasPipelines::new(&gpu.device, &layout));
  shared.push((device, layout.clone(), Arc::clone(&canvas)));
  (layout, canvas)
}

/// The canvas shader built for `kinds` and `features`, compiled for each
/// target as asked.
struct Variant {
  kinds: KindMask,
  features: FeatureMask,
  precompiled: bool,
  modules: OnceLock<CanvasModules>,
  pipelines: [OnceLock<wgpu::RenderPipeline>; 3],
  /// Which targets a background thread is compiling already.
  queued: [AtomicBool; 3],
}

impl Variant {
  /// The pipeline for `target`, compiled here unless it is ready. A thread
  /// compiling it already is waited for rather than repeated.
  fn pipeline(
    &self,
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    target: Target,
  ) -> &wgpu::RenderPipeline {
    self.pipelines[target as usize].get_or_init(|| {
      let modules = self.modules.get_or_init(|| {
        canvas_modules::modules(device, self.kinds, self.features, self.precompiled)
      });
      canvas_modules::pipeline(device, layout, modules, target)
    })
  }
}

impl CanvasPipelines {
  pub(super) fn new(device: &wgpu::Device, bindings: &wgpu::BindGroupLayout) -> Self {
    Self::with_precompiled(device, bindings, true)
  }

  /// Pipelines that compile every variant from WGSL, the precompiled ones
  /// included: what the build's output is checked against.
  #[cfg(test)]
  pub(super) fn compiled_from_wgsl(
    device: &wgpu::Device,
    bindings: &wgpu::BindGroupLayout,
  ) -> Self {
    Self::with_precompiled(device, bindings, false)
  }

  fn with_precompiled(
    device: &wgpu::Device,
    bindings: &wgpu::BindGroupLayout,
    precompiled: bool,
  ) -> Self {
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("Screenwide preview layout"),
      bind_group_layouts: &[Some(bindings)],
      immediate_size: 0,
    });
    Self {
      device: device.clone(),
      layout,
      variants: Mutex::default(),
      in_background: AtomicBool::new(false),
      precompiled,
    }
  }

  fn variant(&self, kinds: KindMask, features: FeatureMask) -> Arc<Variant> {
    let mut variants = self.variants.lock().unwrap_or_else(PoisonError::into_inner);
    Arc::clone(variants.entry((kinds, features)).or_insert_with(|| {
      Arc::new(Variant {
        kinds,
        features,
        precompiled: self.precompiled,
        modules: OnceLock::new(),
        pipelines: Default::default(),
        queued: Default::default(),
      })
    }))
  }

  /// The pipeline that draws `kinds` into `target` keeping `features`: the
  /// slim variant keeping only those where it is ready, starting it compiling
  /// in the background where it is not; otherwise one keeping every feature:
  /// with `every`, the one with every kind for any annotated draw; otherwise
  /// their own where it is ready, else the smallest ready one holding them
  /// all, else their own, compiled now.
  pub(super) fn get(
    &self,
    target: Target,
    kinds: KindMask,
    every: bool,
    features: FeatureMask,
  ) -> wgpu::RenderPipeline {
    if features != ALL_FEATURES {
      let slim = self.variant(kinds, features);
      if let Some(pipeline) = slim.pipelines[target as usize].get() {
        return pipeline.clone();
      }
      #[cfg(test)]
      if super::canvas_features::TEST_SLIM.get() == Some(true) {
        return slim.pipeline(&self.device, &self.layout, target).clone();
      }
      self.compile_in_background(&slim, target);
    }
    if (every || ALWAYS_EVERY_KIND) && kinds != 0 {
      return self
        .variant(every_kind(), ALL_FEATURES)
        .pipeline(&self.device, &self.layout, target)
        .clone();
    }
    let own = self.variant(kinds, ALL_FEATURES);
    if let Some(pipeline) = own.pipelines[target as usize].get() {
      return pipeline.clone();
    }
    if self.in_background.load(Ordering::Relaxed) {
      self.compile_in_background(&own, target);
    }
    if let Some(pipeline) = self.ready_cover(target, kinds) {
      return pipeline;
    }
    own.pipeline(&self.device, &self.layout, target).clone()
  }

  /// The smallest ready variant for `target` keeping every feature that holds
  /// every kind in `kinds`, if one is.
  fn ready_cover(&self, target: Target, kinds: KindMask) -> Option<wgpu::RenderPipeline> {
    let variants = self.variants.lock().ok()?;
    variants
      .values()
      .filter(|variant| variant.features == ALL_FEATURES && covers(variant.kinds, kinds))
      .filter_map(|variant| {
        let pipeline = variant.pipelines[target as usize].get()?;
        Some((variant.kinds.count_ones(), pipeline))
      })
      .min_by_key(|(count, _)| *count)
      .map(|(_, pipeline)| pipeline.clone())
  }

  fn compile_in_background(&self, variant: &Arc<Variant>, target: Target) {
    if variant.queued[target as usize].swap(true, Ordering::AcqRel) {
      return;
    }
    let (task, device, layout) = (
      Arc::clone(variant),
      self.device.clone(),
      self.layout.clone(),
    );
    let spawned = std::thread::Builder::new()
      .name("canvas-shader".to_owned())
      .spawn(move || {
        task.pipeline(&device, &layout, target);
      });
    if spawned.is_err() {
      // Without a thread, the draw that next asks compiles it instead.
      variant.queued[target as usize].store(false, Ordering::Release);
    }
  }
}
