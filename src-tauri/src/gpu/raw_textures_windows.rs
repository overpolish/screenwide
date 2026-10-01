// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Textures Direct3D 12 allocates for wgpu and a Direct3D 11 device to share.
//!
//! Each is allocated with simultaneous access, so it returns to the common
//! state at the end of every submission and neither API has to know the
//! other's barriers.

use windows::Win32::Graphics::{
  Direct3D12::{
    ID3D12Resource, D3D12_HEAP_FLAGS, D3D12_HEAP_PROPERTIES, D3D12_HEAP_TYPE_DEFAULT,
    D3D12_RESOURCE_DESC, D3D12_RESOURCE_DIMENSION_TEXTURE2D,
    D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET, D3D12_RESOURCE_FLAG_ALLOW_SIMULTANEOUS_ACCESS,
    D3D12_RESOURCE_STATE_COMMON, D3D12_TEXTURE_LAYOUT_UNKNOWN,
  },
  Dxgi::Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_SAMPLE_DESC},
};

use super::Gpu;

pub(super) type Dx12 = wgpu::hal::api::Dx12;

/// A render-target texture of `size` in `format` - `Bgra8Unorm` or
/// `Rgba8Unorm` - on wgpu's device, in a heap with `heap` flags.
pub(super) fn committed_texture(
  gpu: &Gpu,
  size: (u32, u32),
  format: wgpu::TextureFormat,
  heap: D3D12_HEAP_FLAGS,
  label: &str,
) -> Result<ID3D12Resource, String> {
  let format = match format {
    wgpu::TextureFormat::Bgra8Unorm => DXGI_FORMAT_B8G8R8A8_UNORM,
    wgpu::TextureFormat::Rgba8Unorm => DXGI_FORMAT_R8G8B8A8_UNORM,
    other => {
      return Err(format!(
        "{other:?} textures are not shared with Direct3D 11"
      ))
    }
  };
  let hal = unsafe { gpu.device.as_hal::<Dx12>() }
    .ok_or_else(|| "The graphics device is not Direct3D 12".to_owned())?;
  let description = D3D12_RESOURCE_DESC {
    Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
    Alignment: 0,
    Width: u64::from(size.0.max(1)),
    Height: size.1.max(1),
    DepthOrArraySize: 1,
    MipLevels: 1,
    Format: format,
    SampleDesc: DXGI_SAMPLE_DESC {
      Count: 1,
      Quality: 0,
    },
    Layout: D3D12_TEXTURE_LAYOUT_UNKNOWN,
    Flags: D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET | D3D12_RESOURCE_FLAG_ALLOW_SIMULTANEOUS_ACCESS,
  };
  let mut resource: Option<ID3D12Resource> = None;
  unsafe {
    hal.raw_device().CreateCommittedResource(
      &D3D12_HEAP_PROPERTIES {
        Type: D3D12_HEAP_TYPE_DEFAULT,
        ..Default::default()
      },
      heap,
      &description,
      D3D12_RESOURCE_STATE_COMMON,
      None,
      &mut resource,
    )
  }
  .map_err(|error| format!("The {label} could not be created: {error}"))?;
  resource.ok_or_else(|| format!("Direct3D 12 created no {label}"))
}

/// `resource` as a wgpu texture wgpu samples, renders into and copies.
pub(super) fn wgpu_texture(
  gpu: &Gpu,
  resource: ID3D12Resource,
  size: (u32, u32),
  format: wgpu::TextureFormat,
  label: &str,
) -> (wgpu::Texture, wgpu::TextureView) {
  let extent = wgpu::Extent3d {
    width: size.0.max(1),
    height: size.1.max(1),
    depth_or_array_layers: 1,
  };
  let hal = unsafe {
    wgpu::hal::dx12::Device::texture_from_raw(
      resource,
      format,
      wgpu::TextureDimension::D2,
      extent,
      1,
      1,
    )
  };
  let texture = unsafe {
    gpu.device.create_texture_from_hal::<Dx12>(
      hal,
      &wgpu::TextureDescriptor {
        label: Some(label),
        size: extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
          | wgpu::TextureUsages::RENDER_ATTACHMENT
          | wgpu::TextureUsages::COPY_SRC
          | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
      },
      wgpu::TextureUses::UNINITIALIZED,
    )
  };
  let view = texture.create_view(&Default::default());
  (texture, view)
}
