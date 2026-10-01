// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The shared OSC shader's wgpu pipeline, which every Windows OSC surface
//! draws its vertices through: the region OSC and the editor's selection.

use super::{RenderConstants, Vertex};

const SHADER: &str = include_str!(concat!(env!("OUT_DIR"), "/osc.wgsl"));

use crate::app_windows::overlay_surface::FORMAT;

pub(crate) fn shader_module(device: &wgpu::Device) -> wgpu::ShaderModule {
  device.create_shader_module(wgpu::ShaderModuleDescriptor {
    label: Some("Screenwide OSC shader"),
    source: wgpu::ShaderSource::Wgsl(SHADER.into()),
  })
}

/// The constants block, read at a dynamic offset one `RenderConstants` wide,
/// then the label, secondary label, icon atlas, frozen desktop and magnifier
/// source textures, then the linear and point samplers.
pub(crate) fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
  let texture = |binding| wgpu::BindGroupLayoutEntry {
    binding,
    visibility: wgpu::ShaderStages::FRAGMENT,
    ty: wgpu::BindingType::Texture {
      sample_type: wgpu::TextureSampleType::Float { filterable: true },
      view_dimension: wgpu::TextureViewDimension::D2,
      multisampled: false,
    },
    count: None,
  };
  let sampler = |binding| wgpu::BindGroupLayoutEntry {
    binding,
    visibility: wgpu::ShaderStages::FRAGMENT,
    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
    count: None,
  };
  device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
    label: Some("Screenwide OSC bindings"),
    entries: &[
      wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
          ty: wgpu::BufferBindingType::Uniform,
          has_dynamic_offset: true,
          min_binding_size: wgpu::BufferSize::new(size_of::<RenderConstants>() as u64),
        },
        count: None,
      },
      texture(1),
      texture(2),
      texture(3),
      texture(4),
      texture(5),
      sampler(6),
      sampler(7),
    ],
  })
}

/// Straight source-over on colour. `source_alpha` is the alpha channel's
/// source factor: `SrcAlpha` normally, `One` over the opaque frozen desktop.
pub(crate) fn pipeline(
  device: &wgpu::Device,
  layout: &wgpu::BindGroupLayout,
  module: &wgpu::ShaderModule,
  source_alpha: wgpu::BlendFactor,
) -> wgpu::RenderPipeline {
  let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
    label: Some("Screenwide OSC layout"),
    bind_group_layouts: &[Some(layout)],
    immediate_size: 0,
  });
  device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
    label: Some("Screenwide OSC pipeline"),
    layout: Some(&pipeline_layout),
    vertex: wgpu::VertexState {
      module,
      entry_point: Some("vs_main"),
      compilation_options: Default::default(),
      buffers: &[Some(wgpu::VertexBufferLayout {
        array_stride: size_of::<Vertex>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![
          0 => Float32x2,
          1 => Float32x2,
          2 => Float32x2,
          3 => Uint32,
        ],
      })],
    },
    fragment: Some(wgpu::FragmentState {
      module,
      entry_point: Some("fs_main"),
      compilation_options: Default::default(),
      targets: &[Some(wgpu::ColorTargetState {
        format: FORMAT,
        blend: Some(wgpu::BlendState {
          color: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::SrcAlpha,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation: wgpu::BlendOperation::Add,
          },
          alpha: wgpu::BlendComponent {
            src_factor: source_alpha,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation: wgpu::BlendOperation::Add,
          },
        }),
        write_mask: wgpu::ColorWrites::ALL,
      })],
    }),
    // Metal's default is no face culling. Several shared OSC primitives are
    // deliberately emitted in either winding (notably line quads).
    primitive: wgpu::PrimitiveState {
      cull_mode: None,
      ..Default::default()
    },
    depth_stencil: None,
    multisample: Default::default(),
    multiview_mask: None,
    cache: None,
  })
}

pub(crate) fn sampler(device: &wgpu::Device, filter: wgpu::FilterMode) -> wgpu::Sampler {
  device.create_sampler(&wgpu::SamplerDescriptor {
    label: Some("Screenwide OSC sampler"),
    address_mode_u: wgpu::AddressMode::ClampToEdge,
    address_mode_v: wgpu::AddressMode::ClampToEdge,
    address_mode_w: wgpu::AddressMode::ClampToEdge,
    mag_filter: filter,
    min_filter: filter,
    mipmap_filter: wgpu::MipmapFilterMode::Nearest,
    ..Default::default()
  })
}
