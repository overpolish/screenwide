// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A plain Direct3D 11 device on wgpu's GPU, for screen capture, and the
//! textures and fences that carry its frames to and from wgpu.
//!
//! Windows.Graphics.Capture only recycles its frame buffers on a real
//! Direct3D 11 device; on the D3D11On12 layer it stops after its first few
//! frames. So recording captures and encodes on a device of its own, made
//! on the adapter wgpu draws with so a texture can be shared between them.
//! The two devices have separate queues: each direction has its own shared
//! fence, which one side signals and the other's queue waits on before its
//! next work.

use std::sync::Mutex;

use windows::{
  core::Interface,
  Win32::{
    Foundation::{CloseHandle, GENERIC_ALL, HANDLE, HMODULE},
    Graphics::{
      Direct3D::{D3D_DRIVER_TYPE_UNKNOWN, D3D_FEATURE_LEVEL_11_0, D3D_FEATURE_LEVEL_11_1},
      Direct3D10::ID3D10Multithread,
      Direct3D11::{
        D3D11CreateDevice, ID3D11Device, ID3D11Device5, ID3D11DeviceContext, ID3D11DeviceContext4,
        ID3D11Fence, ID3D11Texture2D, D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_SDK_VERSION,
      },
      Direct3D12::{ID3D12Device, ID3D12Fence, D3D12_FENCE_FLAG_SHARED, D3D12_HEAP_FLAG_SHARED},
      Dxgi::{CreateDXGIFactory2, IDXGIAdapter, IDXGIFactory4, DXGI_CREATE_FACTORY_FLAGS},
    },
  },
};

use super::raw_textures_windows::{committed_texture, wgpu_texture, Dx12};
use super::Gpu;

/// A Direct3D 11 device on the adapter wgpu draws with. Screen capture needs
/// a device of its own, and only one on this adapter can share a texture
/// with wgpu.
pub(crate) fn device_on_gpu_adapter(gpu: &Gpu) -> Result<ID3D11Device, String> {
  let luid = {
    let hal = unsafe { gpu.device.as_hal::<Dx12>() }
      .ok_or_else(|| "The graphics device is not Direct3D 12".to_owned())?;
    unsafe { hal.raw_device().GetAdapterLuid() }
  };
  let factory: IDXGIFactory4 = unsafe { CreateDXGIFactory2(DXGI_CREATE_FACTORY_FLAGS(0)) }
    .map_err(|error| error.to_string())?;
  let adapter: IDXGIAdapter =
    unsafe { factory.EnumAdapterByLuid(luid) }.map_err(|error| error.to_string())?;
  let mut device = None;
  unsafe {
    D3D11CreateDevice(
      &adapter,
      D3D_DRIVER_TYPE_UNKNOWN,
      HMODULE::default(),
      D3D11_CREATE_DEVICE_BGRA_SUPPORT,
      Some(&[D3D_FEATURE_LEVEL_11_1, D3D_FEATURE_LEVEL_11_0]),
      D3D11_SDK_VERSION,
      Some(&mut device),
      None,
      None,
    )
  }
  .map_err(|error| format!("The capture device could not be opened: {error}"))?;
  let device: ID3D11Device = device.ok_or("Direct3D 11 returned no capture device")?;
  // Capture, the camera and Media Foundation each use it from their own
  // threads.
  let multithread: ID3D10Multithread = device.cast().map_err(|error| error.to_string())?;
  let _ = unsafe { multithread.SetMultithreadProtected(true) };
  Ok(device)
}

/// One direction across the bridge: the fence as each device sees it, and
/// the last value signalled. Allocating a value and signalling it happen
/// under one lock, so the fence never steps backwards.
struct Lane {
  d3d12: ID3D12Fence,
  d3d11: ID3D11Fence,
  value: Mutex<u64>,
}

/// A capture device and the fences that order its work against wgpu's.
pub(crate) struct D3d11Bridge {
  pub(crate) context: ID3D11DeviceContext,
  device5: ID3D11Device5,
  context4: ID3D11DeviceContext4,
  to_gpu: Lane,
  to_d3d11: Lane,
}

// The device is multithread-protected, and every fence step holds its lane's
// lock.
unsafe impl Send for D3d11Bridge {}
unsafe impl Sync for D3d11Bridge {}

/// A texture the capture device and wgpu both open.
pub(crate) struct BridgedTexture {
  pub(crate) texture: wgpu::Texture,
  pub(crate) view: wgpu::TextureView,
  pub(crate) d3d11: ID3D11Texture2D,
}

unsafe impl Send for BridgedTexture {}
unsafe impl Sync for BridgedTexture {}

