// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Direct3D 11 interface Media Foundation requires, layered over wgpu's
//! own Direct3D 12 device and queue.
//!
//! Media Foundation only accepts a Direct3D 11 device. D3D11On12 gives it one
//! without a second device: its work is recorded onto wgpu's device and
//! submitted to wgpu's queue, so a frame never crosses adapters and the two
//! sides need no fences. Work submitted by one runs before work the other
//! submits afterwards; a texture both use is handed over by submission order
//! alone.
//!
//! Screen capture cannot use the layer: Windows.Graphics.Capture never gets
//! its frame buffers back from a D3D11On12 device and stops after its first
//! few frames. Recording captures on a device of its own; see `bridge_windows`.

use std::sync::{LazyLock, Mutex};

use windows::{
  core::Interface,
  Win32::Graphics::{
    Direct3D::{D3D_FEATURE_LEVEL_11_0, D3D_FEATURE_LEVEL_11_1},
    Direct3D10::ID3D10Multithread,
    Direct3D11::{
      ID3D11Device, ID3D11DeviceContext, ID3D11Resource, ID3D11Texture2D, D3D11_BIND_RENDER_TARGET,
      D3D11_BIND_SHADER_RESOURCE, D3D11_CREATE_DEVICE_BGRA_SUPPORT,
      D3D11_CREATE_DEVICE_VIDEO_SUPPORT,
    },
    Direct3D11on12::{D3D11On12CreateDevice, ID3D11On12Device, D3D11_RESOURCE_FLAGS},
    Direct3D12::{ID3D12Device, D3D12_HEAP_FLAG_NONE, D3D12_RESOURCE_STATE_COMMON},
  },
};

use super::raw_textures_windows::{committed_texture, wgpu_texture, Dx12};
use super::Gpu;

/// Direct3D 11 on wgpu's device: what Media Foundation decodes and encodes
/// on.
pub(crate) struct D3d11Layer {
  pub(crate) device: ID3D11Device,
  context: ID3D11DeviceContext,
  on12: ID3D11On12Device,
  /// Held across acquiring wrapped textures, using them and submitting, so
  /// two threads' hand-overs never interleave on the immediate context.
  hand_over: Mutex<()>,
}

// The device is multithread-protected, and every use of the immediate
// context holds `hand_over`.
unsafe impl Send for D3d11Layer {}
unsafe impl Sync for D3d11Layer {}

/// A texture both sides use: wgpu samples it, renders into it and copies it,
/// and Direct3D 11 copies into and out of it.
pub(crate) struct SharedTexture {
  pub(crate) texture: wgpu::Texture,
  pub(crate) view: wgpu::TextureView,
  d3d11: ID3D11Texture2D,
}

unsafe impl Send for SharedTexture {}
unsafe impl Sync for SharedTexture {}

/// Opened on first use and kept, like the device it is layered on.
static LAYER: LazyLock<Result<D3d11Layer, String>> =
  LazyLock::new(|| D3d11Layer::open(super::shared()?));

pub(crate) fn d3d11() -> Result<&'static D3d11Layer, String> {
  LAYER.as_ref().map_err(Clone::clone)
}

impl D3d11Layer {
  fn open(gpu: &Gpu) -> Result<Self, String> {
    let hal = unsafe { gpu.device.as_hal::<Dx12>() }
      .ok_or_else(|| "The graphics device is not Direct3D 12".to_owned())?;
    let device12: ID3D12Device = hal.raw_device().clone();
    let queue = hal
      .raw_queue()
      .cast::<windows::core::IUnknown>()
      .map_err(|error| error.to_string())?;
    drop(hal);
    let (mut device, mut context) = (None, None);
    unsafe {
      D3D11On12CreateDevice(
        &device12,
        (D3D11_CREATE_DEVICE_BGRA_SUPPORT | D3D11_CREATE_DEVICE_VIDEO_SUPPORT).0,
        Some(&[D3D_FEATURE_LEVEL_11_1, D3D_FEATURE_LEVEL_11_0]),
        Some(&[Some(queue)]),
        0,
        Some(&mut device),
        Some(&mut context),
        None,
      )
    }
    .map_err(|error| format!("Direct3D 11 could not be layered on the graphics device: {error}"))?;
    let device: ID3D11Device = device.ok_or("Direct3D 11 returned no device")?;
    let context: ID3D11DeviceContext = context.ok_or("Direct3D 11 returned no context")?;
    // Media Foundation decodes and encodes on threads of its own.
    let multithread: ID3D10Multithread = device.cast().map_err(|error| error.to_string())?;
    let _ = unsafe { multithread.SetMultithreadProtected(true) };
    let on12: ID3D11On12Device = device.cast().map_err(|error| error.to_string())?;
    Ok(Self {
      device,
      context,
      on12,
      hand_over: Mutex::new(()),
    })
  }

  /// Runs `work` on the immediate context with `textures` handed to
  /// Direct3D 11, then hands them back and submits, so wgpu work submitted
  /// afterwards sees what `work` did and `work` sees what wgpu submitted
  /// before.
  pub(crate) fn with<R>(
    &self,
    textures: &[&SharedTexture],
    work: impl FnOnce(&ID3D11DeviceContext, &[ID3D11Texture2D]) -> R,
  ) -> Result<R, String> {
    let _held = self
      .hand_over
      .lock()
      .map_err(|_| "The Direct3D 11 layer is poisoned".to_owned())?;
    let wrapped = textures
      .iter()
      .map(|texture| texture.d3d11.cast::<ID3D11Resource>().map(Some))
      .collect::<windows::core::Result<Vec<_>>>()
      .map_err(|error| error.to_string())?;
    let views: Vec<ID3D11Texture2D> = textures
      .iter()
      .map(|texture| texture.d3d11.clone())
      .collect();
    unsafe { self.on12.AcquireWrappedResources(&wrapped) };
    let result = work(&self.context, &views);
    unsafe {
      self.on12.ReleaseWrappedResources(&wrapped);
      self.context.Flush();
    }
    Ok(result)
  }

  /// A texture of `size` in `format` - `Bgra8Unorm` or `Rgba8Unorm` - that
  /// both sides use.
  pub(crate) fn shared_texture(
    &self,
    gpu: &Gpu,
    size: (u32, u32),
    format: wgpu::TextureFormat,
    label: &'static str,
  ) -> Result<SharedTexture, String> {
    let resource = committed_texture(gpu, size, format, D3D12_HEAP_FLAG_NONE, label)?;
    let mut d3d11: Option<ID3D11Texture2D> = None;
    unsafe {
      self.on12.CreateWrappedResource(
        &resource,
        &D3D11_RESOURCE_FLAGS {
          BindFlags: (D3D11_BIND_RENDER_TARGET.0 | D3D11_BIND_SHADER_RESOURCE.0) as u32,
          ..Default::default()
        },
        D3D12_RESOURCE_STATE_COMMON,
        D3D12_RESOURCE_STATE_COMMON,
        &mut d3d11,
      )
    }
    .map_err(|error| format!("Direct3D 11 could not use the {label}: {error}"))?;
    let d3d11 = d3d11.ok_or_else(|| format!("Direct3D 11 wrapped no {label}"))?;
    let (texture, view) = wgpu_texture(gpu, resource, size, format, label);
    Ok(SharedTexture {
      texture,
      view,
      d3d11,
    })
  }
}

#[cfg(test)]
#[path = "interop_windows_tests.rs"]
mod tests;
