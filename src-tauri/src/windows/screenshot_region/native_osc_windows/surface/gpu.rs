// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Gpu {
  pub(crate) fn new() -> Result<Arc<Self>, String> {
    let shared = crate::gpu::shared()?;
    let device = &shared.device;
    let module = shader_module(device);
    let layout = bind_group_layout(device);
    let blended = pipeline(device, &layout, &module, wgpu::BlendFactor::SrcAlpha);
    let opaque = pipeline(device, &layout, &module, wgpu::BlendFactor::One);
    let placeholder = upload_rgba(shared, &[0_u8; 4], 1, 1);
    let icons = upload_icons(shared).unwrap_or_else(|error| {
      eprintln!("The Windows region OSC could not upload the icon atlas: {error}");
      placeholder.clone()
    });
    Ok(Arc::new(Self {
      shared,
      layout,
      pipeline: blended,
      opaque_pipeline: opaque,
      linear_sampler: sampler(device, wgpu::FilterMode::Linear),
      point_sampler: sampler(device, wgpu::FilterMode::Nearest),
      placeholder,
      icons,
    }))
  }

  /// One draw call's bindings. `constants` is read at a dynamic offset, one
  /// `RenderConstants` block wide.
  pub(super) fn bindings(
    &self,
    constants: &wgpu::Buffer,
    label: &wgpu::TextureView,
    secondary: &wgpu::TextureView,
    snapshot: &wgpu::TextureView,
    magnifier: &wgpu::TextureView,
  ) -> wgpu::BindGroup {
    let view = |binding, view| wgpu::BindGroupEntry {
      binding,
      resource: wgpu::BindingResource::TextureView(view),
    };
    self
      .shared
      .device
      .create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Screenwide region OSC draw"),
        layout: &self.layout,
        entries: &[
          wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
              buffer: constants,
              offset: 0,
              size: wgpu::BufferSize::new(size_of::<RenderConstants>() as u64),
            }),
          },
          view(1, label),
          view(2, secondary),
          view(3, &self.icons),
          view(4, snapshot),
          view(5, magnifier),
          wgpu::BindGroupEntry {
            binding: 6,
            resource: wgpu::BindingResource::Sampler(&self.linear_sampler),
          },
          wgpu::BindGroupEntry {
            binding: 7,
            resource: wgpu::BindingResource::Sampler(&self.point_sampler),
          },
        ],
      })
  }
}
