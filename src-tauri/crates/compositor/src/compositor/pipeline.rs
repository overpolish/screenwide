// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Compositor {
  /// A compositor drawing the cursor from the platform's artwork.
  pub fn new(gpu: &std::sync::Arc<Gpu>) -> Result<Self, String> {
    Self::with_cursor_artworks(gpu, screenwide_cursor_effects::gpu_artworks())
  }

  /// A compositor drawing the cursor from `artworks`, one per style index.
  pub fn with_cursor_artworks(
    gpu: &std::sync::Arc<Gpu>,
    artworks: &[screenwide_cursor_effects::GpuArtwork],
  ) -> Result<Self, String> {
    let device = &gpu.device;
    let (layout, canvas) = super::canvas_pipelines::shared(gpu);
    let uniform = |label, size: usize| {
      device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: size as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      })
    };
    let (cursor_view, cursor_artworks) = super::cursor_artwork::upload(gpu, artworks)?;
    let fallback_view = gpu
      .texture_with_pixels("Screenwide preview fallback", (1, 1, 1), FORMAT, &[0; 4])
      .create_view(&Default::default());
    Ok(Self {
      gpu: std::sync::Arc::clone(gpu),
      background_cache: BackgroundImageCache::default(),
      annotations: GpuBuffer::new(gpu, "Screenwide annotations"),
      samples: GpuBuffer::new(gpu, "Screenwide annotation exposure"),
      annotation_points: GpuBuffer::new(gpu, "Screenwide annotation points"),
      annotation_text: GpuBuffer::new(gpu, "Screenwide annotation text"),
      constants: uniform("Screenwide canvas", size_of::<Constants>()),
      keyboard_constants: uniform("Screenwide keyboard", size_of::<KeyboardConstants>()),
      cursor_view,
      cursor_artworks,
      counter_atlas: CounterAtlas::default(),
      image_atlas: ImageAtlas::default(),
      keyboard_cache: KeyboardArtworkCache::default(),
      fallback_view,
      layout,
      tiles: super::tiles::AnnotationTiles::new(gpu),
      canvas,
      every_annotation_kind: std::sync::atomic::AtomicBool::new(false),
      redactor: redact::Redactor::new(gpu),
      camera_canvases: std::sync::Mutex::new(std::collections::HashMap::new()),
      mark_blur: super::mark_blur::MarkBlur::new(gpu),
      mips: screenwide_gpu::mips::Mips::new(&gpu.device, FORMAT),
      focus: super::focus::FocusBlur::new(gpu),
      lens_ground: super::lens_ground::LensGround::new(device),
      lens_under: Default::default(),
      sampler: sampler(device, wgpu::FilterMode::Linear),
      point_sampler: sampler(device, wgpu::FilterMode::Nearest),
    })
  }
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

/// The bind group entry `binding` reading `view`.
pub(super) fn texture(binding: u32, view: &wgpu::TextureView) -> wgpu::BindGroupEntry<'_> {
  wgpu::BindGroupEntry {
    binding,
    resource: wgpu::BindingResource::TextureView(view),
  }
}

/// The bind group entry `binding` reading all of `buffer`.
pub(super) fn buffer(binding: u32, buffer: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
  wgpu::BindGroupEntry {
    binding,
    resource: buffer.as_entire_binding(),
  }
}
