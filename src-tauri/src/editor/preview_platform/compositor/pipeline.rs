// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Compositor {
  pub(crate) fn new(gpu: &'static Gpu, cursors: NativeCursors) -> Result<Self, String> {
    let device = &gpu.device;
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some("Screenwide preview shader"),
      source: wgpu::ShaderSource::Wgsl(SHADER.into()),
    });
    let layout = bind_group_layout(device);
    let canvas = super::canvas_pipelines::CanvasPipelines::new(device, &layout, &module);
    let uniform = |label, size: usize| {
      device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: size as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      })
    };
    let layer_bytes = cursors.size.0 as usize * cursors.size.1 as usize * 4;
    if layer_bytes == 0
      || cursors.layers.is_empty()
      || !cursors.layers.len().is_multiple_of(layer_bytes)
    {
      return Err("The cursor artwork does not fill its layers".to_owned());
    }
    let cursor_view = gpu
      .texture_with_pixels(
        "Screenwide cursor atlas",
        (
          cursors.size.0,
          cursors.size.1,
          (cursors.layers.len() / layer_bytes) as u32,
        ),
        FORMAT,
        &cursors.layers,
      )
      .create_view(&wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::D2Array),
        ..Default::default()
      });
    let cursor_hotspots = cursors.hotspots;
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
      cursor_artworks: Vec::new(),
      #[cfg(target_os = "macos")]
      cursor_artwork_key: 0,
      counter_atlas: CounterAtlas::default(),
      keyboard_cache: KeyboardArtworkCache::default(),
      fallback_view,
      layout,
      tiles: super::tiles::AnnotationTiles::new(gpu, &module),
      canvas,
      redactor: redact::Redactor::new(gpu),
      mark_blur: super::mark_blur::MarkBlur::new(gpu),
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
      entry(14, storage),
      entry(15, plane),
      entry(16, plane),
    ],
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
