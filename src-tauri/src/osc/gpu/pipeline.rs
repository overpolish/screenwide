// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The shared OSC shader's wgpu pipeline, which every OSC surface draws its
//! vertices through on both platforms: the region OSC, its ruler and OCR
//! chrome, and the editor's selection.

use crate::gpu::surface::FORMAT;
use crate::osc::style::{control_palette, ocr_palette, overlay_palette, ruler_palette};

const SHADER: &str = include_str!(concat!(env!("OUT_DIR"), "/osc.wgsl"));

/// One triangle-list vertex. `position` is already in NDC: the pixel-to-clip
/// mapping happens on the CPU so the vertex shader stays a pass-through. The
/// twin of `ScreenwideRegionOscVertex`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Vertex {
  pub position: [f32; 2],
  pub uv: [f32; 2],
  pub kind: u32,
  pub padding: u32,
}

const _: () = assert!(std::mem::size_of::<Vertex>() == 24);

/// The twin of `OscGpu` in `osc.wgsl`: every member is a 16-byte row, so the
/// uniform layout needs no padding.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct RenderConstants {
  pub light_mode: [u32; 4],
  pub magnifier_box: [f32; 4],
  pub action_fills: [[f32; 4]; 2],
  pub control_colors: [[f32; 4]; 2],
  pub ocr_colors: [[f32; 4]; 8],
  pub overlay_shade: [f32; 4],
  pub ruler_colors: [[f32; 4]; 2],
  pub ruler_sample: [f32; 4],
  pub ruler_animation: [f32; 4],
  pub magnifier_source: [f32; 4],
  pub magnifier_sample: [f32; 4],
  pub magnifier_source_range: [f32; 4],
  pub magnifier_flags: [u32; 4],
  /// The rounded chrome plate Windows draws itself: `.x` is its corner
  /// radius. macOS hands that radius to the material surface instead.
  pub chrome: [f32; 4],
  pub chrome_outline: [f32; 4],
  /// Physical viewport size and source texel size for the Windows material
  /// blur sampled from the already-resident frozen desktop texture.
  pub chrome_backdrop: [f32; 4],
  /// The snapshot UV window after Ruler pan/zoom.
  pub chrome_source: [f32; 4],
}

const _: () = assert!(std::mem::size_of::<RenderConstants>().is_multiple_of(16));

impl RenderConstants {
  /// The palettes come from the platform-neutral tokens; action fills stay
  /// zero because the OCR controls push their own pair per draw.
  pub(crate) fn new(light_mode: bool) -> Self {
    let controls = control_palette(light_mode);
    let ocr = ocr_palette(light_mode);
    let ruler = ruler_palette(light_mode);
    Self {
      light_mode: [u32::from(light_mode), 0, 0, 0],
      magnifier_box: [0.0; 4],
      action_fills: [[0.0; 4]; 2],
      control_colors: [controls.fill, controls.outline],
      ocr_colors: [
        ocr.primary_fill,
        ocr.primary_outline,
        ocr.qr_fill,
        ocr.qr_outline,
        ocr.error_fill,
        ocr.error_outline,
        ocr.selection_fill,
        ocr.selection_outline,
      ],
      overlay_shade: overlay_palette().shade,
      ruler_colors: [ruler.primary, ruler.info],
      ruler_sample: [0.0; 4],
      ruler_animation: [0.0; 4],
      magnifier_source: [0.0; 4],
      magnifier_sample: [0.0; 4],
      magnifier_source_range: [0.0, 0.0, 1.0, 1.0],
      magnifier_flags: [0; 4],
      chrome: [0.0; 4],
      chrome_outline: [0.0; 4],
      chrome_backdrop: [0.0; 4],
      chrome_source: [0.0, 0.0, 1.0, 1.0],
    }
  }
}

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

/// Straight source-over: colour is premultiplied through its own alpha, and
/// the alpha channel is laid over the destination's as it is, so the target
/// holds premultiplied colour for the compositor to blend.
pub(crate) fn pipeline(
  device: &wgpu::Device,
  layout: &wgpu::BindGroupLayout,
  module: &wgpu::ShaderModule,
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
          2 => Uint32,
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
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation: wgpu::BlendOperation::Add,
          },
        }),
        write_mask: wgpu::ColorWrites::ALL,
      })],
    }),
    // Several OSC primitives are deliberately emitted in either winding
    // (notably line quads), so nothing is culled.
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
