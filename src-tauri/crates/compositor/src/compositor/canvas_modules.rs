// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A canvas variant's shader, from what the build compiled or from WGSL, and
//! the render pipeline built from it for each target.

use screenwide_gpu::{RenderStages, Shader};

use super::canvas_features::{self, FeatureMask, ALL_FEATURES};
use super::canvas_pipelines::Target;
use super::canvas_variants::{every_kind, source, KindMask};
use super::{mark_blur::LAYER_FORMAT, FORMAT};
use crate::precompiled_shaders::{EVERY_KIND, NO_ANNOTATIONS};

/// The shader drawing `kinds` keeping `features`: the build's where it
/// compiled it and `precompiled` allows, otherwise compiled from WGSL. The
/// build compiles the variant with no annotation and the one with every
/// kind, both keeping every feature.
pub(super) fn shader(
  device: &wgpu::Device,
  kinds: KindMask,
  features: FeatureMask,
  precompiled: bool,
) -> Shader {
  let built = match kinds {
    _ if !precompiled || features != ALL_FEATURES => None,
    0 => NO_ANNOTATIONS,
    kinds if kinds == every_kind() => EVERY_KIND,
    _ => None,
  };
  Shader::new(
    device,
    "Screenwide preview",
    &crate::shader_layouts::PREVIEW,
    &canvas_features::keeping(source(kinds), features),
    built,
  )
}

pub(super) fn pipeline(shader: &Shader, target: Target) -> wgpu::RenderPipeline {
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
  shader.render_pipeline(&RenderStages {
    label: "Screenwide preview pipeline",
    vertex: "vs_main",
    fragment: "fs_main",
    targets: &[Some(wgpu::ColorTargetState {
      format,
      blend: matches!(target, Target::Layer).then_some(wgpu::BlendState {
        color: over,
        alpha: over,
      }),
      write_mask: wgpu::ColorWrites::ALL,
    })],
    primitive: Default::default(),
  })
}
