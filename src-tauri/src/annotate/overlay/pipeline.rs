// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

//! The overlay's shader and pipeline. The shader comes compiled from the
//! build (`build/precompiled_shaders.rs`): compiled from WGSL on Windows it
//! took about 1.5 seconds each time the shared device opened, as nothing
//! keeps DXC's output.

use screenwide_gpu::{RenderStages, Shader};

use super::overlay_layout::ANNOTATE_OVERLAY;

const SHADER: &str = include_str!(concat!(env!("OUT_DIR"), "/annotate_overlay.wgsl"));

use crate::precompiled_shaders::ANNOTATE_OVERLAY as PRECOMPILED;

pub(super) fn shader(device: &wgpu::Device) -> Shader {
  Shader::new(
    device,
    "Screenwide annotate overlay",
    &ANNOTATE_OVERLAY,
    SHADER,
    PRECOMPILED,
  )
}

/// Whether the build compiled the shader for this platform.
#[cfg(test)]
pub(super) fn precompiled() -> bool {
  PRECOMPILED.is_some()
}

pub(super) fn pipeline(shader: &Shader) -> wgpu::RenderPipeline {
  // The result is already premultiplied, so it is written rather than
  // blended: one pass over a cleared target has nothing to blend with.
  shader.render_pipeline(&RenderStages {
    label: "Screenwide annotate overlay pipeline",
    vertex: "vs_main",
    fragment: "fs_main",
    targets: &[Some(wgpu::ColorTargetState {
      format: screenwide_gpu::surface::FORMAT,
      blend: None,
      write_mask: wgpu::ColorWrites::ALL,
    })],
    primitive: Default::default(),
  })
}
