// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The canvas pass's pipelines: a variant for each set of annotation kinds a
//! draw shows ([`super::canvas_variants`]), each for a whole canvas, for a
//! screenshot layer blended over the layers under it, or for the pass that
//! draws what the annotations under a blurring spotlight add
//! ([`super::mark_blur`]).
//!
//! A variant is compiled the first time something asks for it, and every
//! compositor on the device shares them. The first time on a machine that
//! takes from a third of a second for one kind to about twenty seconds for
//! them all; the system keeps what it compiled, so later runs find it in
//! milliseconds. A draw takes its own variant where it is ready; otherwise
//! the smallest ready variant holding all its kinds, which draws the same
//! picture more slowly; and only otherwise waits for its own. An export
//! prepares every set its timeline shows before its first frame, all at
//! once. A preview keeps the variant with every kind ready, so a kind new to
//! what it draws is drawn at once, and compiles the set it was missing in the
//! background, which the frames after it change to.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock, PoisonError};

use super::canvas_variants::{covers, every_kind, source, KindMask};
use super::{mark_blur::LAYER_FORMAT, Compositor, FORMAT};
use crate::gpu::Gpu;

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
  variants: Mutex<HashMap<KindMask, Arc<Variant>>>,
  /// Whether a draw missing its variant starts compiling it on another
  /// thread, as a preview's does, rather than only waiting for it.
  in_background: AtomicBool,
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

/// The canvas shader built for `kinds`, compiled for each target as asked.
struct Variant {
  kinds: KindMask,
  module: OnceLock<wgpu::ShaderModule>,
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
      let module = self.module.get_or_init(|| {
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
          label: Some("Screenwide preview shader"),
          source: wgpu::ShaderSource::Wgsl(source(self.kinds).into()),
        })
      });
      pipeline(device, layout, module, target)
    })
  }
}

impl CanvasPipelines {
  pub(super) fn new(device: &wgpu::Device, bindings: &wgpu::BindGroupLayout) -> Self {
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
    }
  }

  fn variant(&self, kinds: KindMask) -> Arc<Variant> {
    let mut variants = self.variants.lock().unwrap_or_else(PoisonError::into_inner);
    Arc::clone(variants.entry(kinds).or_insert_with(|| {
      Arc::new(Variant {
        kinds,
        module: OnceLock::new(),
        pipelines: Default::default(),
        queued: Default::default(),
      })
    }))
  }

  /// The pipeline that draws `kinds` into `target`: their own where it is
  /// ready, else the smallest ready one holding them all, else their own,
  /// compiled now.
  pub(super) fn get(&self, target: Target, kinds: KindMask) -> wgpu::RenderPipeline {
    let own = self.variant(kinds);
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

  /// The smallest ready variant for `target` that holds every kind in
  /// `kinds`, if one is.
  fn ready_cover(&self, target: Target, kinds: KindMask) -> Option<wgpu::RenderPipeline> {
    let variants = self.variants.lock().ok()?;
    variants
      .values()
      .filter(|variant| covers(variant.kinds, kinds))
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

impl Compositor {
  /// Compiles the canvas pipeline for each set of kinds in `sets`, and the
  /// one with none, all at once, returning when every one is ready: every
  /// frame of an export then finds a variant holding what it shows without
  /// waiting on a compile, and a frame showing nothing draws with no
  /// annotation code at all rather than borrowing a set's.
  pub(crate) fn prepare_annotation_kinds(&self, sets: &[KindMask]) {
    let canvas = &self.canvas;
    let variants: Vec<_> = std::iter::once(0)
      .chain(sets.iter().copied())
      .map(|kinds| canvas.variant(kinds))
      .collect();
    std::thread::scope(|scope| {
      for variant in &variants {
        scope.spawn(|| {
          variant.pipeline(&canvas.device, &canvas.layout, Target::Canvas);
        });
      }
    });
  }

  /// Has a draw that is missing its variant compile it in the background
  /// from now on, as a preview wants, and makes ready the two variants a
  /// preview leans on: the one with no annotation, which most of its frames
  /// draw with, and the one with every kind, which draws any frame whose own
  /// variant is still compiling.
  pub(crate) fn compile_annotation_kinds_in_background(&self) {
    let canvas = &self.canvas;
    canvas.in_background.store(true, Ordering::Relaxed);
    let variants: Vec<_> = [0, every_kind()]
      .into_iter()
      .map(|kinds| canvas.variant(kinds))
      .collect();
    let (device, layout) = (canvas.device.clone(), canvas.layout.clone());
    // One thread, the variant with none first: a preview mostly draws with
    // it, and the one with every kind takes many times as long to compile.
    let _ = std::thread::Builder::new()
      .name("canvas-shader-warm".to_owned())
      .spawn(move || {
        for variant in variants {
          variant.pipeline(&device, &layout, Target::Canvas);
        }
      });
  }
}

fn pipeline(
  device: &wgpu::Device,
  layout: &wgpu::PipelineLayout,
  module: &wgpu::ShaderModule,
  target: Target,
) -> wgpu::RenderPipeline {
  let format = match target {
    Target::Canvas | Target::Layer => FORMAT,
    Target::Delta => LAYER_FORMAT,
  };
  // A layer's canvas is premultiplied, so it lays over what is under it as
  // `source + destination * (1 - source alpha)`.
  let over = wgpu::BlendComponent {
    src_factor: wgpu::BlendFactor::One,
    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
    operation: wgpu::BlendOperation::Add,
  };
  device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
    label: Some("Screenwide preview pipeline"),
    layout: Some(layout),
    vertex: wgpu::VertexState {
      module,
      entry_point: Some("vs_main"),
      compilation_options: Default::default(),
      buffers: &[],
    },
    fragment: Some(wgpu::FragmentState {
      module,
      entry_point: Some("fs_main"),
      compilation_options: Default::default(),
      targets: &[Some(wgpu::ColorTargetState {
        format,
        blend: matches!(target, Target::Layer).then_some(wgpu::BlendState {
          color: over,
          alpha: over,
        }),
        write_mask: wgpu::ColorWrites::ALL,
      })],
    }),
    primitive: Default::default(),
    depth_stencil: None,
    multisample: Default::default(),
    multiview_mask: None,
    cache: None,
  })
}
