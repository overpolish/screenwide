// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The one wgpu device every renderer shares. Shaders are WGSL everywhere;
//! on Windows they run on Direct3D 12 and compile through the bundled DXC,
//! because wgpu's default there, FXC, takes minutes on the larger shaders.

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub(crate) use self::windows::bridge::{device_on_gpu_adapter, BridgedTexture, D3d11Bridge};
#[cfg(target_os = "windows")]
pub(crate) use self::windows::interop::{d3d11, D3d11Layer, SharedTexture};
mod textures;

use std::sync::{Arc, LazyLock};

pub(crate) struct Gpu {
  /// Surfaces are created from the instance that owns the device.
  #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
  pub(crate) instance: wgpu::Instance,
  pub(crate) device: wgpu::Device,
  pub(crate) queue: wgpu::Queue,
}

/// Opened on first use. A failure is kept, so every renderer reports the same
/// reason instead of retrying a device that cannot open.
static SHARED: LazyLock<Result<Gpu, String>> = LazyLock::new(|| pollster::block_on(Gpu::open()));

pub(crate) fn shared() -> Result<&'static Gpu, String> {
  SHARED.as_ref().map_err(Clone::clone)
}

impl Gpu {
  async fn open() -> Result<Self, String> {
    let instance = wgpu::Instance::new(instance_descriptor()?);
    // A dedicated GPU where there is one; wgpu falls back to the integrated
    // GPU on a machine without one.
    let adapter = instance
      .request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        ..Default::default()
      })
      .await
      .map_err(|error| format!("A graphics adapter is required: {error}"))?;
    let info = adapter.get_info();
    eprintln!("Graphics adapter: {} ({:?})", info.name, info.device_type);
    let (device, queue) = adapter
      .request_device(&wgpu::DeviceDescriptor {
        label: Some("Screenwide"),
        ..Default::default()
      })
      .await
      .map_err(|error| format!("The graphics device could not be opened: {error}"))?;
    // wgpu's default handler panics. A rendering mistake must cost a frame,
    // not the app.
    device.on_uncaptured_error(Arc::new(|error| {
      eprintln!("The graphics device reported an error: {error}");
    }));
    Ok(Self {
      instance,
      device,
      queue,
    })
  }
}

#[cfg(target_os = "windows")]
fn instance_descriptor() -> Result<wgpu::InstanceDescriptor, String> {
  let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
  descriptor.backends = wgpu::Backends::DX12;
  descriptor.backend_options.dx12 = wgpu::Dx12BackendOptions {
    shader_compiler: wgpu::Dx12Compiler::DynamicDxc {
      dxc_path: self::windows::dxc::library_path()?,
    },
    // Overlay frames are drawn from the pointer's thread, which must never
    // wait on the compositor.
    latency_waitable_object: wgpu::Dx12UseFrameLatencyWaitableObject::None,
    ..Default::default()
  };
  Ok(descriptor)
}

#[cfg(not(target_os = "windows"))]
fn instance_descriptor() -> Result<wgpu::InstanceDescriptor, String> {
  Ok(wgpu::InstanceDescriptor::new_without_display_handle())
}
