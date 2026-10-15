// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The focus blur's GPU objects: the bindings and pipelines that make the
//! chain's levels and gather from them, the chain itself, and the one
//! triangle every pass draws.

use super::super::FORMAT;
use super::plan::FocusConstants;
use screenwide_gpu::Gpu;

const GATHER_SHADER: &str = include_str!(concat!(env!("OUT_DIR"), "/focus_blur.wgsl"));
const COPY_SHADER: &str = include_str!(concat!(env!("OUT_DIR"), "/focus_copy.wgsl"));

/// The bindings a level is made with, a level above and a sampler, and the
/// gather's, which adds its constants and its boxes.
pub(super) fn layouts(device: &wgpu::Device) -> [wgpu::BindGroupLayout; 2] {
  let entry = |binding, ty| wgpu::BindGroupLayoutEntry {
    binding,
    visibility: wgpu::ShaderStages::FRAGMENT,
    ty,
    count: None,
  };
  let texture = entry(
    0,
    wgpu::BindingType::Texture {
      sample_type: wgpu::TextureSampleType::Float { filterable: true },
      view_dimension: wgpu::TextureViewDimension::D2,
      multisampled: false,
    },
  );
  let sampler = entry(
    1,
    wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
  );
  [
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("Screenwide focus level bindings"),
      entries: &[texture, sampler],
    }),
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("Screenwide focus gather bindings"),
      entries: &[
        texture,
        sampler,
        entry(
          2,
          wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: wgpu::BufferSize::new(size_of::<FocusConstants>() as u64),
          },
        ),
        entry(
          3,
          wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
          },
        ),
      ],
    }),
  ]
}

/// The pass that makes a level from the one above it, then the gather onto a
/// whole canvas and over a layer.
pub(super) fn pipelines(
  device: &wgpu::Device,
  [level, gather]: &[wgpu::BindGroupLayout; 2],
) -> (wgpu::RenderPipeline, [wgpu::RenderPipeline; 2]) {
  let module = |source: &str, label: &str| {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some(label),
      source: wgpu::ShaderSource::Wgsl(source.into()),
    })
  };
  let copy = module(COPY_SHADER, "Screenwide focus level shader");
  let blur = module(GATHER_SHADER, "Screenwide focus blur shader");
  // A layer's frame is premultiplied, so it lays over what is under it as
  // `source + destination * (1 - source alpha)`, as the canvas pass's does.
  let over = wgpu::BlendComponent {
    src_factor: wgpu::BlendFactor::One,
    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
    operation: wgpu::BlendOperation::Add,
  };
  let pipeline = |module: &wgpu::ShaderModule,
                  entry: &str,
                  layout: &wgpu::BindGroupLayout,
                  blend: Option<wgpu::BlendState>| {
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("Screenwide focus layout"),
      bind_group_layouts: &[Some(layout)],
      immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("Screenwide focus pipeline"),
      layout: Some(&layout),
      vertex: wgpu::VertexState {
        module,
        entry_point: Some("vs_main"),
        compilation_options: Default::default(),
        buffers: &[],
      },
      fragment: Some(wgpu::FragmentState {
        module,
        entry_point: Some(entry),
        compilation_options: Default::default(),
        targets: &[Some(wgpu::ColorTargetState {
          format: FORMAT,
          blend,
          write_mask: wgpu::ColorWrites::ALL,
        })],
      }),
      primitive: Default::default(),
      depth_stencil: None,
      multisample: Default::default(),
      multiview_mask: None,
      cache: None,
    })
  };
  (
    pipeline(&copy, "fs_downsample", level, None),
    [
      pipeline(&blur, "fs_main", gather, None),
      pipeline(
        &blur,
        "fs_main",
        gather,
        Some(wgpu::BlendState {
          color: over,
          alpha: over,
        }),
      ),
    ],
  )
}

/// One slot's chain of levels, the size of the target.
pub(super) struct Chain {
  pub(super) size: (u32, u32),
  pub(super) texture: wgpu::Texture,
  /// Every level, which the gather reads.
  pub(super) view: wgpu::TextureView,
}

/// A chain `size` texels at its first level, down to the levels the widest
/// blur reads.
pub(super) fn chain(gpu: &Gpu, size: (u32, u32)) -> Chain {
  let size = (size.0.max(1), size.1.max(1));
  let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
    label: Some("Screenwide focus chain"),
    size: wgpu::Extent3d {
      width: size.0,
      height: size.1,
      depth_or_array_layers: 1,
    },
    mip_level_count: super::super::mips::levels(size),
    sample_count: 1,
    dimension: wgpu::TextureDimension::D2,
    format: FORMAT,
    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
    view_formats: &[],
  });
  Chain {
    size,
    view: texture.create_view(&Default::default()),
    texture,
  }
}

/// One level of `texture`, which a pass draws into or reads.
pub(super) fn level(texture: &wgpu::Texture, at: u32) -> wgpu::TextureView {
  texture.create_view(&wgpu::TextureViewDescriptor {
    base_mip_level: at,
    mip_level_count: Some(1),
    ..Default::default()
  })
}

/// A pass's viewport size and scissor rectangle, as the canvas pass sets them.
pub(super) type Viewport = ((u32, u32), Option<[u32; 4]>);

/// The one triangle over `view`, through `pipeline` with `bindings`, within
/// `viewport` where it is given.
pub(super) fn draw(
  encoder: &mut wgpu::CommandEncoder,
  view: &wgpu::TextureView,
  load: wgpu::LoadOp<wgpu::Color>,
  (pipeline, bindings): (&wgpu::RenderPipeline, &wgpu::BindGroup),
  viewport: Option<Viewport>,
) {
  let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
    label: Some("Screenwide focus pass"),
    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
      view,
      depth_slice: None,
      resolve_target: None,
      ops: wgpu::Operations {
        load,
        store: wgpu::StoreOp::Store,
      },
    })],
    ..Default::default()
  });
  if let Some(((width, height), scissor)) = viewport {
    pass.set_viewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
    if let Some([x, y, width, height]) = scissor {
      pass.set_scissor_rect(x, y, width, height);
    }
  }
  pass.set_pipeline(pipeline);
  pass.set_bind_group(0, bindings, &[]);
  pass.draw(0..3, 0..1);
}
