// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::preview_bindings::{PreviewBinding, PREVIEW_BINDINGS};
use super::*;

impl Compositor {
  pub(crate) fn new(gpu: &'static Gpu, cursors: NativeCursors) -> Result<Self, String> {
    let device = &gpu.device;
    // The tile binning reads the full module: it compiles only its own entry
    // point, which needs every kind's reach. The canvas pass compiles a
    // variant per set of kinds instead, when one is first asked for.
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some("Screenwide preview shader"),
      source: wgpu::ShaderSource::Wgsl(SHADER.into()),
    });
    let (layout, canvas) = super::canvas_pipelines::shared(gpu, || bind_group_layout(device));
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
      sticker_atlas: StickerAtlas::default(),
      keyboard_cache: KeyboardArtworkCache::default(),
      fallback_view,
      layout,
      tiles: super::tiles::AnnotationTiles::new(gpu, &module),
      canvas,
      redactor: redact::Redactor::new(gpu),
      camera_canvases: std::sync::Mutex::new(std::collections::HashMap::new()),
      mark_blur: super::mark_blur::MarkBlur::new(gpu),
      sampler: sampler(device, wgpu::FilterMode::Linear),
      point_sampler: sampler(device, wgpu::FilterMode::Nearest),
    })
  }
}

/// The bindings `preview.wgsl` and `annotations.wgsl` declare, as
/// [`PREVIEW_BINDINGS`] lists them.
fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
  let buffer = |ty| wgpu::BindingType::Buffer {
    ty,
    has_dynamic_offset: false,
    min_binding_size: None,
  };
  let texture = |view_dimension| wgpu::BindingType::Texture {
    sample_type: wgpu::TextureSampleType::Float { filterable: true },
    view_dimension,
    multisampled: false,
  };
  let entries: Vec<_> = PREVIEW_BINDINGS
    .iter()
    .zip(0..)
    .map(|(binding, index)| wgpu::BindGroupLayoutEntry {
      binding: index,
      visibility: wgpu::ShaderStages::FRAGMENT,
      ty: match binding {
        PreviewBinding::Uniform => buffer(wgpu::BufferBindingType::Uniform),
        PreviewBinding::Storage => buffer(wgpu::BufferBindingType::Storage { read_only: true }),
        PreviewBinding::Texture => texture(wgpu::TextureViewDimension::D2),
        PreviewBinding::TextureArray => texture(wgpu::TextureViewDimension::D2Array),
        PreviewBinding::Sampler => wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
      },
      count: None,
    })
    .collect();
  device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
    label: Some("Screenwide preview bindings"),
    entries: &entries,
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