impl D3d11Bridge {
  /// The bridge for `device`, which [`device_on_gpu_adapter`] made.
  pub(crate) fn new(gpu: &Gpu, device: &ID3D11Device) -> Result<Self, String> {
    let context = unsafe { device.GetImmediateContext() }.map_err(|error| error.to_string())?;
    let device5: ID3D11Device5 = device.cast().map_err(|error| error.to_string())?;
    let context4: ID3D11DeviceContext4 = context.cast().map_err(|error| error.to_string())?;
    Ok(Self {
      to_gpu: lane(gpu, &device5)?,
      to_d3d11: lane(gpu, &device5)?,
      context,
      device5,
      context4,
    })
  }

  /// Makes every later wgpu submission wait for the Direct3D 11 work recorded
  /// so far, such as a captured frame copied into a bridged texture.
  pub(crate) fn d3d11_to_gpu(&self, gpu: &Gpu) -> Result<(), String> {
    let mut value = self
      .to_gpu
      .value
      .lock()
      .map_err(|_| "A frame fence is poisoned")?;
    *value += 1;
    unsafe { self.context4.Signal(&self.to_gpu.d3d11, *value) }
      .map_err(|error| error.to_string())?;
    // The signal has to reach the GPU before Direct3D 12 can see it.
    unsafe { self.context4.Flush() };
    let hal = unsafe { gpu.device.as_hal::<Dx12>() }
      .ok_or_else(|| "The graphics device is not Direct3D 12".to_owned())?;
    unsafe { hal.raw_queue().Wait(&self.to_gpu.d3d12, *value) }.map_err(|error| error.to_string())
  }

  /// Makes every later Direct3D 11 command wait for the wgpu work submitted
  /// so far, such as a canvas drawn for the encoder.
  pub(crate) fn gpu_to_d3d11(&self, gpu: &Gpu) -> Result<(), String> {
    let mut value = self
      .to_d3d11
      .value
      .lock()
      .map_err(|_| "A frame fence is poisoned")?;
    *value += 1;
    let hal = unsafe { gpu.device.as_hal::<Dx12>() }
      .ok_or_else(|| "The graphics device is not Direct3D 12".to_owned())?;
    unsafe { hal.raw_queue().Signal(&self.to_d3d11.d3d12, *value) }
      .map_err(|error| error.to_string())?;
    unsafe { self.context4.Wait(&self.to_d3d11.d3d11, *value) }.map_err(|error| error.to_string())
  }

  /// A texture of `size` in `format` - `Bgra8Unorm` or `Rgba8Unorm` - that
  /// both devices open: wgpu samples it and renders into it, and Direct3D 11
  /// copies into and out of it.
  pub(crate) fn shared_texture(
    &self,
    gpu: &Gpu,
    size: (u32, u32),
    format: wgpu::TextureFormat,
    label: &'static str,
  ) -> Result<BridgedTexture, String> {
    let resource = committed_texture(gpu, size, format, D3D12_HEAP_FLAG_SHARED, label)?;
    let handle = {
      let hal = unsafe { gpu.device.as_hal::<Dx12>() }
        .ok_or_else(|| "The graphics device is not Direct3D 12".to_owned())?;
      unsafe {
        hal
          .raw_device()
          .CreateSharedHandle(&resource, None, GENERIC_ALL.0, None)
      }
      .map_err(|error| format!("The {label} could not be exported: {error}"))?
    };
    let d3d11 = open_shared::<ID3D11Texture2D>(&self.device5, handle)
      .map_err(|error| format!("Direct3D 11 could not open the {label}: {error}"))?;
    let (texture, view) = wgpu_texture(gpu, resource, size, format, label);
    Ok(BridgedTexture {
      texture,
      view,
      d3d11,
    })
  }
}

/// A fence Direct3D 12 creates and Direct3D 11 opens.
fn lane(gpu: &Gpu, device5: &ID3D11Device5) -> Result<Lane, String> {
  let hal = unsafe { gpu.device.as_hal::<Dx12>() }
    .ok_or_else(|| "The graphics device is not Direct3D 12".to_owned())?;
  let raw: &ID3D12Device = hal.raw_device();
  let d3d12: ID3D12Fence =
    unsafe { raw.CreateFence(0, D3D12_FENCE_FLAG_SHARED) }.map_err(|error| error.to_string())?;
  let handle = unsafe { raw.CreateSharedHandle(&d3d12, None, GENERIC_ALL.0, None) }
    .map_err(|error| format!("A frame fence could not be exported: {error}"))?;
  let mut d3d11: Option<ID3D11Fence> = None;
  let opened = unsafe { device5.OpenSharedFence(handle, &mut d3d11) };
  let _ = unsafe { CloseHandle(handle) };
  opened.map_err(|error| format!("Direct3D 11 could not open a frame fence: {error}"))?;
  Ok(Lane {
    d3d12,
    d3d11: d3d11.ok_or("Direct3D 11 opened no frame fence")?,
    value: Mutex::new(0),
  })
}

fn open_shared<T: Interface>(device5: &ID3D11Device5, handle: HANDLE) -> windows::core::Result<T> {
  let opened = unsafe { device5.OpenSharedResource1::<T>(handle) };
  let _ = unsafe { CloseHandle(handle) };
  opened
}
