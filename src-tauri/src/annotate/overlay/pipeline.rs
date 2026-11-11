// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

//! The overlay's pipeline and its bindings. Its shader comes compiled from
//! the build where the build compiled it (`build/precompiled_shaders.rs`):
//! compiled from WGSL on Windows it took about 1.5 seconds each time the
//! shared device opened, as nothing keeps DXC's output.

use super::overlay_bindings::{OverlayBinding, OVERLAY_BINDINGS};

const SHADER: &str = include_str!(concat!(env!("OUT_DIR"), "/annotate_overlay.wgsl"));

// `ANNOTATE_OVERLAY`: `None` where the build compiled nothing for this
// target.
include!(concat!(env!("OUT_DIR"), "/precompiled_shaders.rs"));

/// Whether the shader is taken from what the build compiled, where it did.
#[derive(Clone, Copy)]
pub(super) enum Source {
  Precompiled,
  /// What the precompiled shader is checked against.
  #[cfg(test)]
  Wgsl,
}

/// Whether `device` takes the shader the build compiled.
#[cfg(test)]
pub(super) fn precompiled_on(device: &wgpu::Device) -> bool {
  // SAFETY: the modules are only made, never drawn with.
  unsafe { screenwide_gpu::precompiled_modules(device, ANNOTATE_OVERLAY, "") }.is_some()
}

pub(super) fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
  let entries = OVERLAY_BINDINGS.map(|(binding, kind)| wgpu::BindGroupLayoutEntry {
    binding,
    visibility: wgpu::ShaderStages::FRAGMENT,
    ty: match kind {
      OverlayBinding::Uniform => wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Uniform,
        has_dynamic_offset: false,
        min_binding_size: None,
      },
      OverlayBinding::Storage => wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Storage { read_only: true },
        has_dynamic_offset: false,
        min_binding_size: None,
      },
      OverlayBinding::Texture => wgpu::BindingType::Texture {
        sample_type: wgpu::TextureSampleType::Float { filterable: true },
        view_dimension: wgpu::TextureViewDimension::D2,
        multisampled: false,
      },
      OverlayBinding::Sampler => wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
    },
    count: None,
  });
  device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
    label: Some("Screenwide annotate overlay bindings"),
    entries: &entries,
  })
}

pub(super) fn pipeline(
  device: &wgpu::Device,
  layout: &wgpu::BindGroupLayout,
  source: Source,
) -> wgpu::RenderPipeline {
  let precompiled = match source {
    // SAFETY: the build compiled these stages from this shader against
    // `OVERLAY_BINDINGS`, which `bind_group_layout` builds the layout from;
    // the overlay's tests draw with them and with the shader compiled from
    // WGSL and require the same pixels.
    Source::Precompiled => unsafe {
      screenwide_gpu::precompiled_modules(
        device,
        ANNOTATE_OVERLAY,
        "Screenwide precompiled annotate overlay shader",
      )
    },
    #[cfg(test)]
    Source::Wgsl => None,
  };
  let (vertex, fragment) = match precompiled {
    Some(modules) => (modules.vertex, modules.fragment),
    None => {
      let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Screenwide annotate overlay shader"),
        source: wgpu::ShaderSource::Wgsl(SHADER.into()),
      });
      (module.clone(), module)
    }
  };
  let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
    label: Some("Screenwide annotate overlay layout"),
    bind_group_layouts: &[Some(layout)],
    immediate_size: 0,
  });
  // The result is already premultiplied, so it is written rather than
  // blended: one pass over a cleared target has nothing to blend with.
  device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
    label: Some("Screenwide annotate overlay pipeline"),
    layout: Some(&pipeline_layout),
    vertex: wgpu::VertexState {
      module: &vertex,
      entry_point: Some("vs_main"),
      compilation_options: Default::default(),
      buffers: &[],
    },
    fragment: Some(wgpu::FragmentState {
      module: &fragment,
      entry_point: Some("fs_main"),
      compilation_options: Default::default(),
      targets: &[Some(wgpu::ColorTargetState {
        format: screenwide_gpu::surface::FORMAT,
        blend: None,
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
