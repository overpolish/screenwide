// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Compositor {
  pub(in crate::editor::preview_platform::surface_windows) fn new(
    gpu: &'static Gpu,
  ) -> Result<Self, String> {
    let device = &gpu.device;
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some("Screenwide preview shader"),
      source: wgpu::ShaderSource::Wgsl(SHADER.into()),
    });
    let layout = bind_group_layout(device);
    let pipeline = canvas_pipeline(device, &layout, &module, None);
    // A layer's canvas is premultiplied, so it lays over what is under it as
    // `source + destination * (1 - source alpha)`.
    let over = wgpu::BlendComponent {
      src_factor: wgpu::BlendFactor::One,
      dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
      operation: wgpu::BlendOperation::Add,
    };
    let layer_pipeline = canvas_pipeline(
      device,
      &layout,
      &module,
      Some(wgpu::BlendState {
        color: over,
        alpha: over,
      }),
    );
    let uniform = |label, size: usize| {
      device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: size as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      })
    };
    let cursors = [
      (IDC_ARROW, "Arrow", "aero_arrow.cur"),
      (IDC_IBEAM, "IBeam", "beam_r.cur"),
      (IDC_IBEAM, "IBeam", "beam_r.cur"),
      (IDC_SIZEWE, "SizeWE", "aero_ew.cur"),
      (IDC_SIZENS, "SizeNS", "aero_ns.cur"),
      (IDC_HAND, "Hand", "aero_link.cur"),
      (IDC_CROSS, "Crosshair", "cross_r.cur"),
      (IDC_NO, "No", "aero_unavail.cur"),
    ];
    let cursor_pixels = cursors
      .into_iter()
      .map(|(cursor, scheme_value, fallback_file)| {
        native_cursor_pixels(cursor, scheme_value, fallback_file)
      })
      .collect::<Result<Vec<_>, _>>()?;
    let (cursor_width, cursor_height, _, _) = cursor_pixels[0];
    if cursor_pixels
      .iter()
      .any(|(width, height, _, _)| (*width, *height) != (cursor_width, cursor_height))
    {
      return Err("Windows standard cursors do not share one bitmap size".to_owned());
    }
    let layers: Vec<u8> = cursor_pixels
      .iter()
      .flat_map(|(_, _, _, pixels)| pixels.iter().copied())
      .collect();
    let cursor_view = gpu
      .texture_with_pixels(
        "Screenwide cursor atlas",
        (cursor_width, cursor_height, cursor_pixels.len() as u32),
        FORMAT,
        &layers,
      )
      .create_view(&wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::D2Array),
        ..Default::default()
      });
    let cursor_hotspots = std::array::from_fn(|index| {
      let hotspot = cursor_pixels[index].2;
      [hotspot[0], hotspot[1], 0.0, 0.0]
    });
    let fallback_view = gpu
      .texture_with_pixels("Screenwide preview fallback", (1, 1, 1), FORMAT, &[0; 4])
      .create_view(&Default::default());
    Ok(Self {
      gpu,
      background_cache: BackgroundImageCache::default(),
      annotations: GpuBuffer::new(gpu, "Screenwide annotations"),
      samples: GpuBuffer::new(gpu, "Screenwide annotation exposure"),
      annotation_points: GpuBuffer::new(gpu, "Screenwide annotation points"),
      annotation_text: GpuBuffer::new(gpu, "Screenwide annotation text"),
      constants: uniform("Screenwide canvas", size_of::<Constants>()),
      keyboard_constants: uniform("Screenwide keyboard", size_of::<KeyboardConstants>()),
      cursor_hotspots,
      cursor_view,
      counter_atlas: CounterAtlas::default(),
      keyboard_cache: KeyboardArtworkCache::default(),
      fallback_view,
      layout,
      pipeline,
      layer_pipeline,
      redactor: redact::Redactor::new(gpu),
      sampler: sampler(device, wgpu::FilterMode::Linear),
      point_sampler: sampler(device, wgpu::FilterMode::Nearest),
    })
  }
}

/// The bindings `preview.wgsl` and `annotations.wgsl` declare.
fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
  let entry = |binding, ty| wgpu::BindGroupLayoutEntry {
    binding,
    visibility: wgpu::ShaderStages::FRAGMENT,
    ty,
    count: None,
  };
  let uniform = wgpu::BindingType::Buffer {
    ty: wgpu::BufferBindingType::Uniform,
    has_dynamic_offset: false,
    min_binding_size: None,
  };
  let storage = wgpu::BindingType::Buffer {
    ty: wgpu::BufferBindingType::Storage { read_only: true },
    has_dynamic_offset: false,
    min_binding_size: None,
  };
  let texture = |dimension| wgpu::BindingType::Texture {
    sample_type: wgpu::TextureSampleType::Float { filterable: true },
    view_dimension: dimension,
    multisampled: false,
  };
  let plane = texture(wgpu::TextureViewDimension::D2);
  let sampler = wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering);
  device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
    label: Some("Screenwide preview bindings"),
    entries: &[
      entry(0, uniform),
      entry(1, uniform),
      entry(2, plane),
      entry(3, texture(wgpu::TextureViewDimension::D2Array)),
      entry(4, plane),
      entry(5, plane),
      entry(6, plane),
      entry(7, storage),
      entry(8, storage),
      entry(9, plane),
      entry(10, storage),
      entry(11, storage),
      entry(12, sampler),
      entry(13, sampler),
    ],
  })
}

fn canvas_pipeline(
  device: &wgpu::Device,
  layout: &wgpu::BindGroupLayout,
  module: &wgpu::ShaderModule,
  blend: Option<wgpu::BlendState>,
) -> wgpu::RenderPipeline {
  let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
    label: Some("Screenwide preview layout"),
    bind_group_layouts: &[Some(layout)],
    immediate_size: 0,
  });
  device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
    label: Some("Screenwide preview pipeline"),
    layout: Some(&pipeline_layout),
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
}

pub(super) fn sampler(device: &wgpu::Device, filter: wgpu::FilterMode) -> wgpu::Sampler {
  device.create_sampler(&wgpu::SamplerDescriptor {
    label: Some("Screenwide preview sampler"),
    mag_filter: filter,
    min_filter: filter,
    mipmap_filter: match filter {
      wgpu::FilterMode::Linear => wgpu::MipmapFilterMode::Linear,
      wgpu::FilterMode::Nearest => wgpu::MipmapFilterMode::Nearest,
    },
    ..Default::default()
  })
}
