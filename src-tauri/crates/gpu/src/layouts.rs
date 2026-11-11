// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! wgpu's bind group layouts and vertex attributes made from a
//! [`screenwide_shader_layout::ShaderLayout`]'s description of them.

use screenwide_shader_layout::{
  Binding, Entry, Sample, Stages, StorageFormat, VertexBuffer, VertexFormat,
};

/// The bind group layout `entries` describe, for bind groups made apart from
/// the [`crate::Shader`] whose pipelines read them.
pub fn bind_group_layout(
  device: &wgpu::Device,
  label: &str,
  entries: &[Entry],
) -> wgpu::BindGroupLayout {
  let entries = entries.iter().map(layout_entry).collect::<Vec<_>>();
  device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
    label: Some(label),
    entries: &entries,
  })
}

/// The attributes `buffer` holds.
pub(crate) fn vertex_attributes(buffer: &VertexBuffer) -> Vec<wgpu::VertexAttribute> {
  buffer
    .attributes
    .iter()
    .map(|attribute| wgpu::VertexAttribute {
      format: match attribute.format {
        VertexFormat::Float32x2 => wgpu::VertexFormat::Float32x2,
        VertexFormat::Uint32 => wgpu::VertexFormat::Uint32,
      },
      offset: attribute.offset,
      shader_location: attribute.location,
    })
    .collect()
}

fn layout_entry(entry: &Entry) -> wgpu::BindGroupLayoutEntry {
  let buffer = |ty, has_dynamic_offset| wgpu::BindingType::Buffer {
    ty,
    has_dynamic_offset,
    min_binding_size: None,
  };
  wgpu::BindGroupLayoutEntry {
    binding: entry.binding,
    visibility: [
      (Stages::VERTEX, wgpu::ShaderStages::VERTEX),
      (Stages::FRAGMENT, wgpu::ShaderStages::FRAGMENT),
      (Stages::COMPUTE, wgpu::ShaderStages::COMPUTE),
    ]
    .into_iter()
    .filter(|(stage, _)| entry.stages.contains(*stage))
    .fold(wgpu::ShaderStages::NONE, |stages, (_, stage)| {
      stages | stage
    }),
    ty: match entry.kind {
      Binding::Uniform => buffer(wgpu::BufferBindingType::Uniform, false),
      Binding::DynamicUniform => buffer(wgpu::BufferBindingType::Uniform, true),
      Binding::Storage { writable } => buffer(
        wgpu::BufferBindingType::Storage {
          read_only: !writable,
        },
        false,
      ),
      Binding::Texture { sample, array } => wgpu::BindingType::Texture {
        sample_type: wgpu::TextureSampleType::Float {
          filterable: sample == Sample::Float,
        },
        view_dimension: if array {
          wgpu::TextureViewDimension::D2Array
        } else {
          wgpu::TextureViewDimension::D2
        },
        multisampled: false,
      },
      Binding::StorageTexture { format } => wgpu::BindingType::StorageTexture {
        access: wgpu::StorageTextureAccess::WriteOnly,
        format: match format {
          StorageFormat::Rgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
        },
        view_dimension: wgpu::TextureViewDimension::D2,
      },
      Binding::Sampler { filtering } => wgpu::BindingType::Sampler(if filtering {
        wgpu::SamplerBindingType::Filtering
      } else {
        wgpu::SamplerBindingType::NonFiltering
      }),
    },
    count: None,
  }
}
